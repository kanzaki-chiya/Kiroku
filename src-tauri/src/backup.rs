use rusqlite::Connection;

use crate::db;
use crate::error::AppError;
use crate::models::{BackupDocument, BackupEntry, ImportPayload, ImportPreview};
use crate::validate::now_iso;

const FORMAT_VERSION: i64 = 3;

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
    let document = normalize_document(document)?;
    preview_normalized(conn, &document)
}

pub fn import_document(conn: &mut Connection, payload: ImportPayload) -> Result<ImportPreview, AppError> {
    let document = normalize_document(&payload.document)?;
    let preview = preview_normalized(conn, &document)?;
    db::import_entries(conn, &document.entries, payload.overwrite)?;
    Ok(preview)
}

fn preview_normalized(conn: &Connection, document: &BackupDocument) -> Result<ImportPreview, AppError> {
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

fn validate_document(document: &BackupDocument) -> Result<(), AppError> {
    if !(1..=3).contains(&document.format_version) {
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

fn scale_v1_dimension(value: Option<f64>) -> Option<f64> {
    value.map(|v| (((v / 2.0) * 2.0).round() / 2.0).max(0.5))
}

fn convert_v1_dimensions(dims: &mut crate::models::DimensionsDto) {
    dims.story = scale_v1_dimension(dims.story);
    dims.characters = scale_v1_dimension(dims.characters);
    dims.direction = scale_v1_dimension(dims.direction);
    dims.animation = scale_v1_dimension(dims.animation);
    dims.music = scale_v1_dimension(dims.music);
}

fn normalize_document(document: &BackupDocument) -> Result<BackupDocument, AppError> {
    validate_document(document)?;
    let mut document = document.clone();
    if document.format_version == 1 {
        for entry in &mut document.entries {
            convert_v1_dimensions(&mut entry.personal.dimensions);
        }
    }
    Ok(document)
}

fn personal_conflicts(
    left: &crate::models::PersonalRecordDto,
    right: &crate::models::PersonalRecordDto,
) -> bool {
    left.score != right.score
        || left.review != right.review
        || left.tier != right.tier
        || left.status != right.status
        || left.progress != right.progress
        || left.dimensions != right.dimensions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{CommunityDto, DimensionsDto, ImportPayload, PersonalDraftDto, SubjectDto};

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
            progress: Some(12),
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
    fn dimension_only_difference_counts_as_conflict() {
        let mut conn = db::open_memory().unwrap();
        db::add_library_entry(&mut conn, &subject(1), &draft()).unwrap();
        let mut document = export_document(&conn).unwrap();
        document.entries[0].personal.dimensions.music = Some(4.5);
        let preview = preview_import(&conn, &document).unwrap();
        assert_eq!(preview.added, 0);
        assert_eq!(preview.duplicates, 1);
        assert_eq!(preview.conflicts, 1);
    }

    #[test]
    fn imports_v1_dimensions_as_half_stars() {
        let mut conn = db::open_memory().unwrap();
        db::add_library_entry(&mut conn, &subject(1), &draft()).unwrap();
        let mut document = export_document(&conn).unwrap();
        assert_eq!(document.format_version, 3);
        document.format_version = 1;
        document.entries[0].personal.dimensions.story = Some(8.7);
        document.entries[0].personal.dimensions.music = Some(9.0);
        document.entries[0].personal.dimensions.direction = Some(0.4);
        import_document(
            &mut conn,
            ImportPayload {
                document,
                overwrite: true,
            },
        )
        .unwrap();
        let entries = db::list_library_entries(&conn).unwrap();
        assert_eq!(entries[0].personal.dimensions.story, Some(4.5));
        assert_eq!(entries[0].personal.dimensions.music, Some(4.5));
        assert_eq!(entries[0].personal.dimensions.direction, Some(0.5));
        assert_eq!(entries[0].personal.dimensions.characters, None);
    }
}
