use std::sync::MutexGuard;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::auth::{self, RpcError, Session};
use crate::commands::LiveState;
use crate::db;
use crate::error::AppError;
use crate::models::{SyncConflictDto, SyncStatusDto};

const PUSH_BATCH: i64 = 50;
const PULL_LIMIT: i64 = 500;

struct SyncStateRow {
    account_id: Option<String>,
    sync_enabled: bool,
    epoch: i64,
    cursor: i64,
    session_generation: i64,
    reconcile_required: bool,
    last_sync_at: Option<String>,
    last_error: Option<String>,
}

fn guard(live: &LiveState) -> Result<MutexGuard<'_, Connection>, AppError> {
    live.db
        .lock()
        .map_err(|_| AppError::internal("数据库锁失效"))
}

fn read_state(conn: &Connection) -> Result<SyncStateRow, AppError> {
    conn.query_row(
        "SELECT account_id, sync_enabled, epoch, cursor, session_generation,
                reconcile_required, last_sync_at, last_error
         FROM sync_state WHERE id = 1",
        [],
        |row| {
            Ok(SyncStateRow {
                account_id: row.get(0)?,
                sync_enabled: row.get::<_, i64>(1)? == 1,
                epoch: row.get(2)?,
                cursor: row.get(3)?,
                session_generation: row.get(4)?,
                reconcile_required: row.get::<_, i64>(5)? == 1,
                last_sync_at: row.get(6)?,
                last_error: row.get(7)?,
            })
        },
    )
    .map_err(AppError::from)
}

fn set_last_error(conn: &Connection, error: Option<&str>) -> Result<(), AppError> {
    conn.execute(
        "UPDATE sync_state SET last_error = ?1 WHERE id = 1",
        params![error],
    )?;
    Ok(())
}

fn current_session(live: &LiveState) -> Option<Session> {
    live.session.lock().ok().and_then(|slot| slot.clone())
}

/// 带一次 401 刷新的 RPC 调用。
async fn rpc(live: &LiveState, function: &str, params: Value) -> Result<Value, AppError> {
    let session = current_session(live).ok_or_else(|| AppError::new("AUTH", "未登录"))?;
    let session = auth::ensure_fresh(&live.http, &live.data_dir, session).await?;
    if let Ok(mut slot) = live.session.lock() {
        *slot = Some(session.clone());
    }
    match auth::rpc(&live.http, &session, function, params.clone()).await {
        Err(RpcError::Auth(_)) => {
            let refreshed = auth::refresh(&live.http, &live.data_dir, &session).await?;
            if let Ok(mut slot) = live.session.lock() {
                *slot = Some(refreshed.clone());
            }
            auth::rpc(&live.http, &refreshed, function, params)
                .await
                .map_err(rpc_error)
        }
        other => other.map_err(rpc_error),
    }
}

fn rpc_error(err: RpcError) -> AppError {
    match err {
        RpcError::Auth(msg) => AppError::new("AUTH_EXPIRED", msg),
        RpcError::Transport(e) => e,
    }
}

/// 拉取 + 推送一轮。供 worker 与手动 sync_now 使用；未开启/未登录/对账态直接跳过。
pub async fn run_once(live: &LiveState) -> Result<(), AppError> {
    let (can_run, generation) = {
        let conn = guard(live)?;
        let state = read_state(&conn)?;
        (
            state.sync_enabled && state.account_id.is_some() && !state.reconcile_required,
            state.session_generation,
        )
    };
    if !can_run || current_session(live).is_none() {
        return Ok(());
    }
    match pull_all(live, generation).await {
        Ok(()) => {}
        Err(err) => {
            let conn = guard(live)?;
            set_last_error(&conn, Some(&err.message))?;
            return Err(err);
        }
    }
    match push_all(live, generation).await {
        Ok(()) => {}
        Err(err) => {
            let conn = guard(live)?;
            set_last_error(&conn, Some(&err.message))?;
            return Err(err);
        }
    }
    let conn = guard(live)?;
    conn.execute(
        "UPDATE sync_state SET last_sync_at = ?1, last_error = NULL WHERE id = 1",
        params![crate::validate::now_iso()],
    )?;
    Ok(())
}

