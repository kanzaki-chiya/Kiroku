use crate::models::{
    LibraryEntryDto, StatBinDto, StatDifferenceDto, StatDimensionDto, StatisticsDto,
};

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

pub fn calculate_statistics(entries: &[LibraryEntryDto]) -> StatisticsDto {
    let rated: Vec<&LibraryEntryDto> = entries
        .iter()
        .filter(|entry| entry.personal.score.is_some())
        .collect();
    let community: Vec<&LibraryEntryDto> = entries
        .iter()
        .filter(|entry| entry.subject.community.score.is_some())
        .collect();
    let mut differences: Vec<StatDifferenceDto> = entries
        .iter()
        .filter_map(|entry| {
            let personal = entry.personal.score?;
            let community_score = entry.subject.community.score?;
            Some(StatDifferenceDto {
                delta: personal - community_score,
                entry: entry.clone(),
            })
        })
        .collect();
    differences.sort_by(|a, b| {
        let abs = b.delta.abs().partial_cmp(&a.delta.abs()).unwrap_or(std::cmp::Ordering::Equal);
        if abs != std::cmp::Ordering::Equal {
            return abs;
        }
        a.entry.subject.id.cmp(&b.entry.subject.id)
    });
    let bin_defs = [
        ("0–<6", 0.0, 6.0),
        ("6–<7", 6.0, 7.0),
        ("7–<8", 7.0, 8.0),
        ("8–<9", 8.0, 9.0),
        ("9–10", 9.0, 10.1),
    ];
    let bins = bin_defs
        .into_iter()
        .map(|(label, min, max)| StatBinDto {
            label: label.into(),
            min,
            max,
            count: rated
                .iter()
                .filter(|entry| {
                    let score = entry.personal.score.unwrap();
                    score >= min && score < max
                })
                .count() as i64,
        })
        .collect();
    let mut highest_list = rated.clone();
    highest_list.sort_by(|a, b| {
        let score = b
            .personal
            .score
            .unwrap()
            .partial_cmp(&a.personal.score.unwrap())
            .unwrap_or(std::cmp::Ordering::Equal);
        if score != std::cmp::Ordering::Equal {
            return score;
        }
        a.subject.id.cmp(&b.subject.id)
    });
    let dimensions = ["story", "characters", "direction", "animation", "music"]
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            let values: Vec<f64> = entries
                .iter()
                .filter_map(|entry| entry.personal.dimensions.values()[index].1)
                .collect();
            StatDimensionDto {
                key: key.into(),
                mean: mean(&values),
                count: values.len() as i64,
            }
        })
        .collect();
    StatisticsDto {
        total: entries.len() as i64,
        rated_count: rated.len() as i64,
        community_count: community.len() as i64,
        paired_count: differences.len() as i64,
        personal_mean: mean(
            &rated
                .iter()
                .map(|entry| entry.personal.score.unwrap())
                .collect::<Vec<_>>(),
        ),
        community_mean: mean(
            &community
                .iter()
                .map(|entry| entry.subject.community.score.unwrap())
                .collect::<Vec<_>>(),
        ),
        mean_difference: mean(&differences.iter().map(|item| item.delta).collect::<Vec<_>>()),
        highest: highest_list.first().cloned().cloned(),
        differences,
        bins,
        dimensions,
    }
}

pub fn filter_entries(
    mut entries: Vec<LibraryEntryDto>,
    query: &str,
    tier: &str,
    status: &str,
    year: Option<i64>,
    sort: &str,
    direction: &str,
) -> Vec<LibraryEntryDto> {
    let query = query.trim().to_lowercase();
    entries.retain(|entry| {
        if !query.is_empty() {
            let mut haystack = format!("{}\n{}", entry.subject.name_cn, entry.subject.name);
            if let Some(aliases) = &entry.subject.aliases {
                for alias in aliases {
                    haystack.push('\n');
                    haystack.push_str(alias);
                }
            }
            for tag in &entry.subject.tags {
                haystack.push('\n');
                haystack.push_str(tag);
            }
            if !haystack.to_lowercase().contains(&query) {
                return false;
            }
        }
        if tier == "unassigned" {
            if entry.personal.tier.is_some() {
                return false;
            }
        } else if tier != "all" && tier != "" && entry.personal.tier.as_deref() != Some(tier) {
            return false;
        }
        if status != "all" && status != "" && entry.personal.status != status {
            return false;
        }
        if let Some(year) = year {
            if entry.subject.year != year {
                return false;
            }
        }
        true
    });
    let dir: i32 = if direction == "asc" { 1 } else { -1 };
    entries.sort_by(|a, b| {
        let result = match sort {
            "personal" => compare_opt_f64(a.personal.score, b.personal.score, dir),
            "community" => compare_opt_f64(
                a.subject.community.score,
                b.subject.community.score,
                dir,
            ),
            "title" => a
                .subject
                .name_cn
                .cmp(&b.subject.name_cn)
                .then_with(|| a.subject.id.cmp(&b.subject.id)),
            _ => a
                .personal
                .updated_at
                .cmp(&b.personal.updated_at)
                .then_with(|| a.subject.id.cmp(&b.subject.id)),
        };
        if sort == "title" || sort == "updated" || sort.is_empty() {
            if dir < 0 {
                result.reverse()
            } else {
                result
            }
        } else {
            result.then_with(|| a.subject.id.cmp(&b.subject.id))
        }
    });
    entries
}

