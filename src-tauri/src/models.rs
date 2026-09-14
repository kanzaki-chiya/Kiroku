use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityDto {
    pub score: Option<f64>,
    pub votes: i64,
    pub rank: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDto {
    pub id: i64,
    pub name: String,
    pub name_cn: String,
    #[serde(default)]
    pub aliases: Option<Vec<String>>,
    pub summary: String,
    pub cover_url: String,
    #[serde(default)]
    pub cover_local_path: Option<String>,
    pub year: i64,
    pub format: String,
    pub episodes: i64,
    pub studio: String,
    pub tags: Vec<String>,
    pub community: CommunityDto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionsDto {
    pub story: Option<f64>,
    pub characters: Option<f64>,
    pub direction: Option<f64>,
    pub animation: Option<f64>,
    pub music: Option<f64>,
}

impl DimensionsDto {
    pub fn values(&self) -> [(&'static str, Option<f64>); 5] {
        [
            ("story", self.story),
            ("characters", self.characters),
            ("direction", self.direction),
            ("animation", self.animation),
            ("music", self.music),
        ]
    }

    pub fn from_map(mut values: std::collections::HashMap<String, Option<f64>>) -> Self {
        Self {
            story: values.remove("story").flatten(),
            characters: values.remove("characters").flatten(),
            direction: values.remove("direction").flatten(),
            animation: values.remove("animation").flatten(),
            music: values.remove("music").flatten(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalDraftDto {
    pub score: Option<f64>,
    pub tier: Option<String>,
    pub status: String,
    #[serde(default)]
    pub progress: Option<i64>,
    pub dimensions: DimensionsDto,
    pub review: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalRecordDto {
    pub score: Option<f64>,
    pub tier: Option<String>,
    pub status: String,
    #[serde(default)]
    pub progress: Option<i64>,
    pub dimensions: DimensionsDto,
    pub review: String,
    pub subject_id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedSubjectDto {
    pub relation: String,
    pub subject: SubjectDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDayDto {
    pub weekday: i64,
    pub label: String,
    pub items: Vec<SubjectDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntryDto {
    pub local_id: i64,
    pub subject: SubjectDto,
    pub personal: PersonalRecordDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLibraryQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub status: String,
    pub year: Option<i64>,
    #[serde(default)]
    pub sort: String,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub limit: i64,
}

impl Default for ListLibraryQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            tier: "all".into(),
            status: "all".into(),
            year: None,
            sort: "updated".into(),
            direction: "desc".into(),
            offset: 0,
            limit: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLibraryResponse {
    pub items: Vec<LibraryEntryDto>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEntryPayload {
    pub subject: SubjectDto,
    pub draft: PersonalDraftDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecordPayload {
    pub bangumi_subject_id: i64,
    pub draft: PersonalDraftDto,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub color: String,
    pub sort_order: i64,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTierPayload {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapInfo {
    pub data_dir: String,
    pub log_path: String,
    pub db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverReadyEvent {
    pub bangumi_subject_id: i64,
    pub cover_local_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatBinDto {
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatDimensionDto {
    pub key: String,
    pub mean: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatDifferenceDto {
    pub delta: f64,
    pub entry: LibraryEntryDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsDto {
    pub total: i64,
    pub rated_count: i64,
    pub community_count: i64,
    pub paired_count: i64,
    pub personal_mean: Option<f64>,
    pub community_mean: Option<f64>,
    pub mean_difference: Option<f64>,
    pub highest: Option<LibraryEntryDto>,
    pub differences: Vec<StatDifferenceDto>,
    pub bins: Vec<StatBinDto>,
    pub dimensions: Vec<StatDimensionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDocument {
    pub format_version: i64,
    pub exported_at: String,
    pub tiers: Vec<TierDto>,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub bangumi_subject_id: i64,
    pub subject: SubjectDto,
    pub personal: PersonalRecordDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub added: i64,
    pub duplicates: i64,
    pub conflicts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPayload {
    pub document: BackupDocument,
    #[serde(default)]
    pub overwrite: bool,
}
