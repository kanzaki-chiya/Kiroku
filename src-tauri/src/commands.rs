use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

use crate::auth::{self, Session};
use crate::bangumi::BangumiClient;
use crate::covers;
use crate::db;
use crate::error::AppError;
use crate::models::{
    AddEntryPayload, BackupDocument, BootstrapInfo, CalendarDayDto, ImportPayload, ImportPreview,
    LibraryEntryDto, ListLibraryQuery, ListLibraryResponse, RelatedSubjectDto, SaveTierPayload,
    StatisticsDto, SubjectDto, SyncConflictDto, SyncLoginPayload, SyncResolvePayload,
    SyncStatusDto, TierDto, UpdateRecordPayload,
};
use crate::{backup, stats, sync};

pub struct LiveState {
    pub db: Mutex<rusqlite::Connection>,
    pub data_dir: PathBuf,
    pub bangumi: BangumiClient,
    pub http: reqwest::Client,
    pub session: Mutex<Option<Session>>,
    pub sync_notify: Arc<tokio::sync::Notify>,
}

pub struct AppState {
    pub ready: Option<LiveState>,
    pub error: Option<String>,
}

fn live(state: &AppState) -> Result<&LiveState, AppError> {
    state.ready.as_ref().ok_or_else(|| {
        AppError::startup(state.error.clone().unwrap_or_else(|| "应用启动失败".into()))
    })
}

fn lock_db(live: &LiveState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, AppError> {
    live.db
        .lock()
        .map_err(|_| AppError::internal("数据库锁失效"))
}

/// 写操作后唤起同步 worker（未开启同步时 worker 自检即退，代价为零）。
fn kick_sync(live: &LiveState) {
    live.sync_notify.notify_one();
}

#[tauri::command]
pub fn bootstrap(state: State<AppState>) -> Result<BootstrapInfo, AppError> {
    let live = live(&state)?;
    Ok(BootstrapInfo {
        data_dir: live.data_dir.to_string_lossy().into(),
        log_path: live
            .data_dir
            .join("logs")
            .join("kiroku.log")
            .to_string_lossy()
            .into(),
        db_path: live.data_dir.join("kiroku.db").to_string_lossy().into(),
    })
}

#[tauri::command]
pub fn list_library(
    state: State<AppState>,
    query: Option<ListLibraryQuery>,
) -> Result<ListLibraryResponse, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    let query = query.unwrap_or_default();
    let mut items = db::list_library_entries(&conn)?;
    items = stats::filter_entries(
        items,
        &query.query,
        &query.tier,
        &query.status,
        query.year,
        &query.sort,
        &query.direction,
    );
    let total = items.len() as i64;
    let offset = query.offset.max(0) as usize;
    let limit = if query.limit <= 0 {
        500
    } else {
        query.limit as usize
    };
    let items = items.into_iter().skip(offset).take(limit).collect();
    Ok(ListLibraryResponse { items, total })
}

#[tauri::command]
pub fn get_library_entry(
    state: State<AppState>,
    bangumi_subject_id: i64,
) -> Result<Option<LibraryEntryDto>, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    db::get_library_entry_by_bangumi_id(&conn, bangumi_subject_id)
}

#[tauri::command]
pub fn add_library_entry(
    app: AppHandle,
    state: State<AppState>,
    payload: AddEntryPayload,
) -> Result<LibraryEntryDto, AppError> {
    let live = live(&state)?;
    let remote_cover = payload.subject.cover_url.clone();
    let entry = {
        let mut conn = lock_db(live)?;
        db::add_library_entry(&mut conn, &payload.subject, &payload.draft)?
    };
    kick_sync(live);
    spawn_cover_download(
        app,
        live.data_dir.clone(),
        entry.local_id,
        entry.subject.id,
        remote_cover,
    );
    if let Err(err) = remember_recent(&state, &entry.subject) {
        log::warn!("收藏已保存，但最近搜索未更新：{err}");
    }
    Ok(entry)
}

#[tauri::command]
pub fn update_personal_record(
    state: State<AppState>,
    payload: UpdateRecordPayload,
) -> Result<LibraryEntryDto, AppError> {
    let live = live(&state)?;
    let mut conn = lock_db(live)?;
    let entry = db::update_personal_record(
        &mut conn,
        payload.bangumi_subject_id,
        &payload.draft,
        payload.version,
    )?;
    drop(conn);
    kick_sync(live);
    Ok(entry)
}

