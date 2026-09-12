use rusqlite::Connection;

use crate::db;
use crate::error::AppError;
use crate::models::{BackupDocument, BackupEntry, ImportPayload, ImportPreview};
use crate::validate::now_iso;

const FORMAT_VERSION: i64 = 1;

pub fn export_document(conn: &Connection) -> Result<BackupDocument, AppError> {
    let entries = db::list_library_entries(conn)?
        .into_iter()
        .map(|entry| BackupEntry {
            bangumi_subject_id: entry.subject.id,
            subject: entry.subject,
            personal: entry.personal,
        })
        .collect();
    Ok(BackupDocument {
        format_version: FORMAT_VERSION,
        exported_at: now_iso(),
        tiers: db::list_tiers(conn)?,
        entries,
    })
}

pub fn preview_import(conn: &Connection, document: &BackupDocument) -> Result<ImportPreview, AppError> {
    validate_document(document)?;
    let existing = db::list_library_entries(conn)?;
    let mut added = 0;
    let mut duplicates = 0;
    let mut conflicts = 0;
    for item in &document.entries {
        match existing.iter().find(|entry| entry.subject.id == item.bangumi_subject_id) {
            None => added += 1,
            Some(entry) => {
                duplicates += 1;
                if personal_conflicts(&entry.personal, &item.personal) {
                    conflicts += 1;
                }
            }
        }
    }
    Ok(ImportPreview {
        added,
        duplicates,
        conflicts,
    })
}

pub fn import_document(conn: &mut Connection, payload: ImportPayload) -> Result<ImportPreview, AppError> {
    validate_document(&payload.document)?;
    let preview = preview_import(conn, &payload.document)?;
    db::import_entries(conn, &payload.document.entries, payload.overwrite)?;
    Ok(preview)
}

fn validate_document(document: &BackupDocument) -> Result<(), AppError> {
    if document.format_version != FORMAT_VERSION {
        return Err(AppError::validation(format!(
            "不支持的备份版本 {}",
            document.format_version
        )));
    }
    for entry in &document.entries {
        if entry.bangumi_subject_id != entry.subject.id
            || entry.bangumi_subject_id != entry.personal.subject_id
        {
            return Err(AppError::validation("备份中的条目 ID 不一致"));
        }
    }
    Ok(())
}

fn personal_conflicts(
    left: &crate::models::PersonalRecordDto,
    right: &crate::models::PersonalRecordDto,
) -> bool {
    left.score != right.score
        || left.review != right.review
        || left.tier != right.tier
        || left.status != right.status
        || left.dimensions != right.dimensions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{CommunityDto, DimensionsDto, PersonalDraftDto, SubjectDto};

    fn subject(id: i64) -> SubjectDto {
        SubjectDto {
            id,
            name: "Name".into(),
            name_cn: "作品".into(),
            aliases: None,
            summary: String::new(),
            cover_url: "https://lain.bgm.tv/pic/cover/l/00.jpg".into(),
            cover_local_path: None,
            year: 2024,
            format: "TV".into(),
            episodes: 12,
            studio: "Studio".into(),
            tags: vec![],
            community: CommunityDto {
                score: Some(8.0),
                votes: 10,
                rank: None,
                fetched_at: None,
                status: Some("ok".into()),
            },
        }
    }

    fn draft() -> PersonalDraftDto {
        PersonalDraftDto {
            score: Some(8.0),
            tier: Some("A".into()),
            status: "completed".into(),
            dimensions: DimensionsDto {
                story: Some(8.0),
                characters: None,
                direction: None,
                animation: None,
                music: None,
            },
            review: "短评".into(),
        }
    }

    #[test]
    fn dimension_only_difference_counts_as_conflict() {
        let mut conn = db::open_memory().unwrap();
        db::add_library_entry(&mut conn, &subject(1), &draft()).unwrap();
        let mut document = export_document(&conn).unwrap();
        document.entries[0].personal.dimensions.music = Some(9.0);
        let preview = preview_import(&conn, &document).unwrap();
        assert_eq!(preview.added, 0);
        assert_eq!(preview.duplicates, 1);
        assert_eq!(preview.conflicts, 1);
    }
}
