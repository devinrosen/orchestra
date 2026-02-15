use rusqlite::{params, Connection};

use crate::db::library_repo;
use crate::error::AppError;
use crate::models::smart_playlist::{Rule, SmartPlaylist, SmartPlaylistWithTracks};
use crate::models::track::Track;

pub fn create_smart_playlist(conn: &Connection, sp: &SmartPlaylist) -> Result<(), AppError> {
    let rule_json = serde_json::to_string(&sp.rule)
        .map_err(|e| AppError::General(e.to_string()))?;
    conn.execute(
        "INSERT INTO smart_playlists (id, name, rule_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![sp.id, sp.name, rule_json, sp.created_at, sp.updated_at],
    )?;
    Ok(())
}

pub fn get_smart_playlist(conn: &Connection, id: &str) -> Result<SmartPlaylist, AppError> {
    conn.query_row(
        "SELECT id, name, rule_json, created_at, updated_at FROM smart_playlists WHERE id = ?1",
        params![id],
        |row| {
            let rule_json: String = row.get(2)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                rule_json,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::SmartPlaylistNotFound(id.to_string()),
        other => AppError::Database(other),
    })
    .and_then(|(sp_id, name, rule_json, created_at, updated_at)| {
        let rule = serde_json::from_str(&rule_json)
            .map_err(|e| AppError::General(e.to_string()))?;
        Ok(SmartPlaylist {
            id: sp_id,
            name,
            rule,
            created_at,
            updated_at,
        })
    })
}

pub fn list_smart_playlists(conn: &Connection) -> Result<Vec<SmartPlaylist>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, rule_json, created_at, updated_at FROM smart_playlists ORDER BY updated_at DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut playlists = Vec::with_capacity(rows.len());
    for (id, name, rule_json, created_at, updated_at) in rows {
        let rule = serde_json::from_str(&rule_json)
            .map_err(|e| AppError::General(e.to_string()))?;
        playlists.push(SmartPlaylist {
            id,
            name,
            rule,
            created_at,
            updated_at,
        });
    }
    Ok(playlists)
}

pub fn update_smart_playlist(conn: &Connection, sp: &SmartPlaylist) -> Result<(), AppError> {
    let rule_json = serde_json::to_string(&sp.rule)
        .map_err(|e| AppError::General(e.to_string()))?;
    let rows = conn.execute(
        "UPDATE smart_playlists SET name = ?2, rule_json = ?3, updated_at = ?4 WHERE id = ?1",
        params![sp.id, sp.name, rule_json, sp.updated_at],
    )?;
    if rows == 0 {
        return Err(AppError::SmartPlaylistNotFound(sp.id.clone()));
    }
    Ok(())
}

pub fn delete_smart_playlist(conn: &Connection, id: &str) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM smart_playlists WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::SmartPlaylistNotFound(id.to_string()));
    }
    Ok(())
}

pub fn evaluate_smart_playlist(
    conn: &Connection,
    id: &str,
) -> Result<SmartPlaylistWithTracks, AppError> {
    let sp = get_smart_playlist(conn, id)?;
    let all_tracks = library_repo::get_all_tracks(conn)?;
    let matching: Vec<Track> = all_tracks
        .into_iter()
        .filter(|t| matches_rule(t, &sp.rule))
        .collect();
    Ok(SmartPlaylistWithTracks {
        playlist: sp,
        tracks: matching,
    })
}

fn matches_rule(track: &Track, rule: &Rule) -> bool {
    match rule {
        Rule::Condition(cond) => matches_condition(track, cond),
        Rule::Group { operator, rules } => {
            if rules.is_empty() {
                return true;
            }
            match operator.as_str() {
                "and" => rules.iter().all(|r| matches_rule(track, r)),
                "or" => rules.iter().any(|r| matches_rule(track, r)),
                _ => false,
            }
        }
    }
}