#[tauri::command]
pub fn remove_library_entry(
    state: State<AppState>,
    bangumi_subject_id: i64,
) -> Result<(), AppError> {
    let live = live(&state)?;
    let cover_path = {
        let conn = lock_db(live)?;
        db::remove_library_entry(&conn, bangumi_subject_id)?
    };
    kick_sync(live);
    if let Some(relative) = cover_path {
        let path = live.data_dir.join(&relative);
        if let Err(err) = fs::remove_file(&path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("移除收藏后封面清理失败：{err}");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_tiers(state: State<AppState>) -> Result<Vec<TierDto>, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    db::list_tiers(&conn)
}

#[tauri::command]
pub fn save_tier(state: State<AppState>, payload: SaveTierPayload) -> Result<TierDto, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    let tier = db::save_tier(&conn, payload)?;
    drop(conn);
    kick_sync(live);
    Ok(tier)
}

#[tauri::command]
pub fn reorder_tiers(state: State<AppState>, ids: Vec<i64>) -> Result<Vec<TierDto>, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    let tiers = db::reorder_tiers(&conn, ids)?;
    drop(conn);
    kick_sync(live);
    Ok(tiers)
}

#[tauri::command]
pub fn delete_tier(state: State<AppState>, id: i64) -> Result<(), AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    db::delete_tier(&conn, id)?;
    drop(conn);
    kick_sync(live);
    Ok(())
}

#[tauri::command]
pub async fn search_subjects(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SubjectDto>, AppError> {
    let live = live(&state)?;
    live.bangumi.search(&query).await
}

#[tauri::command]
pub async fn get_subject(
    state: State<'_, AppState>,
    bangumi_subject_id: i64,
) -> Result<SubjectDto, AppError> {
    let live = live(&state)?;
    let subject = live.bangumi.get_subject(bangumi_subject_id).await?;
    if let Err(err) = remember_recent(&state, &subject) {
        log::warn!("已获取资料，但最近搜索未更新：{err}");
    }
    Ok(subject)
}

#[tauri::command]
pub async fn get_subject_relations(
    state: State<'_, AppState>,
    bangumi_subject_id: i64,
) -> Result<Vec<RelatedSubjectDto>, AppError> {
    let live = live(&state)?;
    live.bangumi.get_relations(bangumi_subject_id).await
}

#[tauri::command]
pub async fn get_calendar(state: State<'_, AppState>) -> Result<Vec<CalendarDayDto>, AppError> {
    let live = live(&state)?;
    live.bangumi.get_calendar().await
}

#[tauri::command]
pub fn list_recent_searches(state: State<AppState>) -> Result<Vec<SubjectDto>, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    let raw = db::get_setting(&conn, "recent_searches")?;
    match raw {
        Some(value) => Ok(serde_json::from_str(&value).unwrap_or_default()),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn refresh_subject(
    app: AppHandle,
    state: State<'_, AppState>,
    bangumi_subject_id: i64,
) -> Result<LibraryEntryDto, AppError> {
    let live = live(&state)?;
    let fetched = live.bangumi.get_subject(bangumi_subject_id).await?;
    let entry = {
        let mut conn = lock_db(live)?;
        db::refresh_subject_metadata(&mut conn, &fetched)?
    };
    let cover_missing = entry
        .subject
        .cover_local_path
        .as_deref()
        .map(|path| !live.data_dir.join(path).exists())
        .unwrap_or(true);
    if cover_missing {
        spawn_cover_download(
            app,
            live.data_dir.clone(),
            entry.local_id,
            entry.subject.id,
            fetched.cover_url.clone(),
        );
    }
    Ok(entry)
}

#[tauri::command]
pub fn get_statistics(state: State<AppState>) -> Result<StatisticsDto, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    let entries = db::list_library_entries(&conn)?;
    Ok(stats::calculate_statistics(&entries))
}

#[tauri::command]
pub fn export_backup(state: State<AppState>) -> Result<BackupDocument, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    backup::export_document(&conn)
}

#[tauri::command]
pub fn preview_import(
    state: State<AppState>,
    document: BackupDocument,
) -> Result<ImportPreview, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    backup::preview_import(&conn, &document)
}

#[tauri::command]
pub fn import_backup(
    state: State<AppState>,
    payload: ImportPayload,
) -> Result<ImportPreview, AppError> {
    let live = live(&state)?;
    let mut conn = lock_db(live)?;
    let preview = backup::import_document(&mut conn, payload)?;
    drop(conn);
    kick_sync(live);
    Ok(preview)
}