async fn pull_all(live: &LiveState, generation: i64) -> Result<(), AppError> {
    loop {
        let (cursor, epoch) = {
            let conn = guard(live)?;
            let state = read_state(&conn)?;
            (state.cursor, state.epoch)
        };
        let resp = rpc(
            live,
            "kiroku_sync_pull",
            json!({ "p_cursor": cursor, "p_limit": PULL_LIMIT }),
        )
        .await?;
        let status = resp.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "no_access" {
            let conn = guard(live)?;
            set_last_error(&conn, Some("会员已到期，云端数据进入保留期"))?;
            return Ok(());
        }
        if status == "cursor_expired" {
            let conn = guard(live)?;
            conn.execute(
                "UPDATE sync_state SET reconcile_required = 1, last_error = '本地游标过期，需要对账' WHERE id = 1",
                [],
            )?;
            return Ok(());
        }
        let remote_epoch = resp.get("epoch").and_then(|v| v.as_i64()).unwrap_or(epoch);
        if epoch != 0 && remote_epoch != epoch {
            let conn = guard(live)?;
            conn.execute(
                "UPDATE sync_state SET reconcile_required = 1, last_error = '云端库已重建，需要对账' WHERE id = 1",
                [],
            )?;
            return Ok(());
        }

        let conn = guard(live)?;
        if read_state(&conn)?.session_generation != generation {
            return Ok(()); // 会话已切换，丢弃在途结果
        }
        let tx = conn.unchecked_transaction()?;
        if resp.get("mode").and_then(|v| v.as_str()) == Some("snapshot") {
            if let Some(items) = resp.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    apply_remote_item(&tx, item)?;
                }
            }
            let boundary = resp.get("boundary").and_then(|v| v.as_i64()).unwrap_or(0);
            tx.execute(
                "UPDATE sync_state SET cursor = ?1, epoch = ?2 WHERE id = 1",
                params![boundary, remote_epoch],
            )?;
            tx.commit()?;
            return Ok(());
        }
        let mut next_cursor = cursor;
        if let Some(changes) = resp.get("changes").and_then(|v| v.as_array()) {
            for change in changes {
                apply_remote_item(&tx, change)?;
            }
        }
        next_cursor = resp
            .get("next_cursor")
            .and_then(|v| v.as_i64())
            .unwrap_or(next_cursor);
        tx.execute(
            "UPDATE sync_state SET cursor = ?1 WHERE id = 1",
            params![next_cursor],
        )?;
        tx.commit()?;
        if !resp
            .get("has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Ok(());
        }
        drop(conn);
    }
}