fn compare_opt_f64(a: Option<f64>, b: Option<f64>, direction: i32) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => {
            let ord = a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal);
            if direction < 0 {
                ord.reverse()
            } else {
                ord
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CommunityDto, DimensionsDto, PersonalRecordDto, SubjectDto};

    fn entry(id: i64, personal: Option<f64>, community: Option<f64>) -> LibraryEntryDto {
        LibraryEntryDto {
            local_id: id,
            subject: SubjectDto {
                id,
                name: format!("Name {id}"),
                name_cn: format!("作品{id}"),
                aliases: None,
                summary: String::new(),
                cover_url: String::new(),
                cover_local_path: None,
                year: 2020,
                format: "TV".into(),
                episodes: 12,
                studio: "Studio".into(),
                tags: vec![],
                community: CommunityDto {
                    score: community,
                    votes: 100,
                    rank: None,
                    fetched_at: None,
                    status: None,
                },
            },
            personal: PersonalRecordDto {
                score: personal,
                tier: None,
                status: "completed".into(),
                progress: None,
                dimensions: DimensionsDto {
                    story: None,
                    characters: None,
                    direction: None,
                    animation: None,
                    music: None,
                },
                review: String::new(),
                subject_id: id,
                created_at: "2025-01-01T00:00:00.000Z".into(),
                updated_at: "2025-01-01T00:00:00.000Z".into(),
                version: 1,
            },
        }
    }

    #[test]
    fn matches_frontend_fixture() {
        let mut entries = vec![entry(1, Some(8.0), Some(7.0)), entry(2, Some(6.0), Some(8.0)), entry(3, None, Some(9.0))];
        entries[0].personal.dimensions = DimensionsDto {
            story: Some(4.0),
            characters: Some(0.0),
            direction: None,
            animation: Some(5.0),
            music: None,
        };
        entries[1].personal.dimensions = DimensionsDto {
            story: None,
            characters: Some(4.0),
            direction: None,
            animation: Some(4.0),
            music: None,
        };
        entries[1].personal.status = "watching".into();
        entries[2].personal.dimensions = DimensionsDto {
            story: Some(3.0),
            characters: None,
            direction: None,
            animation: Some(4.5),
            music: Some(0.5),
        };
        entries[2].personal.status = "planned".into();
        let stats = calculate_statistics(&entries);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.rated_count, 2);
        assert_eq!(stats.community_count, 3);
        assert_eq!(stats.paired_count, 2);
        assert_eq!(stats.personal_mean, Some(7.0));
        assert_eq!(stats.community_mean, Some(8.0));
        assert_eq!(stats.mean_difference, Some(-0.5));
        assert_eq!(stats.highest.as_ref().unwrap().personal.score, Some(8.0));
        assert_eq!(stats.differences.iter().map(|d| d.delta).collect::<Vec<_>>(), vec![-2.0, 1.0]);
        assert_eq!(stats.bins.iter().map(|b| b.count).collect::<Vec<_>>(), vec![0, 1, 0, 1, 0]);
        assert_eq!(serde_json::to_value(&stats).unwrap()["dimensions"], serde_json::json!([
            {"key":"story","mean":3.5,"count":2},
            {"key":"characters","mean":2.0,"count":2},
            {"key":"direction","mean":null,"count":0},
            {"key":"animation","mean":4.5,"count":3},
            {"key":"music","mean":0.5,"count":1}
        ]));
    }

    #[test]
    fn empty_dimensions_keep_fixed_order() {
        for entries in [vec![], vec![entry(1, Some(8.0), Some(7.0))]] {
            let stats = calculate_statistics(&entries);
            assert_eq!(
                stats
                    .dimensions
                    .iter()
                    .map(|d| (d.key.as_str(), d.mean, d.count))
                    .collect::<Vec<_>>(),
                vec![
                    ("story", None, 0),
                    ("characters", None, 0),
                    ("direction", None, 0),
                    ("animation", None, 0),
                    ("music", None, 0)
                ]
            );
        }
    }

    #[test]
    fn dimension_boundary_scores_without_total() {
        let mut entries = vec![entry(1, None, None), entry(2, None, None)];
        entries[0].personal.dimensions = DimensionsDto {
            story: Some(0.0),
            characters: Some(0.5),
            direction: None,
            animation: Some(5.0),
            music: None,
        };
        entries[1].personal.dimensions = DimensionsDto {
            story: None,
            characters: Some(1.0),
            direction: None,
            animation: None,
            music: None,
        };
        let stats = calculate_statistics(&entries);
        assert_eq!(serde_json::to_value(&stats).unwrap()["dimensions"], serde_json::json!([
            {"key":"story","mean":0.0,"count":1},
            {"key":"characters","mean":0.75,"count":2},
            {"key":"direction","mean":null,"count":0},
            {"key":"animation","mean":5.0,"count":1},
            {"key":"music","mean":null,"count":0}
        ]));
        assert_eq!(stats.rated_count, 0);
    }

    #[test]
    fn null_scores_stay_last_in_both_directions() {
        let entries = vec![entry(1, None, None), entry(2, Some(8.0), None), entry(3, Some(6.0), None)];
        let desc = filter_entries(entries.clone(), "", "all", "all", None, "personal", "desc");
        assert_eq!(desc.iter().map(|e| e.subject.id).collect::<Vec<_>>(), vec![2, 3, 1]);
        let asc = filter_entries(entries, "", "all", "all", None, "personal", "asc");
        assert_eq!(asc.iter().map(|e| e.subject.id).collect::<Vec<_>>(), vec![3, 2, 1]);
    }
}