#[tauri::command]
pub async fn clear_cover_cache(state: State<'_, AppState>) -> Result<(), AppError> {
    let live = live(&state)?;
    covers::clear_cover_cache(&live.data_dir).await?;
    let conn = lock_db(live)?;
    conn.execute("UPDATE subjects SET cover_local_path = NULL", [])?;
    Ok(())
}

#[tauri::command]
pub async fn delete_personal_data(state: State<'_, AppState>) -> Result<(), AppError> {
    let live = live(&state)?;
    {
        let mut conn = lock_db(live)?;
        db::delete_personal_data(&mut conn)?;
    }
    covers::clear_cover_cache(&live.data_dir).await
}

// ---------- 云同步命令（SYNC_DESIGN.md） ----------

#[tauri::command]
pub async fn sync_login(
    state: State<'_, AppState>,
    payload: SyncLoginPayload,
) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    let session = auth::login(
        &live.http,
        &live.data_dir,
        &payload.email,
        &payload.password,
    )
    .await?;
    if let Err(err) = sync::bind_account(live, &session.user_id) {
        auth::logout(&live.http, &live.data_dir, Some(session)).await;
        return Err(err);
    }
    if let Ok(mut slot) = live.session.lock() {
        *slot = Some(session);
    }
    kick_sync(live);
    sync::status(live)
}

#[tauri::command]
pub async fn sync_signup(
    state: State<'_, AppState>,
    payload: SyncLoginPayload,
) -> Result<serde_json::Value, AppError> {
    let live = live(&state)?;
    match auth::signup(
        &live.http,
        &live.data_dir,
        &payload.email,
        &payload.password,
    )
    .await?
    {
        auth::SignupOutcome::ConfirmEmail => {
            Ok(serde_json::json!({ "status": "confirm_email" }))
        }
        auth::SignupOutcome::Session(session) => {
            // 自动确认开启：注册即登录，与 sync_login 同路径绑定账号
            if let Err(err) = sync::bind_account(live, &session.user_id) {
                auth::logout(&live.http, &live.data_dir, Some(session)).await;
                return Err(err);
            }
            if let Ok(mut slot) = live.session.lock() {
                *slot = Some(session);
            }
            kick_sync(live);
            Ok(serde_json::json!({ "status": "signed_in" }))
        }
    }
}

#[tauri::command]
pub async fn sync_logout(state: State<'_, AppState>) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    let session = live.session.lock().ok().and_then(|mut slot| slot.take());
    auth::logout(&live.http, &live.data_dir, session).await;
    sync::bump_session_generation(live)?;
    sync::status(live)
}

#[tauri::command]
pub async fn sync_status(state: State<'_, AppState>) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    let mut status = sync::status(live)?;
    if status.logged_in {
        // 会员信息是只读查询；离线/失败时保持 None，不影响其余字段
        if let Ok(ent) = sync::get_entitlement(live).await {
            status.member_active = ent.get("member_active").and_then(|v| v.as_bool());
            status.expires_at = ent
                .get("expires_at")
                .and_then(|v| v.as_str())
                .map(String::from);
            status.in_retention = ent.get("in_retention").and_then(|v| v.as_bool());
        }
    }
    Ok(status)
}

#[tauri::command]
pub async fn sync_set_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    if enabled {
        sync::initial_enable(live).await?; // 有冲突时返回 Ok 但 sync_enabled 仍为 0
    } else {
        sync::set_sync_enabled(live, false)?;
    }
    kick_sync(live);
    sync::status(live)
}

/// 轻量唤起同步 worker（窗口回前台/网络恢复用），不阻塞等结果。
#[tauri::command]
pub fn sync_kick(state: State<AppState>) -> Result<(), AppError> {
    let live = live(&state)?;
    kick_sync(live);
    Ok(())
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    sync::run_once(live).await?;
    sync::status(live)
}

#[tauri::command]
pub async fn sync_redeem_code(
    state: State<'_, AppState>,
    code: String,
) -> Result<serde_json::Value, AppError> {
    let live = live(&state)?;
    sync::redeem(live, &code).await
}

#[tauri::command]
pub fn sync_list_conflicts(state: State<AppState>) -> Result<Vec<SyncConflictDto>, AppError> {
    let live = live(&state)?;
    sync::list_conflicts(live)
}

