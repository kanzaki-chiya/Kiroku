use crate::error::AppError;
use crate::models::PersonalDraftDto;

const STATUSES: [&str; 3] = ["completed", "watching", "planned"];

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn score_to_tenths(value: f64, label: &str) -> Result<i64, AppError> {
    if !value.is_finite() || !(0.0..=10.0).contains(&value) {
        return Err(AppError::validation(format!(
            "{label}必须是 0–10 之间的数字"
        )));
    }
    Ok((value * 10.0).round() as i64)
}

pub fn dimension_to_tenths(value: f64) -> Result<i64, AppError> {
    if !value.is_finite() || !(0.5..=5.0).contains(&value) {
        return Err(AppError::validation(
            "维度评分必须是 0.5–5 之间的半星步进",
        ));
    }
    let tenths = (value * 10.0).round() as i64;
    if tenths % 5 != 0 {
        return Err(AppError::validation(
            "维度评分必须是 0.5–5 之间的半星步进",
        ));
    }
    Ok(tenths)
}

pub fn tenths_to_score(value: i64) -> f64 {
    value as f64 / 10.0
}

pub fn validate_draft(draft: &PersonalDraftDto, known_tiers: &[String]) -> Result<(), AppError> {
    if let Some(score) = draft.score {
        score_to_tenths(score, "个人评分")?;
    }
    if let Some(tier) = &draft.tier {
        if !known_tiers.iter().any(|name| name == tier) {
            return Err(AppError::validation("无效的分档"));
        }
    }
    if !STATUSES.contains(&draft.status.as_str()) {
        return Err(AppError::validation("无效的观看状态"));
    }
    if let Some(progress) = draft.progress {
        if progress < 0 {
            return Err(AppError::validation("观看进度不能为负数"));
        }
    }
    for (_key, value) in draft.dimensions.values() {
        if let Some(score) = value {
            dimension_to_tenths(score)?;
        }
    }
    if draft.review.len() > 5000 {
        return Err(AppError::validation("短评不能超过 5000 字"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DimensionsDto;

    fn draft() -> PersonalDraftDto {
        PersonalDraftDto {
            score: Some(8.6),
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
            review: "ok".into(),
        }
    }

    #[test]
    fn accepts_valid_draft() {
        validate_draft(&draft(), &["S".into(), "A".into()]).unwrap();
    }

    #[test]
    fn rejects_out_of_range_score() {
        let mut value = draft();
        value.score = Some(10.5);
        assert!(validate_draft(&value, &["A".into()]).is_err());
    }

    #[test]
    fn rejects_unknown_tier() {
        let value = draft();
        assert!(validate_draft(&value, &["S".into()]).is_err());
    }

    #[test]
    fn dimension_to_tenths_accepts_half_stars() {
        assert_eq!(dimension_to_tenths(4.5).unwrap(), 45);
    }

    #[test]
    fn dimension_to_tenths_rejects_invalid() {
        assert!(dimension_to_tenths(4.3).is_err());
        assert!(dimension_to_tenths(6.0).is_err());
        assert!(dimension_to_tenths(0.0).is_err());
    }
}