fn matches_condition(
    track: &Track,
    cond: &crate::models::smart_playlist::RuleCondition,
) -> bool {
    match cond.field.as_str() {
        // String fields
        "title" => {
            let field_val = track.title.as_deref().unwrap_or("");
            apply_string_op(field_val, &cond.op, &cond.value)
        }
        "artist" => {
            let field_val = track.artist.as_deref().unwrap_or("");
            apply_string_op(field_val, &cond.op, &cond.value)
        }
        "album_artist" => {
            let field_val = track.album_artist.as_deref().unwrap_or("");
            apply_string_op(field_val, &cond.op, &cond.value)
        }
        "album" => {
            let field_val = track.album.as_deref().unwrap_or("");
            apply_string_op(field_val, &cond.op, &cond.value)
        }
        "genre" => {
            let field_val = track.genre.as_deref().unwrap_or("");
            apply_string_op(field_val, &cond.op, &cond.value)
        }
        "format" => apply_string_op(&track.format, &cond.op, &cond.value),
        // Integer fields
        "year" => {
            let field_val = track.year.map(|v| v as f64);
            if let Ok(rule_val) = cond.value.parse::<f64>() {
                apply_numeric_op(field_val, &cond.op, rule_val)
            } else {
                false
            }
        }
        "track_number" => {
            let field_val = track.track_number.map(|v| v as f64);
            if let Ok(rule_val) = cond.value.parse::<f64>() {
                apply_numeric_op(field_val, &cond.op, rule_val)
            } else {
                false
            }
        }
        "disc_number" => {
            let field_val = track.disc_number.map(|v| v as f64);
            if let Ok(rule_val) = cond.value.parse::<f64>() {
                apply_numeric_op(field_val, &cond.op, rule_val)
            } else {
                false
            }
        }
        "bitrate" => {
            let field_val = track.bitrate.map(|v| v as f64);
            if let Ok(rule_val) = cond.value.parse::<f64>() {
                apply_numeric_op(field_val, &cond.op, rule_val)
            } else {
                false
            }
        }
        // Float fields
        "duration_secs" => {
            let field_val = track.duration_secs;
            if let Ok(rule_val) = cond.value.parse::<f64>() {
                apply_numeric_op(field_val, &cond.op, rule_val)
            } else {
                false
            }
        }
        // Boolean fields
        "has_album_art" => {
            let rule_bool = match cond.value.to_lowercase().as_str() {
                "true" | "1" => true,
                _ => false,
            };
            match cond.op.as_str() {
                "equals" => track.has_album_art == rule_bool,
                _ => false,
            }
        }
        _ => false,
    }
}

fn apply_string_op(field_val: &str, op: &str, rule_val: &str) -> bool {
    let field_lower = field_val.to_lowercase();
    let rule_lower = rule_val.to_lowercase();
    match op {
        "contains" => field_lower.contains(&rule_lower),
        "not_contains" => !field_lower.contains(&rule_lower),
        "equals" => field_lower == rule_lower,
        "not_equals" => field_lower != rule_lower,
        "starts_with" => field_lower.starts_with(&rule_lower),
        "ends_with" => field_lower.ends_with(&rule_lower),
        _ => false,
    }
}