#[tauri::command]
pub async fn sync_resolve_conflict(
    state: State<'_, AppState>,
    payload: SyncResolvePayload,
) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    sync::resolve_conflict(
        live,
        &payload.entity_type,
        &payload.entity_key,
        &payload.keep,
    )?;
    kick_sync(live);
    sync::status(live)
}

#[tauri::command]
pub async fn sync_reconcile(
    state: State<'_, AppState>,
    mode: String,
) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    sync::reconcile(live, &mode).await?;
    sync::status(live)
}

#[tauri::command]
pub async fn sync_delete_cloud_library(
    state: State<'_, AppState>,
    confirm: String,
) -> Result<SyncStatusDto, AppError> {
    let live = live(&state)?;
    sync::delete_cloud(live, &confirm).await?;
    sync::status(live)
}

#[tauri::command]
pub fn snapshot_database(state: State<AppState>) -> Result<String, AppError> {
    let live = live(&state)?;
    let dest_dir = live.data_dir.join("backups");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(format!(
        "kiroku-{}.db",
        crate::validate::now_iso().replace(':', "-")
    ));
    let conn = lock_db(live)?;
    db::backup_to_file(&conn, &dest)?;
    Ok(dest.to_string_lossy().into())
}

fn remember_recent(state: &AppState, subject: &SubjectDto) -> Result<(), AppError> {
    let live = live(state)?;
    let conn = lock_db(live)?;
    let mut list: Vec<SubjectDto> = db::get_setting(&conn, "recent_searches")?
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_default();
    list.retain(|item| item.id != subject.id);
    list.insert(0, subject.clone());
    list.truncate(8);
    db::set_setting(&conn, "recent_searches", &serde_json::to_string(&list)?)
}

fn spawn_cover_download(
    app: AppHandle,
    data_dir: PathBuf,
    local_id: i64,
    bangumi_subject_id: i64,
    remote_url: String,
) {
    if remote_url.trim().is_empty() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        match covers::download_cover(&data_dir, local_id, &remote_url).await {
            Ok(relative) => {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(live) = live(&state) {
                        if let Ok(conn) = lock_db(live) {
                            let _ = db::set_cover_path(&conn, bangumi_subject_id, &relative);
                        }
                    }
                }
                covers::emit_cover_ready(&app, bangumi_subject_id, relative);
            }
            Err(err) => log::warn!("封面下载失败（不影响收藏）：{err}"),
        }
    });
}

pub fn init_state(app: &AppHandle) -> AppState {
    match init_live(app) {
        Ok(live) => AppState {
            ready: Some(live),
            error: None,
        },
        Err(err) => {
            log::error!("启动失败：{err}");
            AppState {
                ready: None,
                error: Some(err.message),
            }
        }
    }
}

fn init_live(app: &AppHandle) -> Result<LiveState, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::startup(format!("无法确定数据目录：{err}")))?;
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(data_dir.join("covers"))?;
    let log_dir = data_dir.join("logs");
    fs::create_dir_all(&log_dir)?;
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("kiroku.log"))?;
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        log_file,
    );
    let db_path = data_dir.join("kiroku.db");
    let conn = db::open(&db_path)?;
    log::info!("database ready at {}", db_path.display());
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|err| AppError::startup(format!("无法创建网络客户端：{err}")))?;
    let session = auth::load_session(&data_dir);
    Ok(LiveState {
        db: Mutex::new(conn),
        data_dir,
        bangumi: BangumiClient::new()?,
        http,
        session: Mutex::new(session),
        sync_notify: Arc::new(tokio::sync::Notify::new()),
    })
}

/// 同步 worker：事件触发 + 15 分钟兜底轮询（SYNC_DESIGN §9）。
/// 本地写永远先行，worker 失败只记 last_error。
pub fn spawn_sync_worker(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));
        interval.tick().await; // 首次 tick 立即触发（启动时推一轮）
        loop {
            let state = handle.state::<AppState>();
            if let Ok(live) = live(&state) {
                if let Err(err) = sync::run_once(live).await {
                    log::warn!("同步失败：{}", err.message);
                }
                let notify = live.sync_notify.clone();
                drop(state);
                tokio::select! {
                    _ = interval.tick() => {}
                    _ = notify.notified() => {}
                }
            } else {
                interval.tick().await;
            }
        }
    });
}
