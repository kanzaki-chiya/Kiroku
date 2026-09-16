use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::error::AppError;
use crate::models::{
    CommunityDto, DimensionsDto, LibraryEntryDto, PersonalDraftDto, PersonalRecordDto, SubjectDto,
    TierDto,
};
use crate::validate::{dimension_to_tenths, now_iso, score_to_tenths, tenths_to_score};

const MIGRATION_V1: &str = r#"
CREATE TABLE subjects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bangumi_subject_id INTEGER NOT NULL UNIQUE,
  name TEXT NOT NULL,
  name_cn TEXT NOT NULL,
  aliases_json TEXT NOT NULL DEFAULT '[]',
  summary TEXT NOT NULL DEFAULT '',
  cover_url TEXT NOT NULL DEFAULT '',
  cover_local_path TEXT,
  year INTEGER NOT NULL DEFAULT 0,
  format TEXT NOT NULL,
  episodes INTEGER NOT NULL DEFAULT 0,
  studio TEXT NOT NULL DEFAULT '',
  tags_json TEXT NOT NULL DEFAULT '[]',
  metadata_fetched_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE community_snapshots (
  subject_id INTEGER PRIMARY KEY REFERENCES subjects(id) ON DELETE CASCADE,
  score INTEGER,
  votes INTEGER NOT NULL DEFAULT 0,
  rank INTEGER,
  fetched_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'not_fetched',
  CHECK (score IS NULL OR (score >= 0 AND score <= 100))
);

CREATE TABLE tiers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL DEFAULT '#335d4e',
  sort_order INTEGER NOT NULL,
  is_builtin INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE personal_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  subject_id INTEGER NOT NULL UNIQUE REFERENCES subjects(id) ON DELETE CASCADE,
  score INTEGER,
  tier_id INTEGER REFERENCES tiers(id),
  status TEXT NOT NULL CHECK (status IN ('completed', 'watching', 'planned')),
  review TEXT NOT NULL DEFAULT '' CHECK (length(review) <= 5000),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  CHECK (score IS NULL OR (score >= 0 AND score <= 100))
);

CREATE TABLE dimension_scores (
  record_id INTEGER NOT NULL REFERENCES personal_records(id) ON DELETE CASCADE,
  dimension TEXT NOT NULL CHECK (dimension IN ('story', 'characters', 'direction', 'animation', 'music')),
  score INTEGER,
  PRIMARY KEY (record_id, dimension),
  CHECK (score IS NULL OR (score >= 0 AND score <= 100))
);

CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
"#;

const DEFAULT_TIERS: [(&str, &str, &str, i64); 5] = [
    ("S", "私心珍藏", "#bd592e", 0),
    ("A", "非常推荐", "#335d4e", 1),
    ("B", "值得一看", "#5b7c6e", 2),
    ("C", "略有保留", "#8a7a4d", 3),
    ("D", "不太对味", "#8b6b63", 4),
];

// v4：云同步（SYNC_DESIGN.md §3）。只增不改：新列 + 三张新表。
const MIGRATION_V4: &str = r#"
ALTER TABLE tiers ADD COLUMN sync_id TEXT;
ALTER TABLE tiers ADD COLUMN server_version INTEGER NOT NULL DEFAULT 0;
ALTER TABLE personal_records ADD COLUMN server_version INTEGER NOT NULL DEFAULT 0;

CREATE TABLE sync_outbox (
  op_id TEXT PRIMARY KEY,
  seq INTEGER NOT NULL,
  entity_type TEXT NOT NULL,
  entity_key TEXT NOT NULL,
  op_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  base_server_version INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT
);
CREATE INDEX sync_outbox_seq ON sync_outbox(seq);

CREATE TABLE sync_conflicts (
  entity_type TEXT NOT NULL,
  entity_key TEXT NOT NULL,
  local_payload_json TEXT,
  remote_payload_json TEXT,
  remote_server_version INTEGER,
  detected_at TEXT NOT NULL,
  PRIMARY KEY (entity_type, entity_key)
);

CREATE TABLE sync_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  account_id TEXT,
  sync_enabled INTEGER NOT NULL DEFAULT 0,
  device_id TEXT NOT NULL,
  epoch INTEGER NOT NULL DEFAULT 0,
  cursor INTEGER NOT NULL DEFAULT 0,
  session_generation INTEGER NOT NULL DEFAULT 0,
  outbox_seq INTEGER NOT NULL DEFAULT 0,
  tier_order_server_version INTEGER NOT NULL DEFAULT 0,
  reconcile_required INTEGER NOT NULL DEFAULT 0,
  last_sync_at TEXT,
  last_error TEXT
);
"#;

