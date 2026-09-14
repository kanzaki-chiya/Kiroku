use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::redirect::Policy;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use url::Url;

use crate::error::AppError;
use crate::models::{CommunityDto, RelatedSubjectDto, SubjectDto};
use crate::validate::now_iso;

const BASE: &str = "https://api.bgm.tv";
const USER_AGENT_VALUE: &str = "Kiroku/0.1.0 (personal desktop client)";
const MIN_INTERVAL: Duration = Duration::from_millis(400);
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct BangumiClient {
    http: reqwest::Client,
    last_request: Arc<Mutex<Option<Instant>>>,
}

impl BangumiClient {
    pub fn new() -> Result<Self, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(USER_AGENT_VALUE),
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(TIMEOUT)
            .redirect(Policy::limited(3))
            .build()
            .map_err(|err| AppError::startup(format!("无法初始化网络客户端：{err}")))?;
        Ok(Self {
            http,
            last_request: Arc::new(Mutex::new(None)),
        })
    }


    async fn throttle(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(previous) = *last {
            let elapsed = previous.elapsed();
            if elapsed < MIN_INTERVAL {
                tokio::time::sleep(MIN_INTERVAL - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<Value>,
    ) -> Result<T, AppError> {
        let mut last_error = AppError::network("Bangumi 请求失败");
        for attempt in 0..3 {
            self.throttle().await;
            let mut builder = self.http.request(method.clone(), url);
            if let Some(payload) = &body {
                builder = builder.json(payload);
            }
            match builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.as_u16() == 404 {
                        return Err(AppError::not_found("Bangumi 上找不到这部作品"));
                    }
                    if status.as_u16() == 429 || status.is_server_error() {
                        last_error = AppError::network(format!("Bangumi 暂时不可用（{status}）"));
                        tokio::time::sleep(Duration::from_millis(400 * (attempt as u64 + 1))).await;
                        continue;
                    }
                    if !status.is_success() {
                        return Err(AppError::network(format!("Bangumi 返回 {status}")));
                    }
                    return response.json::<T>().await.map_err(AppError::from);
                }
                Err(err) if err.is_timeout() || err.is_connect() || err.is_request() => {
                    last_error = AppError::from(err);
                    tokio::time::sleep(Duration::from_millis(400 * (attempt as u64 + 1))).await;
                }
                Err(err) => return Err(AppError::from(err)),
            }
        }
        Err(last_error)
    }

    pub async fn search(&self, keyword: &str) -> Result<Vec<SubjectDto>, AppError> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!("{BASE}/v0/search/subjects?limit=20&offset=0");
        let body = json!({
            "keyword": keyword,
            "sort": "match",
            "filter": { "type": [2], "nsfw": false }
        });
        let page: SearchPage = self
            .request_json(reqwest::Method::POST, &url, Some(body))
            .await?;
        Ok(page.data.into_iter().map(adapt_search_hit).collect())
    }

    pub async fn get_subject(&self, id: i64) -> Result<SubjectDto, AppError> {
        let url = format!("{BASE}/v0/subjects/{id}");
        let raw: RawSubject = self
            .request_json(reqwest::Method::GET, &url, None)
            .await?;
        Ok(adapt_subject(raw))
    }

    pub async fn get_relations(&self, id: i64) -> Result<Vec<RelatedSubjectDto>, AppError> {
        let url = format!("{BASE}/v0/subjects/{id}/subjects");
        let raw: Vec<RawRelation> = self
            .request_json(reqwest::Method::GET, &url, None)
            .await?;
        Ok(raw.into_iter().map(adapt_relation).collect())
    }
}