fn apply_numeric_op(field_val: Option<f64>, op: &str, rule_val: f64) -> bool {
    match field_val {
        None => false,
        Some(v) => match op {
            "equals" => (v - rule_val).abs() < f64::EPSILON,
            "not_equals" => (v - rule_val).abs() >= f64::EPSILON,
            "greater_than" => v > rule_val,
            "less_than" => v < rule_val,
            "greater_than_or_equal" => v >= rule_val,
            "less_than_or_equal" => v <= rule_val,
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::library_repo;
    use crate::db::schema;
    use crate::models::smart_playlist::{RuleCondition, Rule};
    use crate::models::track::Track;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn make_sp(id: &str, name: &str, rule: Rule, ts: i64) -> SmartPlaylist {
        SmartPlaylist {
            id: id.to_string(),
            name: name.to_string(),
            rule,
            created_at: ts,
            updated_at: ts,
        }
    }

    fn simple_rule(field: &str, op: &str, value: &str) -> Rule {
        Rule::Condition(RuleCondition {
            field: field.to_string(),
            op: op.to_string(),
            value: value.to_string(),
        })
    }

    fn insert_track(conn: &Connection, suffix: &str, genre: Option<&str>, year: Option<i32>, title: Option<&str>) -> i64 {
        let track = Track {
            id: None,
            file_path: format!("/music/track_{}.flac", suffix),
            relative_path: format!("track_{}.flac", suffix),
            library_root: "/music".to_string(),
            title: title.map(|s| s.to_string()),
            artist: Some("Artist".to_string()),
            album_artist: None,
            album: Some("Album".to_string()),
            track_number: Some(1),
            disc_number: Some(1),
            year,
            genre: genre.map(|s| s.to_string()),
            duration_secs: Some(200.0),
            format: "flac".to_string(),
            file_size: 30_000_000,
            modified_at: 1700000000,
            hash: None,
            has_album_art: false,
            bitrate: None,
        };
        library_repo::upsert_track(conn, &track).unwrap();
        conn.query_row(
            "SELECT id FROM tracks WHERE file_path = ?1",
            params![track.file_path],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_get_smart_playlist() {
        let conn = setup_db();
        let rule = simple_rule("genre", "equals", "Jazz");
        let sp = make_sp("sp1", "Jazz Playlist", rule.clone(), 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let fetched = get_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(fetched.name, "Jazz Playlist");
        // Verify rule round-trips through JSON
        let fetched_json = serde_json::to_string(&fetched.rule).unwrap();
        let orig_json = serde_json::to_string(&rule).unwrap();
        assert_eq!(fetched_json, orig_json);
    }

    #[test]
    fn test_list_smart_playlists_ordered_by_updated_at() {
        let conn = setup_db();
        let rule = simple_rule("genre", "equals", "Rock");
        create_smart_playlist(&conn, &make_sp("sp1", "Old", rule.clone(), 1000)).unwrap();
        create_smart_playlist(&conn, &make_sp("sp2", "New", rule.clone(), 2000)).unwrap();
        let list = list_smart_playlists(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "New"); // most recent first
    }

    #[test]
    fn test_delete_smart_playlist() {
        let conn = setup_db();
        let rule = simple_rule("genre", "equals", "Rock");
        create_smart_playlist(&conn, &make_sp("sp1", "PL", rule, 1000)).unwrap();
        delete_smart_playlist(&conn, "sp1").unwrap();
        let err = get_smart_playlist(&conn, "sp1").unwrap_err();
        assert!(err.to_string().contains("Smart playlist not found"));
    }

    #[test]
    fn test_update_smart_playlist_name() {
        let conn = setup_db();
        let rule = simple_rule("genre", "equals", "Rock");
        let mut sp = make_sp("sp1", "Original", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        sp.name = "Renamed".to_string();
        sp.updated_at = 2000;
        update_smart_playlist(&conn, &sp).unwrap();
        let fetched = get_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(fetched.name, "Renamed");
        assert_eq!(fetched.updated_at, 2000);
    }

    #[test]
    fn test_update_smart_playlist_rule() {
        let conn = setup_db();
        let rule1 = simple_rule("genre", "equals", "Rock");
        let rule2 = simple_rule("genre", "equals", "Jazz");
        let mut sp = make_sp("sp1", "PL", rule1, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        sp.rule = rule2.clone();
        sp.updated_at = 2000;
        update_smart_playlist(&conn, &sp).unwrap();
        let fetched = get_smart_playlist(&conn, "sp1").unwrap();
        let fetched_json = serde_json::to_string(&fetched.rule).unwrap();
        let rule2_json = serde_json::to_string(&rule2).unwrap();
        assert_eq!(fetched_json, rule2_json);
    }

    #[test]
    fn test_evaluate_genre_equals() {
        let conn = setup_db();
        insert_track(&conn, "t1", Some("Jazz"), Some(2000), Some("Jazz Track 1"));
        insert_track(&conn, "t2", Some("Jazz"), Some(2001), Some("Jazz Track 2"));
        insert_track(&conn, "t3", Some("Rock"), Some(2002), Some("Rock Track"));
        let rule = simple_rule("genre", "equals", "Jazz");
        let sp = make_sp("sp1", "Jazz", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(result.tracks.len(), 2);
    }

    #[test]
    fn test_evaluate_year_greater_than() {
        let conn = setup_db();
        insert_track(&conn, "t1", Some("Rock"), Some(1990), Some("Old Track"));
        insert_track(&conn, "t2", Some("Rock"), Some(2000), Some("Mid Track"));
        insert_track(&conn, "t3", Some("Rock"), Some(2010), Some("New Track"));
        let rule = simple_rule("year", "greater_than", "2000");
        let sp = make_sp("sp1", "After 2000", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(result.tracks.len(), 1);
        assert_eq!(result.tracks[0].year, Some(2010));
    }

    #[test]
    fn test_evaluate_and_group() {
        let conn = setup_db();
        // Jazz 1990
        insert_track(&conn, "t1", Some("Jazz"), Some(1990), Some("Jazz Old"));
        // Jazz 2010
        insert_track(&conn, "t2", Some("Jazz"), Some(2010), Some("Jazz New"));
        // Rock 1990
        insert_track(&conn, "t3", Some("Rock"), Some(1990), Some("Rock Old"));
        // Rock 2010
        insert_track(&conn, "t4", Some("Rock"), Some(2010), Some("Rock New"));

        let rule = Rule::Group {
            operator: "and".to_string(),
            rules: vec![
                simple_rule("genre", "equals", "Jazz"),
                simple_rule("year", "greater_than", "2000"),
            ],
        };
        let sp = make_sp("sp1", "New Jazz", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(result.tracks.len(), 1);
        assert_eq!(result.tracks[0].genre.as_deref(), Some("Jazz"));
        assert_eq!(result.tracks[0].year, Some(2010));
    }

    #[test]
    fn test_evaluate_or_group() {
        let conn = setup_db();
        insert_track(&conn, "t1", Some("Jazz"), Some(2000), Some("Jazz Track"));
        insert_track(&conn, "t2", Some("Blues"), Some(2001), Some("Blues Track"));
        insert_track(&conn, "t3", Some("Rock"), Some(2002), Some("Rock Track"));

        let rule = Rule::Group {
            operator: "or".to_string(),
            rules: vec![
                simple_rule("genre", "equals", "Jazz"),
                simple_rule("genre", "equals", "Blues"),
            ],
        };
        let sp = make_sp("sp1", "Jazz or Blues", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(result.tracks.len(), 2);
    }

    #[test]
    fn test_evaluate_empty_library() {
        let conn = setup_db();
        let rule = simple_rule("genre", "equals", "Jazz");
        let sp = make_sp("sp1", "Jazz", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert!(result.tracks.is_empty());
    }

    #[test]
    fn test_evaluate_string_contains() {
        let conn = setup_db();
        insert_track(&conn, "t1", Some("Pop"), Some(2000), Some("Love Me Tender"));
        insert_track(&conn, "t2", Some("Pop"), Some(2001), Some("All You Need Is Love"));
        insert_track(&conn, "t3", Some("Pop"), Some(2002), Some("Yesterday"));

        let rule = simple_rule("title", "contains", "Love");
        let sp = make_sp("sp1", "Love Songs", rule, 1000);
        create_smart_playlist(&conn, &sp).unwrap();
        let result = evaluate_smart_playlist(&conn, "sp1").unwrap();
        assert_eq!(result.tracks.len(), 2);
    }

    #[test]
    fn test_rule_condition_not_found() {
        let conn = setup_db();
        let err = get_smart_playlist(&conn, "nonexistent").unwrap_err();
        assert!(err.to_string().contains("Smart playlist not found"));
    }
}