// v5：outbox 逐 op 退避（SYNC_DESIGN §9 指数退避，上限 5 分钟）。
const MIGRATION_V5: &str = r#"
ALTER TABLE sync_outbox ADD COLUMN next_retry_at TEXT;
"#;

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)
        .map_err(|err| AppError::startup(format!("无法打开数据库：{err}")))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    migrate(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;
    let mut current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    if current < 1 {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(MIGRATION_V1)?;
        seed_tiers(&tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
            params![now_iso()],
        )?;
        tx.commit()?;
        current = 1;
    }
    if current < 2 {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE dimension_scores SET score = MAX(5, CAST(ROUND(score / 10.0) AS INTEGER) * 5)",
            [],
        )?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (2, ?1)",
            params![now_iso()],
        )?;
        tx.commit()?;
    }
    if current < 3 {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "ALTER TABLE personal_records ADD COLUMN progress INTEGER",
            [],
        )?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (3, ?1)",
            params![now_iso()],
        )?;
        tx.commit()?;
    }
    if current < 4 {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(MIGRATION_V4)?;
        backfill_tier_sync_ids(&tx)?;
        tx.execute(
            "INSERT INTO sync_state (id, device_id) VALUES (1, ?1)",
            params![uuid::Uuid::new_v4().to_string()],
        )?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (4, ?1)",
            params![now_iso()],
        )?;
        tx.commit()?;
    }
    if current < 5 {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(MIGRATION_V5)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (5, ?1)",
            params![now_iso()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

// 内置分档按当前名称映射 builtin:<name>——用户在旧版可改内置名，名称比位置更可靠；
// 改名不一致的多端会在首次合并时作为不同分档进冲突，而不是静默错配。
fn backfill_tier_sync_ids(tx: &Transaction<'_>) -> Result<(), AppError> {
    let mut stmt = tx.prepare("SELECT id, name, is_builtin FROM tiers")?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, name, is_builtin) in rows {
        let sync_id = if is_builtin == 1 {
            format!("builtin:{name}")
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        tx.execute(
            "UPDATE tiers SET sync_id = ?1 WHERE id = ?2",
            params![sync_id, id],
        )?;
    }
    tx.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS tiers_sync_id_unique ON tiers(sync_id)",
        [],
    )?;
    Ok(())
}

fn seed_tiers(tx: &Transaction<'_>) -> Result<(), AppError> {
    for (name, description, color, order) in DEFAULT_TIERS {
        tx.execute(
            "INSERT INTO tiers (name, description, color, sort_order, is_builtin)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![name, description, color, order],
        )?;
    }
    Ok(())
}

pub fn list_tier_names(conn: &Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare("SELECT name FROM tiers ORDER BY sort_order, id")?;
    let names = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(names)
}

pub fn list_tiers(conn: &Connection) -> Result<Vec<TierDto>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, sort_order, is_builtin FROM tiers ORDER BY sort_order, id",
    )?;
    let tiers = stmt
        .query_map([], |row| {
            Ok(TierDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                sort_order: row.get(4)?,
                builtin: row.get::<_, i64>(5)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tiers)
}

fn tier_id_by_name(conn: &Connection, name: Option<&str>) -> Result<Option<i64>, AppError> {
    let Some(name) = name else {
        return Ok(None);
    };
    conn.query_row(
        "SELECT id FROM tiers WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::validation("无效的分档"))
    .map(Some)
}

pub fn save_tier(
    conn: &Connection,
    payload: crate::models::SaveTierPayload,
) -> Result<TierDto, AppError> {
    let name = payload.name.trim();
    if name.is_empty() || name.len() > 20 {
        return Err(AppError::validation("分档名称不能为空且不超过 20 字"));
    }
    if matches!(name, "all" | "unassigned") {
        return Err(AppError::validation("该名称为筛选保留字"));
    }
    if payload.color.trim().is_empty() {
        return Err(AppError::validation("分档颜色不能为空"));
    }
    let clash: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tiers WHERE name = ?1 AND id != ?2",
        params![name, payload.id.unwrap_or(0)],
        |row| row.get(0),
    )?;
    if clash > 0 {
        return Err(AppError::duplicate("分档名称已存在"));
    }
    if let Some(id) = payload.id {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tiers WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(AppError::not_found("分档不存在"));
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE tiers SET name = ?1, description = ?2, color = ?3 WHERE id = ?4",
            params![name, payload.description, payload.color, id],
        )?;
        let (sync_payload, sync_id, base_version) = tier_sync_payload(&tx, id)?;
        enqueue_sync_op(&tx, "tier", &sync_id, "upsert", &sync_payload, base_version)?;
        tx.commit()?;
        get_tier(conn, id)
    } else {
        let next_order: i64 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM tiers",
            [],
            |row| row.get(0),
        )?;
        let tx = conn.unchecked_transaction()?;
        let sync_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO tiers (name, description, color, sort_order, is_builtin, sync_id) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![name, payload.description, payload.color, next_order, sync_id],
        )?;
        let id = tx.last_insert_rowid();
        let (sync_payload, _, base_version) = tier_sync_payload(&tx, id)?;
        enqueue_sync_op(&tx, "tier", &sync_id, "upsert", &sync_payload, base_version)?;
        tx.commit()?;
        get_tier(conn, id)
    }
}

fn get_tier(conn: &Connection, id: i64) -> Result<TierDto, AppError> {
    conn.query_row(
        "SELECT id, name, description, color, sort_order, is_builtin FROM tiers WHERE id = ?1",
        params![id],
        |row| {
            Ok(TierDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                sort_order: row.get(4)?,
                builtin: row.get::<_, i64>(5)? == 1,
            })
        },
    )
    .map_err(|_| AppError::not_found("分档不存在"))
}

pub fn reorder_tiers(conn: &Connection, ids: Vec<i64>) -> Result<Vec<TierDto>, AppError> {
    let existing = list_tiers(conn)?;
    if ids.len() != existing.len() {
        return Err(AppError::validation("分档排序不完整"));
    }
    let tx = conn.unchecked_transaction()?;
    for (index, id) in ids.iter().enumerate() {
        let changed = tx.execute(
            "UPDATE tiers SET sort_order = ?1 WHERE id = ?2",
            params![index as i64, id],
        )?;
        if changed == 0 {
            return Err(AppError::not_found("分档不存在"));
        }
    }
    let (order_payload, order_base) = tier_order_payload(&tx)?;
    enqueue_sync_op(
        &tx,
        "tier_order",
        "tier_order",
        "set",
        &order_payload,
        order_base,
    )?;
    tx.commit()?;
    list_tiers(conn)
}

