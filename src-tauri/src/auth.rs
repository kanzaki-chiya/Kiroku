use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// 客户端固定走自有域名（kiroku.koxia.cn → zgo 反代 → Supabase 新加坡项目），
/// 不硬编码 *.supabase.co——后端怎么换对客户端都只是 DNS/代理变更。
pub const CLOUD_BASE: &str = "https://kiroku.koxia.cn";
/// dev 项目的 publishable key：设计上即可公开，权限由 RLS/函数边界兜底。
/// 正式项目上线时换正式 key。
pub const PUBLISHABLE_KEY: &str = "sb_publishable_nWERiKummbqEtLcTgWoO5Q_xkV4YxAr";

const SESSION_FILE: &str = "session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_unix: i64,
    pub user_id: String,
    pub email: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
    user: TokenUser,
}

#[derive(Deserialize)]
struct TokenUser {
    id: String,
    email: Option<String>,
}

fn session_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SESSION_FILE)
}

pub fn load_session(data_dir: &Path) -> Option<Session> {
    let raw = std::fs::read_to_string(session_path(data_dir)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_session(data_dir: &Path, session: &Session) -> Result<(), AppError> {
    std::fs::write(session_path(data_dir), serde_json::to_string(session)?)?;
    Ok(())
}

pub fn clear_session(data_dir: &Path) {
    let _ = std::fs::remove_file(session_path(data_dir));
}

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

fn to_session(resp: TokenResponse, fallback_email: &str) -> Session {
    Session {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at_unix: now_unix() + resp.expires_in.unwrap_or(3600) - 60,
        user_id: resp.user.id,
        email: resp.user.email.unwrap_or_else(|| fallback_email.into()),
    }
}

fn auth_error(code: &str, msg: &str) -> AppError {
    AppError::new(
        "AUTH",
        match code {
            "invalid_credentials" => "邮箱或密码不正确".to_string(),
            "email_not_confirmed" => "邮箱未验证，请先完成验证".to_string(),
            "user_already_exists" => "该邮箱已注册，请直接登录".to_string(),
            "weak_password" => "密码强度不足，请使用更复杂的密码".to_string(),
            "signup_disabled" => "当前未开放注册".to_string(),
            "over_request_rate_limit" | "over_email_send_rate_limit" => {
                "请求过于频繁，请稍后再试".to_string()
            }
            _ => msg.to_string(),
        },
    )
}

async fn parse_token_response(resp: reqwest::Response) -> Result<TokenResponse, AppError> {
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        let msg = body
            .get("msg")
            .or_else(|| body.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("登录失败");
        let code = body
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return Err(auth_error(code, msg));
    }
    serde_json::from_value(body).map_err(AppError::from)
}

pub async fn login(
    client: &reqwest::Client,
    data_dir: &Path,
    email: &str,
    password: &str,
) -> Result<Session, AppError> {
    let resp = client
        .post(format!("{CLOUD_BASE}/auth/v1/token?grant_type=password"))
        .header("apikey", PUBLISHABLE_KEY)
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|_| AppError::network("无法连接云服务"))?;
    let session = to_session(parse_token_response(resp).await?, email);
    save_session(data_dir, &session)?;
    Ok(session)
}

#[derive(Debug)]
pub enum SignupOutcome {
    /// dev 项目开自动确认：注册即得会话。
    Session(Session),
    /// 需要邮箱验证：会话未建立，提示用户查收验证邮件。
    ConfirmEmail,
}

/// 邮箱+密码注册。Supabase Auth：开 Confirm email 时返回 user 对象（无 token），
/// 关时直接返回会话 token——两种响应按 access_token 字段区分。
pub async fn signup(
    client: &reqwest::Client,
    data_dir: &Path,
    email: &str,
    password: &str,
) -> Result<SignupOutcome, AppError> {
    let resp = client
        .post(format!("{CLOUD_BASE}/auth/v1/signup"))
        .header("apikey", PUBLISHABLE_KEY)
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|_| AppError::network("无法连接云服务"))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        let msg = body
            .get("msg")
            .or_else(|| body.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("注册失败");
        let code = body
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return Err(auth_error(code, msg));
    }
    if body.get("access_token").and_then(|v| v.as_str()).is_some() {
        let token: TokenResponse = serde_json::from_value(body).map_err(AppError::from)?;
        let session = to_session(token, email);
        save_session(data_dir, &session)?;
        return Ok(SignupOutcome::Session(session));
    }
    Ok(SignupOutcome::ConfirmEmail)
}

pub async fn refresh(
    client: &reqwest::Client,
    data_dir: &Path,
    session: &Session,
) -> Result<Session, AppError> {
    let resp = client
        .post(format!(
            "{CLOUD_BASE}/auth/v1/token?grant_type=refresh_token"
        ))
        .header("apikey", PUBLISHABLE_KEY)
        .json(&serde_json::json!({ "refresh_token": session.refresh_token }))
        .send()
        .await
        .map_err(|_| AppError::network("无法连接云服务"))?;
    if !resp.status().is_success() {
        clear_session(data_dir);
        return Err(AppError::new("AUTH_EXPIRED", "登录已过期，请重新登录"));
    }
    let next = to_session(parse_token_response(resp).await?, &session.email);
    save_session(data_dir, &next)?;
    Ok(next)
}

/// 返回一个保证未过期的会话；过期则刷新。
pub async fn ensure_fresh(
    client: &reqwest::Client,
    data_dir: &Path,
    session: Session,
) -> Result<Session, AppError> {
    if session.expires_at_unix - 60 > now_unix() {
        return Ok(session);
    }
    refresh(client, data_dir, &session).await
}

pub async fn logout(client: &reqwest::Client, data_dir: &Path, session: Option<Session>) {
    if let Some(s) = session {
        let _ = client
            .post(format!("{CLOUD_BASE}/auth/v1/logout"))
            .header("apikey", PUBLISHABLE_KEY)
            .bearer_auth(&s.access_token)
            .send()
            .await;
    }
    clear_session(data_dir);
}

#[derive(Debug)]
pub enum RpcError {
    Auth(String),
    Transport(AppError),
}

/// 调 kiroku_* RPC。401 由调用方决定刷新重试；其余原样透传 jsonb 结果。
pub async fn rpc(
    client: &reqwest::Client,
    session: &Session,
    function: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    let resp = client
        .post(format!("{CLOUD_BASE}/rest/v1/rpc/{function}"))
        .header("apikey", PUBLISHABLE_KEY)
        .bearer_auth(&session.access_token)
        .json(&params)
        .send()
        .await
        .map_err(|err| RpcError::Transport(AppError::network(format!("云端请求失败：{err}"))))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if status.as_u16() == 401 {
        return Err(RpcError::Auth(
            body.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("鉴权失败")
                .to_string(),
        ));
    }
    if !status.is_success() {
        let message = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("云端返回错误");
        return Err(RpcError::Transport(AppError::new(
            "CLOUD",
            message.to_string(),
        )));
    }
    Ok(body)
}
