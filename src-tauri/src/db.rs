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

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path).map_err(|err| {
        AppError::startup(format!("无法打开数据库：{err}"))
    })?;
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
    conn.query_row("SELECT id FROM tiers WHERE name = ?1", params![name], |row| {
        row.get(0)
    })
    .optional()?
    .ok_or_else(|| AppError::validation("无效的分档"))
    .map(Some)
}

pub fn save_tier(conn: &Connection, payload: crate::models::SaveTierPayload) -> Result<TierDto, AppError> {
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
        conn.execute(
            "UPDATE tiers SET name = ?1, description = ?2, color = ?3 WHERE id = ?4",
            params![name, payload.description, payload.color, id],
        )?;
        get_tier(conn, id)
    } else {
        let next_order: i64 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM tiers",
            [],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO tiers (name, description, color, sort_order, is_builtin) VALUES (?1, ?2, ?3, ?4, 0)",
            params![name, payload.description, payload.color, next_order],
        )?;
        get_tier(conn, conn.last_insert_rowid())
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
    let deleted = conn.execute("DELETE FROM tiers WHERE id = ?1 AND is_builtin = 0", params![id])?;
    if deleted == 0 {
        let builtin: i64 = conn
            .query_row(
                "SELECT is_builtin FROM tiers WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        if builtin == 1 {
            return Err(AppError::conflict("内置分档不能删除"));
        }
        return Err(AppError::not_found("分档不存在"));
    }
    Ok(())
}

pub fn upsert_subject(tx: &Transaction<'_>, subject: &SubjectDto, fetched_at: &str) -> Result<i64, AppError> {
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
            subject.community.fetched_at.clone().unwrap_or_else(|| fetched_at.to_string()),
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
            "SELECT s.id, s.cover_local_path
             FROM subjects s
             JOIN personal_records p ON p.subject_id = s.id
             WHERE s.bangumi_subject_id = ?1",
            params![bangumi_subject_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?;
    let Some((local_id, cover_path)) = row else {
        return Err(AppError::not_found("这部作品还没有收录"));
    };
    conn.execute("DELETE FROM subjects WHERE id = ?1", params![local_id])?;
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
    let Some((record_id, current_version, _created_at, _subject_id)) = row else {
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
        let existing: Option<(i64, i64)> = tx
            .query_row(
                "SELECT p.id, p.version
                 FROM personal_records p
                 JOIN subjects s ON s.id = p.subject_id
                 WHERE s.bangumi_subject_id = ?1",
                params![item.bangumi_subject_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let now = now_iso();
        if let Some((record_id, version)) = existing {
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
         DELETE FROM subjects;",
    )?;
    tx.commit()?;
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
        let stale = update_personal_record(&mut conn, 2, &next, created.personal.version).unwrap_err();
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
        assert_eq!(version, 3);
    }
}