pub fn delete_tier(conn: &Connection, id: i64) -> Result<(), AppError> {
    let referenced: i64 = conn.query_row(
        "SELECT COUNT(*) FROM personal_records WHERE tier_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if referenced > 0 {
        return Err(AppError::conflict("该分档仍被收藏引用，请先迁移或取消关联"));
    }
    let meta = conn
        .query_row(
            "SELECT is_builtin, sync_id, server_version FROM tiers WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((builtin, sync_id, server_version)) = meta else {
        return Err(AppError::not_found("分档不存在"));
    };
    if builtin == 1 {
        return Err(AppError::conflict("内置分档不能删除"));
    }
    let Some(sync_id) = sync_id else {
        return Err(AppError::internal("分档缺少同步标识"));
    };
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM tiers WHERE id = ?1", params![id])?;
    enqueue_sync_op(
        &tx,
        "tier",
        &sync_id,
        "delete",
        &serde_json::Value::Null,
        server_version,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn upsert_subject(
    tx: &Transaction<'_>,
    subject: &SubjectDto,
    fetched_at: &str,
) -> Result<i64, AppError> {
    let aliases = serde_json::to_string(subject.aliases.as_ref().unwrap_or(&Vec::new()))?;
    let tags = serde_json::to_string(&subject.tags)?;
    tx.execute(
        "INSERT INTO subjects (
            bangumi_subject_id, name, name_cn, aliases_json, summary, cover_url, cover_local_path,
            year, format, episodes, studio, tags_json, metadata_fetched_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13, ?13)
         ON CONFLICT(bangumi_subject_id) DO UPDATE SET
            name = excluded.name,
            name_cn = excluded.name_cn,
            aliases_json = excluded.aliases_json,
            summary = excluded.summary,
            cover_url = excluded.cover_url,
            year = excluded.year,
            format = excluded.format,
            episodes = excluded.episodes,
            studio = excluded.studio,
            tags_json = excluded.tags_json,
            metadata_fetched_at = excluded.metadata_fetched_at,
            updated_at = excluded.updated_at",
        params![
            subject.id,
            subject.name,
            subject.name_cn,
            aliases,
            subject.summary,
            subject.cover_url,
            subject.cover_local_path,
            subject.year,
            subject.format,
            subject.episodes,
            subject.studio,
            tags,
            fetched_at
        ],
    )?;
    let local_id: i64 = tx.query_row(
        "SELECT id FROM subjects WHERE bangumi_subject_id = ?1",
        params![subject.id],
        |row| row.get(0),
    )?;
    let comm_score = subject
        .community
        .score
        .map(|value| score_to_tenths(value, "社区评分"))
        .transpose()?;
    tx.execute(
        "INSERT INTO community_snapshots (subject_id, score, votes, rank, fetched_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(subject_id) DO UPDATE SET
            score = excluded.score,
            votes = excluded.votes,
            rank = excluded.rank,
            fetched_at = excluded.fetched_at,
            status = excluded.status",
        params![
            local_id,
            comm_score,
            subject.community.votes,
            subject.community.rank,
            subject
                .community
                .fetched_at
                .clone()
                .unwrap_or_else(|| fetched_at.to_string()),
            subject
                .community
                .status
                .clone()
                .unwrap_or_else(|| "ok".into())
        ],
    )?;
    Ok(local_id)
}

fn insert_dimensions(
    tx: &Transaction<'_>,
    record_id: i64,
    draft: &PersonalDraftDto,
) -> Result<(), AppError> {
    for (dimension, value) in draft.dimensions.values() {
        let tenths = value.map(dimension_to_tenths).transpose()?;
        tx.execute(
            "INSERT INTO dimension_scores (record_id, dimension, score) VALUES (?1, ?2, ?3)
             ON CONFLICT(record_id, dimension) DO UPDATE SET score = excluded.score",
            params![record_id, dimension, tenths],
        )?;
    }
    Ok(())
}

pub fn add_library_entry(
    conn: &mut Connection,
    subject: &SubjectDto,
    draft: &PersonalDraftDto,
) -> Result<LibraryEntryDto, AppError> {
    let names = list_tier_names(conn)?;
    crate::validate::validate_draft(draft, &names)?;
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM subjects s JOIN personal_records p ON p.subject_id = s.id WHERE s.bangumi_subject_id = ?1",
        params![subject.id],
        |row| row.get(0),
    )?;
    if exists > 0 {
        return Err(AppError::duplicate("这部作品已经在你的番剧库中"));
    }
    let now = now_iso();
    let tx = conn.transaction()?;
    let local_id = upsert_subject(&tx, subject, &now)?;
    let tier_id = tier_id_by_name(&tx, draft.tier.as_deref())?;
    let score = draft
        .score
        .map(|value| score_to_tenths(value, "个人评分"))
        .transpose()?;
    tx.execute(
        "INSERT INTO personal_records (subject_id, score, tier_id, status, review, progress, created_at, updated_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, 1)",
        params![local_id, score, tier_id, draft.status, draft.review, draft.progress, now],
    )?;
    let record_id = tx.last_insert_rowid();
    insert_dimensions(&tx, record_id, draft)?;
    let (sync_payload, base_version) = record_sync_payload(&tx, local_id)?;
    enqueue_sync_op(
        &tx,
        "record",
        &subject.id.to_string(),
        "upsert",
        &sync_payload,
        base_version,
    )?;
    tx.commit()?;
    get_library_entry_by_bangumi_id(conn, subject.id)?
        .ok_or_else(|| AppError::internal("写入后无法读取收藏"))
}