#[derive(Debug, Deserialize)]
struct SearchPage {
    #[serde(default)]
    data: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    #[serde(default)]
    summary: String,
    date: Option<String>,
    score: Option<f64>,
    rank: Option<i64>,
    rating: Option<RawRating>,
    #[serde(default)]
    tags: Vec<RawTag>,
    images: Option<RawImages>,
    image: Option<String>,
    #[serde(default)]
    eps: i64,
    platform: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawSubject {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    #[serde(default)]
    summary: String,
    date: Option<String>,
    platform: Option<String>,
    #[serde(default)]
    eps: i64,
    #[serde(default)]
    total_episodes: i64,
    images: Option<RawImages>,
    infobox: Option<Vec<RawInfobox>>,
    rating: Option<RawRating>,
    #[serde(default)]
    tags: Vec<RawTag>,
    #[serde(default)]
    meta_tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawImages {
    large: Option<String>,
    common: Option<String>,
    medium: Option<String>,
    small: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTag {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawRelation {
    id: i64,
    #[serde(default)]
    relation: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    date: Option<String>,
    images: Option<RawImages>,
    platform: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawRating {
    rank: Option<i64>,
    total: Option<i64>,
    score: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RawInfobox {
    key: String,
    value: Value,
}

fn adapt_search_hit(hit: SearchHit) -> SubjectDto {
    let cover = hit
        .images
        .as_ref()
        .and_then(first_image)
        .or(hit.image)
        .unwrap_or_default();
    let fetched = now_iso();
    let rating = hit.rating.unwrap_or(RawRating {
        rank: hit.rank,
        total: None,
        score: hit.score,
    });
    let votes = rating.total.unwrap_or(0);
    SubjectDto {
        id: hit.id,
        name: hit.name.clone(),
        name_cn: fallback_name(&hit.name_cn, &hit.name),
        aliases: None,
        summary: hit.summary,
        cover_url: cover,
        cover_local_path: None,
        year: parse_year(hit.date.as_deref()),
        format: map_format(hit.platform.as_deref().unwrap_or("")),
        episodes: hit.eps,
        studio: String::new(),
        tags: hit
            .tags
            .into_iter()
            .filter_map(|tag| tag.name)
            .take(12)
            .collect(),
        community: CommunityDto {
            score: normalize_score(rating.score, votes),
            votes,
            rank: normalize_rank(rating.rank),
            fetched_at: Some(fetched),
            status: Some("ok".into()),
        },
    }
}

fn adapt_subject(raw: RawSubject) -> SubjectDto {
    let fetched = now_iso();
    let (aliases, studio) = parse_infobox(raw.infobox.as_deref().unwrap_or(&[]));
    let rating = raw.rating.unwrap_or(RawRating {
        rank: None,
        total: None,
        score: None,
    });
    let votes = rating.total.unwrap_or(0);
    let score = normalize_score(rating.score, votes);
    let rank = if votes == 0 {
        None
    } else {
        normalize_rank(rating.rank)
    };
    let status = if votes == 0 && score.is_none() {
        "missing"
    } else {
        "ok"
    };
    let mut tags: Vec<String> = raw
        .tags
        .into_iter()
        .filter_map(|tag| tag.name)
        .collect();
    for tag in raw.meta_tags {
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags.truncate(16);
    let cover = raw.images.as_ref().and_then(first_image).unwrap_or_default();
    let episodes = if raw.total_episodes > 0 {
        raw.total_episodes
    } else {
        raw.eps
    };
    SubjectDto {
        id: raw.id,
        name: raw.name.clone(),
        name_cn: fallback_name(&raw.name_cn, &raw.name),
        aliases: if aliases.is_empty() {
            None
        } else {
            Some(aliases)
        },
        summary: raw.summary,
        cover_url: cover,
        cover_local_path: None,
        year: parse_year(raw.date.as_deref()),
        format: map_format(raw.platform.as_deref().unwrap_or("")),
        episodes,
        studio,
        tags,
        community: CommunityDto {
            score,
            votes,
            rank,
            fetched_at: Some(fetched),
            status: Some(status.into()),
        },
    }
}

fn adapt_relation(raw: RawRelation) -> RelatedSubjectDto {
    RelatedSubjectDto {
        relation: raw.relation,
        subject: SubjectDto {
            id: raw.id,
            name: raw.name.clone(),
            name_cn: fallback_name(&raw.name_cn, &raw.name),
            aliases: None,
            summary: String::new(),
            cover_url: raw.images.as_ref().and_then(first_image).unwrap_or_default(),
            cover_local_path: None,
            year: parse_year(raw.date.as_deref()),
            format: map_format(raw.platform.as_deref().unwrap_or("")),
            episodes: 0,
            studio: String::new(),
            tags: Vec::new(),
            community: CommunityDto {
                score: None,
                votes: 0,
                rank: None,
                fetched_at: None,
                status: Some("not_fetched".into()),
            },
        },
    }
}

fn first_image(images: &RawImages) -> Option<String> {
    images
        .large
        .clone()
        .or_else(|| images.common.clone())
        .or_else(|| images.medium.clone())
        .or_else(|| images.small.clone())
}

fn fallback_name(name_cn: &str, name: &str) -> String {
    if name_cn.trim().is_empty() {
        name.to_string()
    } else {
        name_cn.to_string()
    }
}

fn parse_year(date: Option<&str>) -> i64 {
    date.and_then(|value| value.get(0..4))
        .and_then(|year| year.parse().ok())
        .unwrap_or(0)
}

fn map_format(platform: &str) -> String {
    let lower = platform.to_lowercase();
    if lower.contains("剧场") || lower.contains("movie") {
        "Movie".into()
    } else if lower.contains("ova") || lower.contains("oad") {
        "OVA".into()
    } else {
        "TV".into()
    }
}

fn normalize_score(score: Option<f64>, votes: i64) -> Option<f64> {
    match score {
        Some(value) if value > 0.0 => Some(value),
        Some(value) if value == 0.0 && votes > 0 => Some(0.0),
        _ => None,
    }
}

fn normalize_rank(rank: Option<i64>) -> Option<i64> {
    rank.filter(|value| *value > 0)
}

fn parse_infobox(items: &[RawInfobox]) -> (Vec<String>, String) {
    let mut aliases = Vec::new();
    let mut studio = String::new();
    for item in items {
        match item.key.as_str() {
            "别名" => aliases.extend(flatten_infobox_value(&item.value)),
            "动画制作" | "製作" | "制作" if studio.is_empty() => {
                studio = flatten_infobox_value(&item.value).join(" / ");
            }
            _ => {}
        }
    }
    (aliases, studio)
}

fn flatten_infobox_value(value: &Value) -> Vec<String> {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![trimmed.to_string()]
            }
        }
        Value::Array(items) => items
            .iter()
            .flat_map(|item| match item {
                Value::String(text) => vec![text.trim().to_string()],
                Value::Object(map) => map
                    .get("v")
                    .and_then(Value::as_str)
                    .map(|text| vec![text.trim().to_string()])
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .filter(|text| !text.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

pub fn is_allowed_cover_url(raw: &str) -> Result<Url, AppError> {
    let url = Url::parse(raw).map_err(|_| AppError::validation("封面地址无效"))?;
    if url.scheme() != "https" {
        return Err(AppError::validation("封面必须使用 HTTPS"));
    }
    let host = url.host_str().unwrap_or_default();
    if host == "lain.bgm.tv"
        || host == "bgm.tv"
        || host == "bangumi.tv"
        || host.ends_with(".bgm.tv")
        || host.ends_with(".bangumi.tv")
    {
        Ok(url)
    } else {
        Err(AppError::validation("封面域名不在允许列表中"))
    }
}