/// 应用一条远端变更/快照项；与本地待传 op 撞车则记冲突不落地。
fn apply_remote_item(conn: &Connection, item: &Value) -> Result<(), AppError> {
    let entity_type = item
        .get("entity_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let entity_key = item
        .get("entity_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let op = item.get("op").and_then(|v| v.as_str()).unwrap_or("");
    let version = item
        .get("server_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let payload = item.get("payload").cloned().unwrap_or(Value::Null);
    let applied = match (entity_type, op) {
        ("record", "upsert") => {
            let subject_id: i64 = entity_key
                .parse()
                .map_err(|_| AppError::internal("远端记录键无效"))?;
            let record = payload.get("record").cloned().unwrap_or(Value::Null);
            let snapshot = payload.get("snapshot").cloned();
            db::apply_remote_record_upsert(conn, subject_id, &record, snapshot.as_ref(), version)?
        }
        ("record", "delete") => {
            let subject_id: i64 = entity_key
                .parse()
                .map_err(|_| AppError::internal("远端记录键无效"))?;
            db::apply_remote_record_delete(conn, subject_id)?
        }
        ("tier", "upsert") => db::apply_remote_tier_upsert(conn, entity_key, &payload, version)?,
        ("tier", "delete") => db::apply_remote_tier_delete(conn, entity_key)?,
        ("tier_order", "set") => {
            let keys = payload
                .get("ordered_keys")
                .and_then(|v| v.as_array())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            db::apply_remote_tier_order(conn, &keys, version)?
        }
        _ => true,
    };
    if !applied {
        let local = local_entity_payload(conn, entity_type, entity_key)?;
        let remote = json!({ "op": if op.is_empty() { "upsert" } else { op }, "data": payload });
        db::upsert_conflict(
            conn,
            entity_type,
            entity_key,
            local.as_ref(),
            Some(&remote),
            Some(version),
        )?;
    }
    Ok(())
}

/// 本地实体当前状态（冲突展示用）。
fn local_entity_payload(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
) -> Result<Option<Value>, AppError> {
    match entity_type {
        "record" => {
            let subject_id: i64 = entity_key.parse().unwrap_or(0);
            let local_id: Option<i64> = conn
                .query_row(
                    "SELECT s.id FROM subjects s JOIN personal_records p ON p.subject_id = s.id WHERE s.bangumi_subject_id = ?1",
                    params![subject_id],
                    |row| row.get(0),
                )
                .optional()?;
            match local_id {
                Some(id) => Ok(Some(db::record_sync_payload(conn, id)?.0)),
                None => Ok(None),
            }
        }
        "tier" => {
            let tier_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM tiers WHERE sync_id = ?1",
                    params![entity_key],
                    |row| row.get(0),
                )
                .optional()?;
            match tier_id {
                Some(id) => Ok(Some(db::tier_sync_payload(conn, id)?.0)),
                None => Ok(None),
            }
        }
        "tier_order" => Ok(Some(db::tier_order_payload(conn)?.0)),
        _ => Ok(None),
    }
}

async fn push_all(live: &LiveState, generation: i64) -> Result<(), AppError> {
    loop {
        let ops = {
            let conn = guard(live)?;
            let mut stmt = conn.prepare(
                "SELECT op_id, entity_type, entity_key, op_type, payload_json, base_server_version
                 FROM sync_outbox ORDER BY seq LIMIT ?1",
            )?;
            let rows = stmt
                .query_map(params![PUSH_BATCH], |row| {
                    Ok(json!({
                        "op_id": row.get::<_, String>(0)?,
                        "entity_type": row.get::<_, String>(1)?,
                        "entity_key": row.get::<_, String>(2)?,
                        "op_type": row.get::<_, String>(3)?,
                        "payload": serde_json::from_str::<Value>(&row.get::<_, String>(4)?).unwrap_or(Value::Null),
                        "base_server_version": row.get::<_, i64>(5)?,
                    }))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        if ops.is_empty() {
            return Ok(());
        }
        let sent = ops.len();
        let resp = rpc(live, "kiroku_sync_push", json!({ "p_ops": ops })).await?;
        if resp.get("status").and_then(|v| v.as_str()) == Some("entitlement_expired") {
            let conn = guard(live)?;
            set_last_error(&conn, Some("会员已到期，本地修改保留待续期后上传"))?;
            return Ok(());
        }
        let conn = guard(live)?;
        if read_state(&conn)?.session_generation != generation {
            return Ok(());
        }
        let tx = conn.unchecked_transaction()?;
        let mut conflicts = 0_i64;
        if let Some(results) = resp.get("results").and_then(|v| v.as_array()) {
            for result in results {
                let op_id = result.get("op_id").and_then(|v| v.as_str()).unwrap_or("");
                let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let server_version = result.get("server_version").and_then(|v| v.as_i64());
                match status {
                    "applied" | "replayed" => {
                        if let Some(version) = server_version {
                            mark_applied(&tx, op_id, version)?;
                        }
                    }
                    "conflict" => {
                        conflicts += 1;
                        let meta: Option<(String, String)> = tx
                            .query_row(
                                "SELECT entity_type, entity_key FROM sync_outbox WHERE op_id = ?1",
                                params![op_id],
                                |row| Ok((row.get(0)?, row.get(1)?)),
                            )
                            .optional()?;
                        if let Some((entity_type, entity_key)) = meta {
                            // 撤下该实体全部排队 op；本地最新态由解决流程重新生成
                            tx.execute(
                                "DELETE FROM sync_outbox WHERE entity_type = ?1 AND entity_key = ?2",
                                params![entity_type, entity_key],
                            )?;
                            let local = local_entity_payload(&tx, &entity_type, &entity_key)?;
                            let remote = result
                                .get("remote")
                                .map(|r| json!({ "op": "upsert", "data": r }));
                            db::upsert_conflict(
                                &tx,
                                &entity_type,
                                &entity_key,
                                local.as_ref(),
                                remote.as_ref(),
                                server_version,
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        }
        tx.commit()?;
        drop(conn);
        if resp
            .get("results")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
            >= sent
            && conflicts == 0
        {
            continue;
        }
        // 服务端遇冲突停批：本批未处理的留在队列，下一轮继续
        if sent as usize <= 1 {
            return Ok(());
        }
        // 继续推剩余批次
        let remaining: i64 = {
            let conn = guard(live)?;
            conn.query_row("SELECT COUNT(*) FROM sync_outbox", [], |row| row.get(0))?
        };
        if remaining == 0 {
            return Ok(());
        }
    }
}

/// op 确认：删 outbox 行 + 对齐实体 server_version。
fn mark_applied(conn: &Connection, op_id: &str, server_version: i64) -> Result<(), AppError> {
    let meta: Option<(String, String)> = conn
        .query_row(
            "SELECT entity_type, entity_key FROM sync_outbox WHERE op_id = ?1",
            params![op_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    conn.execute("DELETE FROM sync_outbox WHERE op_id = ?1", params![op_id])?;
    if let Some((entity_type, entity_key)) = meta {
        match entity_type.as_str() {
            "record" => {
                if let Ok(subject_id) = entity_key.parse::<i64>() {
                    conn.execute(
                        "UPDATE personal_records SET server_version = ?1
                         WHERE subject_id = (SELECT id FROM subjects WHERE bangumi_subject_id = ?2)",
                        params![server_version, subject_id],
                    )?;
                }
            }
            "tier" => {
                conn.execute(
                    "UPDATE tiers SET server_version = ?1 WHERE sync_id = ?2",
                    params![server_version, entity_key],
                )?;
            }
            "tier_order" => {
                conn.execute(
                    "UPDATE sync_state SET tier_order_server_version = ?1 WHERE id = 1",
                    params![server_version],
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// 首次开启同步：拉云端快照与本地合并（SYNC_DESIGN §7.3）。
/// 返回冲突数；0 则合并完成并开启同步。
pub async fn initial_enable(live: &LiveState) -> Result<i64, AppError> {
    if current_session(live).is_none() {
        return Err(AppError::new("AUTH", "请先登录"));
    }
    let entitlement = rpc(live, "kiroku_get_entitlement", json!({})).await?;
    if !entitlement
        .get("member_active")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Err(AppError::new(
            "MEMBERSHIP",
            "云同步需要有效会员，请先兑换会员码",
        ));
    }
    let snap = rpc(
        live,
        "kiroku_sync_pull",
        json!({ "p_cursor": 0, "p_limit": PULL_LIMIT }),
    )
    .await?;
    if snap.get("status").and_then(|v| v.as_str()) == Some("no_access") {
        return Err(AppError::new(
            "MEMBERSHIP",
            "云同步需要有效会员，请先兑换会员码",
        ));
    }
    let remote_epoch = snap.get("epoch").and_then(|v| v.as_i64()).unwrap_or(1);
    let boundary = snap.get("boundary").and_then(|v| v.as_i64()).unwrap_or(0);
    let items = snap
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let conflict_count = merge_snapshot(live, &items, remote_epoch, boundary)?;
    if conflict_count == 0 {
        let _ = run_once(live).await;
    }
    Ok(conflict_count)
}

/// 快照合并（同步函数，内部事务）：远端项与本地单边项对齐，返回冲突数。
/// 不清 outbox/conflicts：上次合并中用户已做的 keep_local 解决结果（排队 op）
/// 和未解决的冲突都要原样保留——清掉会让"保本地"的解决结果丢失。
fn merge_snapshot(
    live: &LiveState,
    items: &[Value],
    remote_epoch: i64,
    boundary: i64,
) -> Result<i64, AppError> {
    let conn = guard(live)?;
    let tx = conn.unchecked_transaction()?;

    let mut remote_record_keys: Vec<String> = Vec::new();
    let mut remote_tier_keys: Vec<String> = Vec::new();
    let mut remote_order = false;
    for item in items {
        let entity_type = item
            .get("entity_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let entity_key = item
            .get("entity_key")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        match entity_type {
            "record" => remote_record_keys.push(entity_key.to_string()),
            "tier" => remote_tier_keys.push(entity_key.to_string()),
            "tier_order" => remote_order = true,
            _ => {}
        }
        // 已有排队 op（含已解决的本地优先）或已有冲突 → 不重复判定
        if db::entity_has_pending_op(&tx, entity_type, entity_key)?
            || conflict_exists(&tx, entity_type, entity_key)?
        {
            continue;
        }
        let version = item
            .get("server_version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let payload = item.get("payload").cloned().unwrap_or(Value::Null);
        match entity_type {
            "record" => {
                let subject_id: i64 = entity_key.parse().unwrap_or(0);
                let local = local_entity_payload(&tx, "record", entity_key)?;
                match local {
                    None => {
                        let record = payload.get("record").cloned().unwrap_or(Value::Null);
                        let snapshot = payload.get("snapshot").cloned();
                        db::apply_remote_record_upsert(
                            &tx,
                            subject_id,
                            &record,
                            snapshot.as_ref(),
                            version,
                        )?;
                    }
                    Some(ref local_payload) => {
                        if records_equal(local_payload, &payload) {
                            tx.execute(
                                "UPDATE personal_records SET server_version = ?1
                                 WHERE subject_id = (SELECT id FROM subjects WHERE bangumi_subject_id = ?2)",
                                params![version, subject_id],
                            )?;
                        } else {
                            let remote = json!({ "op": "upsert", "data": payload });
                            db::upsert_conflict(
                                &tx,
                                "record",
                                entity_key,
                                local.as_ref(),
                                Some(&remote),
                                Some(version),
                            )?;
                        }
                    }
                }
            }
            "tier" => {
                let local = local_entity_payload(&tx, "tier", entity_key)?;
                match local {
                    None => {
                        db::apply_remote_tier_upsert(&tx, entity_key, &payload, version)?;
                    }
                    Some(ref local_payload) => {
                        if tiers_equal(local_payload, &payload) {
                            tx.execute(
                                "UPDATE tiers SET server_version = ?1 WHERE sync_id = ?2",
                                params![version, entity_key],
                            )?;
                        } else {
                            let remote = json!({ "op": "upsert", "data": payload });
                            db::upsert_conflict(
                                &tx,
                                "tier",
                                entity_key,
                                local.as_ref(),
                                Some(&remote),
                                Some(version),
                            )?;
                        }
                    }
                }
            }
            "tier_order" => {
                let local = local_entity_payload(&tx, "tier_order", "tier_order")?;
                if !tier_order_equal(local.as_ref(), &payload) {
                    let remote = json!({ "op": "set", "data": payload });
                    db::upsert_conflict(
                        &tx,
                        "tier_order",
                        "tier_order",
                        local.as_ref(),
                        Some(&remote),
                        Some(version),
                    )?;
                } else {
                    tx.execute(
                        "UPDATE sync_state SET tier_order_server_version = ?1 WHERE id = 1",
                        params![version],
                    )?;
                }
            }
            _ => {}
        }
    }

    let conflict_count: i64 =
        tx.query_row("SELECT COUNT(*) FROM sync_conflicts", [], |row| row.get(0))?;

    // 单边存在的本地实体作为 base=0 上行（有冲突的留给解决流程）
    let mut stmt = tx.prepare(
        "SELECT s.id, s.bangumi_subject_id FROM personal_records p
         JOIN subjects s ON s.id = p.subject_id
         WHERE p.server_version = 0",
    )?;
    let local_only = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);
    for (local_id, bangumi_id) in local_only {
        let key = bangumi_id.to_string();
        if remote_record_keys.contains(&key) || conflict_exists(&tx, "record", &key)? {
            continue;
        }
        let (payload, _) = db::record_sync_payload(&tx, local_id)?;
        db::enqueue_sync_op_replace(&tx, "record", &key, "upsert", &payload, 0)?;
    }
    let mut stmt = tx.prepare("SELECT id, sync_id FROM tiers WHERE server_version = 0")?;
    let local_tiers = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);
    for (tier_id, sync_id) in local_tiers {
        let Some(key) = sync_id else { continue };
        if remote_tier_keys.contains(&key) || conflict_exists(&tx, "tier", &key)? {
            continue;
        }
        let (payload, _, _) = db::tier_sync_payload(&tx, tier_id)?;
        db::enqueue_sync_op_replace(&tx, "tier", &key, "upsert", &payload, 0)?;
    }
    if !remote_order {
        let (payload, _) = db::tier_order_payload(&tx)?;
        db::enqueue_sync_op_replace(&tx, "tier_order", "tier_order", "set", &payload, 0)?;
    }

    if conflict_count == 0 {
        tx.execute(
            "UPDATE sync_state SET sync_enabled = 1, epoch = ?1, cursor = ?2, reconcile_required = 0, last_error = NULL WHERE id = 1",
            params![remote_epoch, boundary],
        )?;
    }
    tx.commit()?;
    Ok(conflict_count)
}

fn conflict_exists(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_conflicts WHERE entity_type = ?1 AND entity_key = ?2",
        params![entity_type, entity_key],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn records_equal(local: &Value, remote: &Value) -> bool {
    let l = local.get("record").cloned().unwrap_or(Value::Null);
    let r = remote.get("record").cloned().unwrap_or(Value::Null);
    let pick = |v: &Value, key: &str| v.get(key).cloned().unwrap_or(Value::Null);
    for key in ["score", "status", "progress", "review", "tier_sync_id"] {
        if pick(&l, key) != pick(&r, key) {
            return false;
        }
    }
    let norm = |v: &Value| -> Vec<(String, Value)> {
        let mut out: Vec<(String, Value)> = v
            .get("dimensions")
            .and_then(|d| d.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        for dim in ["story", "characters", "direction", "animation", "music"] {
            if !out.iter().any(|(k, _)| k == dim) {
                out.push((dim.to_string(), Value::Null));
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    };
    norm(&l) == norm(&r)
}

fn tiers_equal(local: &Value, remote: &Value) -> bool {
    for key in ["name", "description", "color", "builtin"] {
        if local.get(key).cloned().unwrap_or(Value::Null)
            != remote.get(key).cloned().unwrap_or(Value::Null)
        {
            return false;
        }
    }
    true
}

fn tier_order_equal(local: Option<&Value>, remote: &Value) -> bool {
    let local_keys = local
        .and_then(|v| v.get("ordered_keys"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let remote_keys = remote
        .get("ordered_keys")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    local_keys == remote_keys
}

/// 冲突解决：keep=local 以本地为准生成新 op；keep=remote 应用远端载荷。
pub fn resolve_conflict(
    live: &LiveState,
    entity_type: &str,
    entity_key: &str,
    keep: &str,
) -> Result<(), AppError> {
    let conn = guard(live)?;
    let tx = conn.unchecked_transaction()?;
    let row: Option<(Option<String>, Option<String>, Option<i64>)> = tx
        .query_row(
            "SELECT local_payload_json, remote_payload_json, remote_server_version
             FROM sync_conflicts WHERE entity_type = ?1 AND entity_key = ?2",
            params![entity_type, entity_key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((_, remote_json, remote_version)) = row else {
        return Err(AppError::not_found("冲突不存在"));
    };
    let base = remote_version.unwrap_or(0);
    match keep {
        "local" => {
            let local = local_entity_payload(&tx, entity_type, entity_key)?;
            let op_type = match entity_type {
                "tier_order" => "set",
                _ if local.is_some() => "upsert",
                _ => "delete",
            };
            let payload = local.unwrap_or(Value::Null);
            db::enqueue_sync_op_replace(&tx, entity_type, entity_key, op_type, &payload, base)?;
        }
        "remote" => {
            let remote: Value = remote_json
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(Value::Null);
            let op = remote
                .get("op")
                .and_then(|v| v.as_str())
                .unwrap_or("upsert");
            let data = remote.get("data").cloned().unwrap_or(Value::Null);
            match (entity_type, op) {
                ("record", "upsert") => {
                    let subject_id: i64 = entity_key.parse().unwrap_or(0);
                    let record = data.get("record").cloned().unwrap_or(Value::Null);
                    let snapshot = data.get("snapshot").cloned();
                    db::apply_remote_record_upsert(
                        &tx,
                        subject_id,
                        &record,
                        snapshot.as_ref(),
                        base,
                    )?;
                }
                ("record", "delete") => {
                    let subject_id: i64 = entity_key.parse().unwrap_or(0);
                    db::apply_remote_record_delete(&tx, subject_id)?;
                }
                ("tier", "upsert") => {
                    db::apply_remote_tier_upsert(&tx, entity_key, &data, base)?;
                }
                ("tier", "delete") => {
                    db::apply_remote_tier_delete(&tx, entity_key)?;
                }
                ("tier_order", _) => {
                    let keys = data
                        .get("ordered_keys")
                        .and_then(|v| v.as_array())
                        .map(|list| {
                            list.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default();
                    db::apply_remote_tier_order(&tx, &keys, base)?;
                }
                _ => {}
            }
        }
        _ => return Err(AppError::validation("keep 只能是 local 或 remote")),
    }
    tx.execute(
        "DELETE FROM sync_conflicts WHERE entity_type = ?1 AND entity_key = ?2",
        params![entity_type, entity_key],
    )?;
    tx.commit()?;
    Ok(())
}

/// 对账选择（游标过期/云端重建后）：rebuild=以本地重建云端；overwrite=以云端覆盖本地。
pub async fn reconcile(live: &LiveState, mode: &str) -> Result<(), AppError> {
    match mode {
        "rebuild" => {
            let conn = guard(live)?;
            let tx = conn.unchecked_transaction()?;
            tx.execute("DELETE FROM sync_outbox", [])?;
            tx.execute("DELETE FROM sync_conflicts", [])?;
            let mut stmt = tx.prepare(
                "SELECT s.id, s.bangumi_subject_id FROM personal_records p JOIN subjects s ON s.id = p.subject_id",
            )?;
            let records = stmt
                .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            drop(stmt);
            for (local_id, bangumi_id) in records {
                let (payload, _) = db::record_sync_payload(&tx, local_id)?;
                db::enqueue_sync_op_replace(
                    &tx,
                    "record",
                    &bangumi_id.to_string(),
                    "upsert",
                    &payload,
                    0,
                )?;
            }
            let mut stmt = tx.prepare("SELECT id, sync_id FROM tiers")?;
            let tiers = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            drop(stmt);
            for (tier_id, sync_id) in tiers {
                let Some(key) = sync_id else { continue };
                let (payload, _, _) = db::tier_sync_payload(&tx, tier_id)?;
                db::enqueue_sync_op_replace(&tx, "tier", &key, "upsert", &payload, 0)?;
            }
            let (payload, _) = db::tier_order_payload(&tx)?;
            db::enqueue_sync_op_replace(&tx, "tier_order", "tier_order", "set", &payload, 0)?;
            tx.execute(
                "UPDATE sync_state SET cursor = 0, reconcile_required = 0, sync_enabled = 1, last_error = NULL WHERE id = 1",
                [],
            )?;
            tx.commit()?;
        }
        "overwrite" => {
            // delete_personal_data 会把 account_id 置 NULL，对账覆盖需保留账号绑定
            let account_id = current_session(live).map(|s| s.user_id);
            let conn = guard(live)?;
            // 先落一份本地快照再清库（SYNC_DESIGN §7.2）
            let dest = live.data_dir.join("backups").join(format!(
                "kiroku-pre-overwrite-{}.db",
                crate::validate::now_iso().replace(':', "-")
            ));
            db::backup_to_file(&conn, &dest)?;
            drop(conn);
            let mut conn = guard(live)?;
            db::delete_personal_data(&mut conn)?;
            conn.execute(
                "UPDATE sync_state SET account_id = ?1, cursor = 0, reconcile_required = 0, sync_enabled = 1 WHERE id = 1",
                params![account_id],
            )?;
        }
        _ => return Err(AppError::validation("mode 只能是 rebuild 或 overwrite")),
    }
    let _ = run_once(live).await;
    Ok(())
}

/// 清空云端库后的本端处理：进入对账态。
pub async fn delete_cloud(live: &LiveState, confirm: &str) -> Result<Value, AppError> {
    if confirm != "DELETE" {
        return Err(AppError::validation("需要确认串 DELETE"));
    }
    let resp = rpc(
        live,
        "kiroku_delete_cloud_library",
        json!({ "p_confirm": "DELETE" }),
    )
    .await?;
    let conn = guard(live)?;
    let epoch = resp.get("epoch").and_then(|v| v.as_i64()).unwrap_or(0);
    conn.execute(
        "UPDATE sync_state SET epoch = ?1, cursor = 0, sync_enabled = 0, reconcile_required = 1,
            last_error = '云端库已清空，请选择重建或保留本地' WHERE id = 1",
        params![epoch],
    )?;
    Ok(resp)
}

/// 权益查询（状态栏展示用）。
pub async fn get_entitlement(live: &LiveState) -> Result<Value, AppError> {
    rpc(live, "kiroku_get_entitlement", json!({})).await
}

/// 兑换码。
pub async fn redeem(live: &LiveState, code: &str) -> Result<Value, AppError> {
    rpc(
        live,
        "kiroku_redeem_code",
        json!({ "p_code": code, "p_request_id": uuid::Uuid::new_v4().to_string() }),
    )
    .await
}

/// 登录成功后的账号绑定：同账号续绑、异账号拒绝。
pub fn bind_account(live: &LiveState, user_id: &str) -> Result<(), AppError> {
    let conn = guard(live)?;
    let state = read_state(&conn)?;
    match state.account_id.as_deref() {
        Some(existing) if existing != user_id => Err(AppError::conflict(
            "本机数据库已绑定其他账号，请先备份或在设置中重置本地数据",
        )),
        _ => {
            conn.execute(
                "UPDATE sync_state SET account_id = ?1, session_generation = session_generation + 1 WHERE id = 1",
                params![user_id],
            )?;
            Ok(())
        }
    }
}

pub fn bump_session_generation(live: &LiveState) -> Result<(), AppError> {
    let conn = guard(live)?;
    conn.execute(
        "UPDATE sync_state SET session_generation = session_generation + 1 WHERE id = 1",
        [],
    )?;
    Ok(())
}

pub fn set_sync_enabled(live: &LiveState, enabled: bool) -> Result<(), AppError> {
    let conn = guard(live)?;
    conn.execute(
        "UPDATE sync_state SET sync_enabled = ?1 WHERE id = 1",
        params![if enabled { 1 } else { 0 }],
    )?;
    Ok(())
}

pub fn status(live: &LiveState) -> Result<SyncStatusDto, AppError> {
    let conn = guard(live)?;
    let state = read_state(&conn)?;
    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM sync_outbox", [], |row| row.get(0))?;
    let conflicts: i64 =
        conn.query_row("SELECT COUNT(*) FROM sync_conflicts", [], |row| row.get(0))?;
    let session = current_session(live);
    Ok(SyncStatusDto {
        logged_in: session.is_some(),
        email: session.map(|s| s.email),
        account_id: state.account_id,
        sync_enabled: state.sync_enabled,
        member_active: None,
        expires_at: None,
        in_retention: None,
        pending_ops: pending,
        conflict_count: conflicts,
        epoch: state.epoch,
        cursor: state.cursor,
        reconcile_required: state.reconcile_required,
        last_sync_at: state.last_sync_at,
        last_error: state.last_error,
    })
}

pub fn list_conflicts(live: &LiveState) -> Result<Vec<SyncConflictDto>, AppError> {
    let conn = guard(live)?;
    let mut stmt = conn.prepare(
        "SELECT entity_type, entity_key, local_payload_json, remote_payload_json, remote_server_version, detected_at
         FROM sync_conflicts ORDER BY detected_at",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SyncConflictDto {
                entity_type: row.get(0)?,
                entity_key: row.get(1)?,
                local_payload: row
                    .get::<_, Option<String>>(2)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                remote_payload: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                remote_server_version: row.get(4)?,
                detected_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bangumi::BangumiClient;
    use crate::models::{CommunityDto, DimensionsDto, PersonalDraftDto, SubjectDto};
    use std::sync::{Arc, Mutex};

    fn subject(id: i64) -> SubjectDto {
        SubjectDto {
            id,
            name: "E2E".into(),
            name_cn: "端到端".into(),
            aliases: None,
            summary: "e2e".into(),
            cover_url: "https://example.invalid/c.jpg".into(),
            cover_local_path: None,
            year: 2024,
            format: "TV".into(),
            episodes: 12,
            studio: "S".into(),
            tags: vec![],
            community: CommunityDto {
                score: None,
                votes: 0,
                rank: None,
                fetched_at: None,
                status: None,
            },
        }
    }

    fn draft(score: f64) -> PersonalDraftDto {
        PersonalDraftDto {
            score: Some(score),
            tier: Some("A".into()),
            status: "completed".into(),
            progress: None,
            dimensions: DimensionsDto {
                story: Some(4.0),
                characters: None,
                direction: None,
                animation: None,
                music: None,
            },
            review: "e2e".into(),
        }
    }

    /// 真实 dev 项目端到端：登录 → 兑换 → 开启同步 → 推送 → 拉取 → 清库。
    /// 需要 KIROKU_E2E_EMAIL / KIROKU_E2E_PASSWORD / KIROKU_E2E_CODE（一次性码，用完作废）。
    /// 手动运行：KIROKU_E2E_*=... cargo test e2e_sync_roundtrip -- --ignored
    #[tokio::test]
    #[ignore = "依赖真实 dev 项目与测试凭据"]
    async fn e2e_sync_roundtrip() {
        let Ok(email) = std::env::var("KIROKU_E2E_EMAIL") else {
            return;
        };
        let password = std::env::var("KIROKU_E2E_PASSWORD").expect("KIROKU_E2E_PASSWORD");
        let code = std::env::var("KIROKU_E2E_CODE").expect("KIROKU_E2E_CODE");

        let dir = tempfile::tempdir().unwrap();
        let conn = db::open(&dir.path().join("kiroku.db")).unwrap();
        let live = LiveState {
            db: Mutex::new(conn),
            data_dir: dir.path().to_path_buf(),
            bangumi: BangumiClient::new().unwrap(),
            http: reqwest::Client::new(),
            session: Mutex::new(None),
            sync_notify: Arc::new(tokio::sync::Notify::new()),
        };

        // 1. 离线本地写入：同步未开启，不产生 outbox
        {
            let mut conn = live.db.lock().unwrap();
            db::add_library_entry(&mut conn, &subject(991122), &draft(8.0)).unwrap();
            let n: i64 = conn
                .query_row("SELECT COUNT(*) FROM sync_outbox", [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0);
        }

        // 2. 登录 + 绑定 + 兑换
        let session = auth::login(&live.http, &live.data_dir, &email, &password)
            .await
            .expect("login");
        bind_account(&live, &session.user_id).unwrap();
        *live.session.lock().unwrap() = Some(session);
        let redeemed = redeem(&live, &code).await.expect("redeem");
        assert!(
            matches!(
                redeemed.get("status").and_then(|v| v.as_str()),
                Some("redeemed" | "already_redeemed" | "replayed")
            ),
            "redeem 返回异常：{redeemed}"
        );

        // 3. 首次开启同步：云端为空 → 本地全量入队（base=0），无冲突
        let conflicts = initial_enable(&live).await.expect("initial_enable");
        assert_eq!(conflicts, 0);
        {
            let conn = live.db.lock().unwrap();
            let enabled: i64 = conn
                .query_row(
                    "SELECT sync_enabled FROM sync_state WHERE id = 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(enabled, 1);
        }

        // 4. 一轮同步：本地记录推上去（含分档与排序）
        run_once(&live).await.expect("run_once");
        {
            let conn = live.db.lock().unwrap();
            let pending: i64 = conn
                .query_row("SELECT COUNT(*) FROM sync_outbox", [], |r| r.get(0))
                .unwrap();
            let remote_version: i64 = conn
                .query_row(
                    "SELECT p.server_version FROM personal_records p
                     JOIN subjects s ON s.id = p.subject_id WHERE s.bangumi_subject_id = 991122",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(pending, 0, "推送后 outbox 应为空");
            assert!(remote_version > 0, "远端版本应回写");
        }

        // 5. 再用 cursor=0 快照验证云端确有该记录
        let snap = rpc(&live, "kiroku_sync_pull", json!({ "p_cursor": 0 }))
            .await
            .expect("pull snapshot");
        let found = snap
            .get("items")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .any(|i| i.get("entity_key").and_then(|k| k.as_str()) == Some("991122"))
            })
            .unwrap_or(false);
        assert!(found, "快照应包含刚推送的记录");

        // 6. 清库：确认串校验 + epoch 递增进入对账态
        let denied = delete_cloud(&live, "wrong").await;
        assert!(denied.is_err());
        delete_cloud(&live, "DELETE").await.expect("delete_cloud");
        {
            let conn = live.db.lock().unwrap();
            let state = read_state(&conn).unwrap();
            assert!(!state.sync_enabled && state.reconcile_required);
        }
    }
}