pub fn remove_library_entry(
    conn: &Connection,
    bangumi_subject_id: i64,
) -> Result<Option<String>, AppError> {
    let row = conn
        .query_row(
            "SELECT s.id, s.cover_local_path, p.server_version
             FROM subjects s
             JOIN personal_records p ON p.subject_id = s.id
             WHERE s.bangumi_subject_id = ?1",
            params![bangumi_subject_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((local_id, cover_path, server_version)) = row else {
        return Err(AppError::not_found("这部作品还没有收录"));
    };
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM subjects WHERE id = ?1", params![local_id])?;
    enqueue_sync_op(
        &tx,
        "record",
        &bangumi_subject_id.to_string(),
        "delete",
        &serde_json::Value::Null,
        server_version,
    )?;
    tx.commit()?;
    Ok(cover_path)
}

pub fn update_personal_record(
    conn: &mut Connection,
    bangumi_subject_id: i64,
    draft: &PersonalDraftDto,
    version: i64,
) -> Result<LibraryEntryDto, AppError> {
    let names = list_tier_names(conn)?;
    crate::validate::validate_draft(draft, &names)?;
    let row = conn
        .query_row(
            "SELECT p.id, p.version, p.created_at, s.id
             FROM personal_records p
             JOIN subjects s ON s.id = p.subject_id
             WHERE s.bangumi_subject_id = ?1",
            params![bangumi_subject_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((record_id, current_version, _created_at, subject_local_id)) = row else {
        return Err(AppError::not_found("这部作品还没有收录"));
    };
    if current_version != version {
        return Err(AppError::conflict("记录已被其他窗口修改，请刷新后重试"));
    }
    let now = now_iso();
    let tx = conn.transaction()?;
    let tier_id = tier_id_by_name(&tx, draft.tier.as_deref())?;
    let score = draft
        .score
        .map(|value| score_to_tenths(value, "个人评分"))
        .transpose()?;
    let changed = tx.execute(
        "UPDATE personal_records
         SET score = ?1, tier_id = ?2, status = ?3, review = ?4, progress = ?5, updated_at = ?6, version = version + 1
         WHERE id = ?7 AND version = ?8",
        params![score, tier_id, draft.status, draft.review, draft.progress, now, record_id, version],
    )?;
    if changed == 0 {
        return Err(AppError::conflict("记录已被其他窗口修改，请刷新后重试"));
    }
    insert_dimensions(&tx, record_id, draft)?;
    let (sync_payload, base_version) = record_sync_payload(&tx, subject_local_id)?;
    enqueue_sync_op(
        &tx,
        "record",
        &bangumi_subject_id.to_string(),
        "upsert",
        &sync_payload,
        base_version,
    )?;
    tx.commit()?;
    get_library_entry_by_bangumi_id(conn, bangumi_subject_id)?
        .ok_or_else(|| AppError::internal("更新后无法读取收藏"))
}

pub fn import_entries(
    conn: &mut Connection,
    entries: &[crate::models::BackupEntry],
    overwrite: bool,
) -> Result<(), AppError> {
    let names = list_tier_names(conn)?;
    let tx = conn.transaction()?;
    for item in entries {
        let draft = PersonalDraftDto {
            score: item.personal.score,
            tier: item.personal.tier.clone(),
            status: item.personal.status.clone(),
            progress: item.personal.progress,
            dimensions: item.personal.dimensions.clone(),
            review: item.personal.review.clone(),
        };
        crate::validate::validate_draft(&draft, &names)?;
        let existing: Option<(i64, i64, i64)> = tx
            .query_row(
                "SELECT p.id, p.version, s.id
                 FROM personal_records p
                 JOIN subjects s ON s.id = p.subject_id
                 WHERE s.bangumi_subject_id = ?1",
                params![item.bangumi_subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let now = now_iso();
        if let Some((record_id, version, subject_local_id)) = existing {
            if !overwrite {
                continue;
            }
            upsert_subject(&tx, &item.subject, &now)?;
            let tier_id = tier_id_by_name(&tx, draft.tier.as_deref())?;
            let score = draft
                .score
                .map(|value| score_to_tenths(value, "个人评分"))
                .transpose()?;
            let changed = tx.execute(
                "UPDATE personal_records
                 SET score = ?1, tier_id = ?2, status = ?3, review = ?4, progress = ?5, updated_at = ?6, version = version + 1
                 WHERE id = ?7 AND version = ?8",
                params![score, tier_id, draft.status, draft.review, draft.progress, now, record_id, version],
            )?;
            if changed == 0 {
                return Err(AppError::conflict("导入时记录版本冲突"));
            }
            insert_dimensions(&tx, record_id, &draft)?;
            let (sync_payload, base_version) = record_sync_payload(&tx, subject_local_id)?;
            enqueue_sync_op(
                &tx,
                "record",
                &item.bangumi_subject_id.to_string(),
                "upsert",
                &sync_payload,
                base_version,
            )?;
        } else {
            let local_id = upsert_subject(&tx, &item.subject, &now)?;
            let tier_id = tier_id_by_name(&tx, draft.tier.as_deref())?;
            let score = draft
                .score
                .map(|value| score_to_tenths(value, "个人评分"))
                .transpose()?;
            tx.execute(
                "INSERT INTO personal_records (subject_id, score, tier_id, status, review, progress, created_at, updated_at, version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, 1)",
                params![local_id, score, tier_id, draft.status, draft.review, draft.progress, now],
            )?;
            let record_id = tx.last_insert_rowid();
            insert_dimensions(&tx, record_id, &draft)?;
            let (sync_payload, base_version) = record_sync_payload(&tx, local_id)?;
            enqueue_sync_op(
                &tx,
                "record",
                &item.bangumi_subject_id.to_string(),
                "upsert",
                &sync_payload,
                base_version,
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn refresh_subject_metadata(
    conn: &mut Connection,
    subject: &SubjectDto,
) -> Result<LibraryEntryDto, AppError> {
    let now = now_iso();
    let tx = conn.transaction()?;
    let exists: i64 = tx.query_row(
        "SELECT COUNT(*) FROM subjects s JOIN personal_records p ON p.subject_id = s.id WHERE s.bangumi_subject_id = ?1",
        params![subject.id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::not_found("这部作品还没有收录"));
    }
    upsert_subject(&tx, subject, &now)?;
    tx.commit()?;
    get_library_entry_by_bangumi_id(conn, subject.id)?
        .ok_or_else(|| AppError::internal("刷新后无法读取收藏"))
}

pub fn set_cover_path(
    conn: &Connection,
    bangumi_subject_id: i64,
    relative_path: &str,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE subjects SET cover_local_path = ?1 WHERE bangumi_subject_id = ?2",
        params![relative_path, bangumi_subject_id],
    )?;
    Ok(())
}

pub fn list_library_entries(conn: &Connection) -> Result<Vec<LibraryEntryDto>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT
            s.id, s.bangumi_subject_id, s.name, s.name_cn, s.aliases_json, s.summary, s.cover_url,
            s.cover_local_path, s.year, s.format, s.episodes, s.studio, s.tags_json,
            c.score, c.votes, c.rank, c.fetched_at, c.status,
            p.id, p.score, p.status, p.review, p.progress, p.created_at, p.updated_at, p.version, t.name
         FROM personal_records p
         JOIN subjects s ON s.id = p.subject_id
         LEFT JOIN community_snapshots c ON c.subject_id = s.id
         LEFT JOIN tiers t ON t.id = p.tier_id
         ORDER BY p.updated_at DESC, s.bangumi_subject_id ASC",
    )?;
    let mut rows = stmt.query([])?;
    let mut entries = Vec::new();
    let mut record_ids = Vec::new();
    while let Some(row) = rows.next()? {
        let record_id: i64 = row.get(18)?;
        record_ids.push(record_id);
        let aliases: String = row.get(4)?;
        let tags: String = row.get(12)?;
        let comm_score: Option<i64> = row.get(13)?;
        let personal_score: Option<i64> = row.get(19)?;
        entries.push(LibraryEntryDto {
            local_id: row.get(0)?,
            subject: SubjectDto {
                id: row.get(1)?,
                name: row.get(2)?,
                name_cn: row.get(3)?,
                aliases: serde_json::from_str(&aliases).ok(),
                summary: row.get(5)?,
                cover_url: row.get(6)?,
                cover_local_path: row.get(7)?,
                year: row.get(8)?,
                format: row.get(9)?,
                episodes: row.get(10)?,
                studio: row.get(11)?,
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                community: CommunityDto {
                    score: comm_score.map(tenths_to_score),
                    votes: row.get(14)?,
                    rank: row.get(15)?,
                    fetched_at: row.get(16)?,
                    status: row.get(17)?,
                },
            },
            personal: PersonalRecordDto {
                score: personal_score.map(tenths_to_score),
                tier: row.get(26)?,
                status: row.get(20)?,
                progress: row.get(22)?,
                dimensions: DimensionsDto {
                    story: None,
                    characters: None,
                    direction: None,
                    animation: None,
                    music: None,
                },
                review: row.get(21)?,
                subject_id: row.get(1)?,
                created_at: row.get(23)?,
                updated_at: row.get(24)?,
                version: row.get(25)?,
            },
        });
    }
    let dimensions = load_dimensions(conn, &record_ids)?;
    for (entry, record_id) in entries.iter_mut().zip(record_ids.iter()) {
        if let Some(map) = dimensions.get(record_id) {
            entry.personal.dimensions = DimensionsDto::from_map(map.clone());
        }
    }
    Ok(entries)
}

fn load_dimensions(
    conn: &Connection,
    record_ids: &[i64],
) -> Result<HashMap<i64, HashMap<String, Option<f64>>>, AppError> {
    let mut out: HashMap<i64, HashMap<String, Option<f64>>> = HashMap::new();
    if record_ids.is_empty() {
        return Ok(out);
    }
    let mut stmt = conn.prepare("SELECT record_id, dimension, score FROM dimension_scores")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<i64>>(2)?,
        ))
    })?;
    for row in rows {
        let (record_id, dimension, score) = row?;
        if !record_ids.contains(&record_id) {
            continue;
        }
        out.entry(record_id)
            .or_default()
            .insert(dimension, score.map(tenths_to_score));
    }
    Ok(out)
}

pub fn get_library_entry_by_bangumi_id(
    conn: &Connection,
    bangumi_subject_id: i64,
) -> Result<Option<LibraryEntryDto>, AppError> {
    Ok(list_library_entries(conn)?
        .into_iter()
        .find(|entry| entry.subject.id == bangumi_subject_id))
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(AppError::from)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn delete_personal_data(conn: &mut Connection) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    tx.execute_batch(
        "DELETE FROM dimension_scores;
         DELETE FROM personal_records;
         DELETE FROM community_snapshots;
         DELETE FROM subjects;
         DELETE FROM sync_outbox;
         DELETE FROM sync_conflicts;
         UPDATE sync_state SET
            account_id = NULL, sync_enabled = 0, epoch = 0, cursor = 0,
            outbox_seq = 0, tier_order_server_version = 0,
            reconcile_required = 0, last_sync_at = NULL, last_error = NULL;",
    )?;
    tx.commit()?;
    Ok(())
}

// ---------- 云同步：outbox 与远端应用（SYNC_DESIGN.md §5/§6） ----------

pub(crate) fn sync_is_enabled(conn: &Connection) -> Result<bool, AppError> {
    let enabled: Option<i64> = conn
        .query_row(
            "SELECT sync_enabled FROM sync_state WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(enabled == Some(1))
}

/// 业务写事务内调用；sync_enabled=0 时静默跳过（首次开启走全量合并，不靠存量 outbox）。
pub(crate) fn enqueue_sync_op(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
    op_type: &str,
    payload: &serde_json::Value,
    base_server_version: i64,
) -> Result<(), AppError> {
    if !sync_is_enabled(conn)? {
        return Ok(());
    }
    enqueue_sync_op_force(
        conn,
        entity_type,
        entity_key,
        op_type,
        payload,
        base_server_version,
    )
}

/// 不检查开关——首次合并/冲突解决时直接入队。
pub(crate) fn enqueue_sync_op_force(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
    op_type: &str,
    payload: &serde_json::Value,
    base_server_version: i64,
) -> Result<(), AppError> {
    let seq: i64 = conn.query_row(
        "UPDATE sync_state SET outbox_seq = outbox_seq + 1 WHERE id = 1 RETURNING outbox_seq",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO sync_outbox (op_id, seq, entity_type, entity_key, op_type, payload_json, base_server_version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            uuid::Uuid::new_v4().to_string(),
            seq,
            entity_type,
            entity_key,
            op_type,
            serde_json::to_string(payload)?,
            base_server_version,
            now_iso()
        ],
    )?;
    Ok(())
}

/// 同实体已有排队 op 时先清掉再入队（合并/冲突解决用，折叠为最新全量态）。
pub(crate) fn enqueue_sync_op_replace(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
    op_type: &str,
    payload: &serde_json::Value,
    base_server_version: i64,
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM sync_outbox WHERE entity_type = ?1 AND entity_key = ?2",
        params![entity_type, entity_key],
    )?;
    enqueue_sync_op_force(
        conn,
        entity_type,
        entity_key,
        op_type,
        payload,
        base_server_version,
    )
}

pub(crate) fn entity_has_pending_op(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_outbox WHERE entity_type = ?1 AND entity_key = ?2",
        params![entity_type, entity_key],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// 组装 record 上行载荷（全量态）。返回 (payload, 当前 server_version)。
pub(crate) fn record_sync_payload(
    conn: &Connection,
    local_subject_id: i64,
) -> Result<(serde_json::Value, i64), AppError> {
    let (record_id, score, tier_sync_id, status, review, progress, created_at, updated_at, server_version) =
        conn.query_row(
            "SELECT p.id, p.score, t.sync_id, p.status, p.review, p.progress, p.created_at, p.updated_at, p.server_version
             FROM personal_records p
             LEFT JOIN tiers t ON t.id = p.tier_id
             WHERE p.subject_id = ?1",
            params![local_subject_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )?;
    let mut stmt =
        conn.prepare("SELECT dimension, score FROM dimension_scores WHERE record_id = ?1")?;
    let dims = stmt
        .query_map(params![record_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            ))
        })?
        .collect::<Result<serde_json::Map<String, serde_json::Value>, _>>()?;
    let snapshot = conn.query_row(
        "SELECT name, name_cn, aliases_json, summary, cover_url, year, format, episodes, studio, tags_json
         FROM subjects WHERE id = ?1",
        params![local_subject_id],
        |row| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "name_cn": row.get::<_, String>(1)?,
                "aliases": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(2)?).unwrap_or(serde_json::Value::Null),
                "summary": row.get::<_, String>(3)?,
                "cover_url": row.get::<_, String>(4)?,
                "year": row.get::<_, i64>(5)?,
                "format": row.get::<_, String>(6)?,
                "episodes": row.get::<_, i64>(7)?,
                "studio": row.get::<_, String>(8)?,
                "tags": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(9)?).unwrap_or(serde_json::Value::Null),
            }))
        },
    )?;
    Ok((
        serde_json::json!({
            "record": {
                "score": score,
                "status": status,
                "progress": progress,
                "review": review,
                "tier_sync_id": tier_sync_id,
                "dimensions": dims,
                "created_at": created_at,
                "updated_at": updated_at,
            },
            "snapshot": snapshot,
        }),
        server_version,
    ))
}

