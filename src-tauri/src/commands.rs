use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::bangumi::BangumiClient;
use crate::covers;
use crate::db;
use crate::error::AppError;
use crate::models::{
    AddEntryPayload, BackupDocument, BootstrapInfo, ImportPayload, ImportPreview, LibraryEntryDto,
    ListLibraryQuery, ListLibraryResponse, RelatedSubjectDto, SaveTierPayload, StatisticsDto,
    SubjectDto, TierDto, UpdateRecordPayload,
};
use crate::{backup, stats};

pub struct LiveState {
    pub db: Mutex<rusqlite::Connection>,
    pub data_dir: PathBuf,
    pub bangumi: BangumiClient,
}

pub struct AppState {
    pub ready: Option<LiveState>,
    pub error: Option<String>,
}

fn live(state: &AppState) -> Result<&LiveState, AppError> {
    state.ready.as_ref().ok_or_else(|| {
        AppError::startup(
            state
                .error
                .clone()
                .unwrap_or_else(|| "应用启动失败".into()),
        )
    })
}

fn lock_db(live: &LiveState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, AppError> {
    live.db
        .lock()
        .map_err(|_| AppError::internal("数据库锁失效"))
}

#[tauri::command]
pub fn bootstrap(state: State<AppState>) -> Result<BootstrapInfo, AppError> {
    let live = live(&state)?;
    Ok(BootstrapInfo {
        data_dir: live.data_dir.to_string_lossy().into(),
        log_path: live.data_dir.join("logs").join("kiroku.log").to_string_lossy().into(),
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
    let limit = if query.limit <= 0 { 500 } else { query.limit as usize };
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
    db::update_personal_record(
        &mut conn,
        payload.bangumi_subject_id,
        &payload.draft,
        payload.version,
    )
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
    db::save_tier(&conn, payload)
}

#[tauri::command]
pub fn reorder_tiers(state: State<AppState>, ids: Vec<i64>) -> Result<Vec<TierDto>, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    db::reorder_tiers(&conn, ids)
}

#[tauri::command]
pub fn delete_tier(state: State<AppState>, id: i64) -> Result<(), AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    db::delete_tier(&conn, id)
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
pub async fn get_subject(state: State<'_, AppState>, bangumi_subject_id: i64) -> Result<SubjectDto, AppError> {
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
pub fn preview_import(state: State<AppState>, document: BackupDocument) -> Result<ImportPreview, AppError> {
    let live = live(&state)?;
    let conn = lock_db(live)?;
    backup::preview_import(&conn, &document)
}

#[tauri::command]
pub fn import_backup(state: State<AppState>, payload: ImportPayload) -> Result<ImportPreview, AppError> {
    let live = live(&state)?;
    let mut conn = lock_db(live)?;
    backup::import_document(&mut conn, payload)
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

#[tauri::command]
pub fn snapshot_database(state: State<AppState>) -> Result<String, AppError> {
    let live = live(&state)?;
    let dest_dir = live.data_dir.join("backups");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(format!("kiroku-{}.db", crate::validate::now_iso().replace(':', "-")));
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
    Ok(LiveState {
        db: Mutex::new(conn),
        data_dir,
        bangumi: BangumiClient::new()?,
    })
}