/// 组装 tier 上行载荷。返回 (payload, server_version)。
pub(crate) fn tier_sync_payload(
    conn: &Connection,
    tier_id: i64,
) -> Result<(serde_json::Value, String, i64), AppError> {
    conn.query_row(
        "SELECT name, description, color, is_builtin, sync_id, server_version FROM tiers WHERE id = ?1",
        params![tier_id],
        |row| {
            Ok((
                serde_json::json!({
                    "name": row.get::<_, String>(0)?,
                    "description": row.get::<_, String>(1)?,
                    "color": row.get::<_, String>(2)?,
                    "builtin": row.get::<_, i64>(3)? == 1,
                }),
                row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                row.get::<_, i64>(5)?,
            ))
        },
    )
    .map_err(AppError::from)
}

/// 组装 tier_order 上行载荷。返回 (payload, server_version)。
pub(crate) fn tier_order_payload(conn: &Connection) -> Result<(serde_json::Value, i64), AppError> {
    let mut stmt = conn.prepare("SELECT sync_id FROM tiers ORDER BY sort_order, id")?;
    let keys = stmt
        .query_map([], |row| row.get::<_, Option<String>>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<String>>();
    let version: i64 = conn.query_row(
        "SELECT tier_order_server_version FROM sync_state WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    Ok((serde_json::json!({ "ordered_keys": keys }), version))
}

/// 写/更新冲突记录（同一实体重复冲突覆盖旧记录）。
pub(crate) fn upsert_conflict(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
    local_payload: Option<&serde_json::Value>,
    remote_payload: Option<&serde_json::Value>,
    remote_server_version: Option<i64>,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO sync_conflicts (entity_type, entity_key, local_payload_json, remote_payload_json, remote_server_version, detected_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(entity_type, entity_key) DO UPDATE SET
            local_payload_json = excluded.local_payload_json,
            remote_payload_json = excluded.remote_payload_json,
            remote_server_version = excluded.remote_server_version,
            detected_at = excluded.detected_at",
        params![
            entity_type,
            entity_key,
            local_payload.map(serde_json::to_string).transpose()?,
            remote_payload.map(serde_json::to_string).transpose()?,
            remote_server_version,
            now_iso()
        ],
    )?;
    Ok(())
}

/// 远端 record.upsert 落地。本地有待传 op → 返回 false（由调用方记冲突）。
/// 返回 Ok(true)=已应用。
pub(crate) fn apply_remote_record_upsert(
    conn: &Connection,
    bangumi_subject_id: i64,
    record: &serde_json::Value,
    snapshot: Option<&serde_json::Value>,
    server_version: i64,
) -> Result<bool, AppError> {
    if entity_has_pending_op(conn, "record", &bangumi_subject_id.to_string())? {
        return Ok(false);
    }
    if let Some(snap) = snapshot {
        upsert_subject_from_snapshot(conn, bangumi_subject_id, snap)?;
    }
    let local_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM subjects WHERE bangumi_subject_id = ?1",
            params![bangumi_subject_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(local_id) = local_id else {
        return Err(AppError::internal("远端记录缺少可用资料快照"));
    };
    let tier_sync_id = record.get("tier_sync_id").and_then(|v| v.as_str());
    let tier_id: Option<i64> = match tier_sync_id {
        Some(key) => conn
            .query_row(
                "SELECT id FROM tiers WHERE sync_id = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?,
        None => None,
    };
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM personal_records WHERE subject_id = ?1",
            params![local_id],
            |row| row.get(0),
        )
        .optional()?;
    let score = json_int(record.get("score"));
    let progress = json_int(record.get("progress"));
    let status = record
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("planned");
    let review = record.get("review").and_then(|v| v.as_str()).unwrap_or("");
    let fallback_now = now_iso();
    let created_at = record
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or(&fallback_now);
    let updated_at = record
        .get("updated_at")
        .and_then(|v| v.as_str())
        .unwrap_or(&fallback_now);
    let record_id = match exists {
        Some(id) => {
            conn.execute(
                "UPDATE personal_records
                 SET score = ?1, tier_id = ?2, status = ?3, review = ?4, progress = ?5,
                     created_at = ?6, updated_at = ?7, version = version + 1, server_version = ?8
                 WHERE id = ?9",
                params![
                    score,
                    tier_id,
                    status,
                    review,
                    progress,
                    created_at,
                    updated_at,
                    server_version,
                    id
                ],
            )?;
            id
        }
        None => {
            conn.execute(
                "INSERT INTO personal_records
                    (subject_id, score, tier_id, status, review, progress, created_at, updated_at, version, server_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
                params![local_id, score, tier_id, status, review, progress, created_at, updated_at, server_version],
            )?;
            conn.last_insert_rowid()
        }
    };
    conn.execute(
        "DELETE FROM dimension_scores WHERE record_id = ?1",
        params![record_id],
    )?;
    if let Some(dims) = record.get("dimensions").and_then(|v| v.as_object()) {
        for (dimension, value) in dims {
            if !matches!(
                dimension.as_str(),
                "story" | "characters" | "direction" | "animation" | "music"
            ) {
                continue;
            }
            conn.execute(
                "INSERT INTO dimension_scores (record_id, dimension, score) VALUES (?1, ?2, ?3)",
                params![record_id, dimension, json_int(Some(value))],
            )?;
        }
    }
    Ok(true)
}

fn json_int(value: Option<&serde_json::Value>) -> Option<i64> {
    value.and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)))
}

/// 远端 record.delete 落地（物理删除，同本地语义）。
pub(crate) fn apply_remote_record_delete(
    conn: &Connection,
    bangumi_subject_id: i64,
) -> Result<bool, AppError> {
    if entity_has_pending_op(conn, "record", &bangumi_subject_id.to_string())? {
        return Ok(false);
    }
    let local_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM subjects WHERE bangumi_subject_id = ?1",
            params![bangumi_subject_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(local_id) = local_id {
        conn.execute("DELETE FROM subjects WHERE id = ?1", params![local_id])?;
    }
    Ok(true)
}

/// 远端 tier.upsert 落地。名称与异 sync_id 分档撞名 → 也按冲突处理。
pub(crate) fn apply_remote_tier_upsert(
    conn: &Connection,
    sync_id: &str,
    payload: &serde_json::Value,
    server_version: i64,
) -> Result<bool, AppError> {
    if entity_has_pending_op(conn, "tier", sync_id)? {
        return Ok(false);
    }
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let clash: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tiers WHERE name = ?1 AND sync_id IS NOT ?2",
        params![name, sync_id],
        |row| row.get(0),
    )?;
    if clash > 0 {
        return Ok(false);
    }
    let builtin = payload
        .get("builtin")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM tiers WHERE sync_id = ?1",
            params![sync_id],
            |row| row.get(0),
        )
        .optional()?;
    match existing {
        Some(id) => {
            conn.execute(
                "UPDATE tiers SET name = ?1, description = ?2, color = ?3, server_version = ?4 WHERE id = ?5",
                params![
                    name,
                    payload.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    payload.get("color").and_then(|v| v.as_str()).unwrap_or("#335d4e"),
                    server_version,
                    id
                ],
            )?;
        }
        None => {
            let next_order: i64 = conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM tiers",
                [],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT INTO tiers (name, description, color, sort_order, is_builtin, sync_id, server_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    name,
                    payload.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    payload.get("color").and_then(|v| v.as_str()).unwrap_or("#335d4e"),
                    next_order,
                    if builtin { 1 } else { 0 },
                    sync_id,
                    server_version
                ],
            )?;
        }
    }
    Ok(true)
}

/// 远端 tier.delete 落地。仍有本地引用 → 冲突。
pub(crate) fn apply_remote_tier_delete(conn: &Connection, sync_id: &str) -> Result<bool, AppError> {
    if entity_has_pending_op(conn, "tier", sync_id)? {
        return Ok(false);
    }
    let row: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, (SELECT COUNT(*) FROM personal_records WHERE tier_id = tiers.id) FROM tiers WHERE sync_id = ?1",
            params![sync_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((tier_id, refs)) = row else {
        return Ok(true); // 本地没有，视为已对齐
    };
    if refs > 0 {
        return Ok(false);
    }
    conn.execute("DELETE FROM tiers WHERE id = ?1", params![tier_id])?;
    Ok(true)
}

/// 远端 tier_order.set 落地：按 sync_id 列表重排，未知 key 忽略，未列出的排尾部。
pub(crate) fn apply_remote_tier_order(
    conn: &Connection,
    ordered_keys: &[String],
    server_version: i64,
) -> Result<bool, AppError> {
    if entity_has_pending_op(conn, "tier_order", "tier_order")? {
        return Ok(false);
    }
    for (index, key) in ordered_keys.iter().enumerate() {
        conn.execute(
            "UPDATE tiers SET sort_order = ?1 WHERE sync_id = ?2",
            params![index as i64, key],
        )?;
    }
    let mut stmt = conn.prepare(
        "SELECT id FROM tiers WHERE sync_id NOT IN (SELECT value FROM json_each(?1)) OR sync_id IS NULL ORDER BY sort_order, id",
    )?;
    let rest = stmt
        .query_map(params![serde_json::to_string(ordered_keys)?], |row| {
            row.get::<_, i64>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (offset, id) in rest.iter().enumerate() {
        conn.execute(
            "UPDATE tiers SET sort_order = ?1 WHERE id = ?2",
            params![(ordered_keys.len() + offset) as i64, id],
        )?;
    }
    conn.execute(
        "UPDATE sync_state SET tier_order_server_version = ?1 WHERE id = 1",
        params![server_version],
    )?;
    Ok(true)
}

fn upsert_subject_from_snapshot(
    conn: &Connection,
    bangumi_subject_id: i64,
    snapshot: &serde_json::Value,
) -> Result<(), AppError> {
    let now = now_iso();
    let text = |key: &str| {
        snapshot
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let num = |key: &str| snapshot.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
    conn.execute(
        "INSERT INTO subjects (
            bangumi_subject_id, name, name_cn, aliases_json, summary, cover_url,
            year, format, episodes, studio, tags_json, metadata_fetched_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL, ?12, ?12)
         ON CONFLICT(bangumi_subject_id) DO NOTHING",
        params![
            bangumi_subject_id,
            text("name"),
            text("name_cn"),
            snapshot
                .get("aliases")
                .map(|v| serde_json::to_string(v))
                .transpose()?
                .unwrap_or_else(|| "[]".into()),
            text("summary"),
            text("cover_url"),
            num("year"),
            text("format"),
            num("episodes"),
            text("studio"),
            snapshot
                .get("tags")
                .map(|v| serde_json::to_string(v))
                .transpose()?
                .unwrap_or_else(|| "[]".into()),
            now
        ],
    )?;
    Ok(())
}

pub fn backup_to_file(conn: &Connection, dest: &Path) -> Result<(), AppError> {
    let mut dst = Connection::open(dest)?;
    let backup = rusqlite::backup::Backup::new(conn, &mut dst)
        .map_err(|err| AppError::internal(format!("无法创建数据库快照：{err}")))?;
    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|err| AppError::internal(format!("数据库快照失败：{err}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DimensionsDto;

    fn subject(id: i64) -> SubjectDto {
        SubjectDto {
            id,
            name: "Name".into(),
            name_cn: "作品".into(),
            aliases: None,
            summary: "简介".into(),
            cover_url: "https://lain.bgm.tv/pic/cover/l/00.jpg".into(),
            cover_local_path: None,
            year: 2024,
            format: "TV".into(),
            episodes: 12,
            studio: "Studio".into(),
            tags: vec!["日常".into()],
            community: CommunityDto {
                score: Some(8.0),
                votes: 10,
                rank: Some(20),
                fetched_at: Some(now_iso()),
                status: Some("ok".into()),
            },
        }
    }

    fn draft(score: Option<f64>) -> PersonalDraftDto {
        PersonalDraftDto {
            score,
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
            review: "短评".into(),
        }
    }

    #[test]
    fn add_rejects_duplicate_and_keeps_transaction() {
        let mut conn = open_memory().unwrap();
        add_library_entry(&mut conn, &subject(1), &draft(Some(8.0))).unwrap();
        let err = add_library_entry(&mut conn, &subject(1), &draft(Some(9.0))).unwrap_err();
        assert_eq!(err.code, "DUPLICATE");
        assert_eq!(list_library_entries(&conn).unwrap().len(), 1);
    }

    #[test]
    fn update_is_transactional_and_bumps_version() {
        let mut conn = open_memory().unwrap();
        let created = add_library_entry(&mut conn, &subject(2), &draft(Some(7.0))).unwrap();
        let mut next = draft(Some(9.5));
        next.review = "改过了".into();
        next.dimensions.music = Some(4.5);
        let updated =
            update_personal_record(&mut conn, 2, &next, created.personal.version).unwrap();
        assert_eq!(updated.personal.score, Some(9.5));
        assert_eq!(updated.personal.dimensions.music, Some(4.5));
        assert_eq!(updated.personal.version, created.personal.version + 1);
        assert_eq!(updated.subject.name_cn, "作品");
        let stale =
            update_personal_record(&mut conn, 2, &next, created.personal.version).unwrap_err();
        assert_eq!(stale.code, "CONFLICT");
    }

    #[test]
    fn remove_deletes_entry_and_reports_missing() {
        let mut conn = open_memory().unwrap();
        let mut watching = draft(Some(7.0));
        watching.status = "watching".into();
        watching.progress = Some(6);
        add_library_entry(&mut conn, &subject(3), &watching).unwrap();
        let stored = get_library_entry_by_bangumi_id(&conn, 3).unwrap().unwrap();
        assert_eq!(stored.personal.progress, Some(6));

        let cover = remove_library_entry(&conn, 3).unwrap();
        assert_eq!(cover, None);
        assert!(get_library_entry_by_bangumi_id(&conn, 3).unwrap().is_none());

        let err = remove_library_entry(&conn, 3).unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    }

    fn tier_payload(name: &str) -> crate::models::SaveTierPayload {
        crate::models::SaveTierPayload {
            id: None,
            name: name.into(),
            description: "说明".into(),
            color: "#335d4e".into(),
        }
    }

    #[test]
    fn save_tier_rejects_duplicate_name() {
        let conn = open_memory().unwrap();
        let err = save_tier(&conn, tier_payload("A")).unwrap_err();
        assert_eq!(err.code, "DUPLICATE");
        assert_eq!(err.message, "分档名称已存在");
    }

    #[test]
    fn save_tier_rejects_reserved_names() {
        let conn = open_memory().unwrap();
        for name in ["all", "unassigned"] {
            let err = save_tier(&conn, tier_payload(name)).unwrap_err();
            assert_eq!(err.code, "VALIDATION");
            assert_eq!(err.message, "该名称为筛选保留字");
        }
    }

    #[test]
    fn migrates_v1_dimension_tenths_to_star_tenths() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        tx.execute_batch(MIGRATION_V1).unwrap();
        seed_tiers(&tx).unwrap();
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
            params![now_iso()],
        )
        .unwrap();
        tx.commit().unwrap();

        conn.execute(
            "INSERT INTO subjects (bangumi_subject_id, name, name_cn, format, created_at, updated_at)
             VALUES (1, 'N', '作品', 'TV', 't', 't')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO personal_records (subject_id, status, review, created_at, updated_at)
             VALUES (1, 'completed', '', 't', 't')",
            [],
        )
        .unwrap();
        let record_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO dimension_scores (record_id, dimension, score)
             VALUES (?1, 'story', 87), (?1, 'characters', 80), (?1, 'direction', 3), (?1, 'animation', NULL)",
            params![record_id],
        )
        .unwrap();

        migrate(&conn).unwrap();

        let story: i64 = conn
            .query_row(
                "SELECT score FROM dimension_scores WHERE dimension = 'story'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let characters: i64 = conn
            .query_row(
                "SELECT score FROM dimension_scores WHERE dimension = 'characters'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let direction: i64 = conn
            .query_row(
                "SELECT score FROM dimension_scores WHERE dimension = 'direction'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let animation: Option<i64> = conn
            .query_row(
                "SELECT score FROM dimension_scores WHERE dimension = 'animation'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(story, 45);
        assert_eq!(characters, 40);
        assert_eq!(direction, 5);
        assert_eq!(animation, None);
        let progress: Option<i64> = conn
            .query_row(
                "SELECT progress FROM personal_records WHERE id = ?1",
                params![record_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(progress, None);
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 5);
    }

    #[test]
    fn migration_v4_creates_sync_tables_and_backfills_tier_sync_ids() {
        let conn = open_memory().unwrap();
        for table in ["sync_state", "sync_outbox", "sync_conflicts"] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    params![table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "缺表 {table}");
        }
        // 内置分档 sync_id 稳定回填
        let builtin: String = conn
            .query_row("SELECT sync_id FROM tiers WHERE name = 'S'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(builtin, "builtin:S");
        // 设备 id 与默认关闭同步
        let (device_id, enabled): (String, i64) = conn
            .query_row(
                "SELECT device_id, sync_enabled FROM sync_state WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(!device_id.is_empty());
        assert_eq!(enabled, 0);
    }

    #[test]
    fn outbox_enqueues_and_commits_with_business_write() {
        let mut conn = open_memory().unwrap();
        conn.execute(
            "UPDATE sync_state SET sync_enabled = 1, account_id = 'u1' WHERE id = 1",
            [],
        )
        .unwrap();
        add_library_entry(&mut conn, &subject(12), &draft(Some(8.0))).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let (entity, key, op): (String, String, String) = conn
            .query_row(
                "SELECT entity_type, entity_key, op_type FROM sync_outbox",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (entity.as_str(), key.as_str(), op.as_str()),
            ("record", "12", "upsert")
        );
        // 关闭同步后写操作不再入队
        conn.execute("UPDATE sync_state SET sync_enabled = 0 WHERE id = 1", [])
            .unwrap();
        add_library_entry(&mut conn, &subject(34), &draft(Some(7.0))).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 1);
        // 重复写入在事务外被拒绝，不产生 op
        let _ = add_library_entry(&mut conn, &subject(12), &draft(Some(7.0)));
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn outbox_rolls_back_with_failed_write() {
        let mut conn = open_memory().unwrap();
        conn.execute(
            "UPDATE sync_state SET sync_enabled = 1, account_id = 'u1' WHERE id = 1",
            [],
        )
        .unwrap();
        // 让事务内的 outbox INSERT 必败：业务写入必须一并回滚
        conn.execute("ALTER TABLE sync_outbox RENAME TO sync_outbox_bak", [])
            .unwrap();
        let err = add_library_entry(&mut conn, &subject(56), &draft(Some(8.0)));
        assert!(err.is_err());
        let records: i64 = conn
            .query_row("SELECT COUNT(*) FROM personal_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        let pending: i64 = conn
            .query_row("SELECT COUNT(*) FROM sync_outbox_bak", [], |row| row.get(0))
            .unwrap();
        assert_eq!((records, pending), (0, 0));
    }

    #[test]
    fn remove_enqueues_delete_op() {
        let mut conn = open_memory().unwrap();
        add_library_entry(&mut conn, &subject(78), &draft(Some(8.0))).unwrap();
        conn.execute(
            "UPDATE sync_state SET sync_enabled = 1, account_id = 'u1' WHERE id = 1",
            [],
        )
        .unwrap();
        remove_library_entry(&conn, 78).unwrap();
        let (op, key): (String, String) = conn
            .query_row(
                "SELECT op_type, entity_key FROM sync_outbox WHERE entity_type = 'record'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!((op.as_str(), key.as_str()), ("delete", "78"));
    }
}
