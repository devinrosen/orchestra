use base64::Engine;
use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::tag::{Accessor, TagExt};
use std::path::Path;

use crate::error::AppError;
use crate::models::track::{AlbumArt, TrackMetadataUpdate};

pub fn write_metadata(path: &Path, update: &TrackMetadataUpdate) -> Result<(), AppError> {
    let mut tagged_file = lofty::read_from_path(path)
        .map_err(|e| AppError::Metadata(format!("{}: {}", path.display(), e)))?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(t) => t,
        None => {
            // Insert a tag appropriate for the format
            let tag_type = tagged_file
                .primary_tag_type();
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    // Set or clear each field
    match &update.title {
        Some(v) if !v.is_empty() => tag.set_title(v.clone()),
        Some(_) => tag.remove_title(),
        None => {}
    }

    match &update.artist {
        Some(v) if !v.is_empty() => tag.set_artist(v.clone()),
        Some(_) => tag.remove_artist(),
        None => {}
    }

    match &update.album_artist {
        Some(v) if !v.is_empty() => {
            tag.insert_text(lofty::tag::ItemKey::AlbumArtist, v.clone());
        }
        Some(_) => {
            tag.remove_key(&lofty::tag::ItemKey::AlbumArtist);
        }
        None => {}
    }

    match &update.album {
        Some(v) if !v.is_empty() => tag.set_album(v.clone()),
        Some(_) => tag.remove_album(),
        None => {}
    }

    match update.track_number {
        Some(n) if n > 0 => tag.set_track(n),
        Some(_) => tag.remove_track(),
        None => {}
    }

    match update.disc_number {
        Some(n) if n > 0 => tag.set_disk(n),
        Some(_) => tag.remove_disk(),
        None => {}
    }

    match update.year {
        Some(y) if y > 0 => tag.set_year(y as u32),
        Some(_) => tag.remove_year(),
        None => {}
    }

    match &update.genre {
        Some(v) if !v.is_empty() => tag.set_genre(v.clone()),
        Some(_) => tag.remove_genre(),
        None => {}
    }

    tag.save_to_path(path, lofty::config::WriteOptions::default())
        .map_err(|e| AppError::Metadata(format!("Failed to write {}: {}", path.display(), e)))?;

    Ok(())
}

pub fn extract_artwork(path: &Path) -> Result<Option<AlbumArt>, AppError> {
    let tagged_file = lofty::read_from_path(path)
        .map_err(|e| AppError::Metadata(format!("{}: {}", path.display(), e)))?;

    let tag = match tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
        Some(t) => t,
        None => return Ok(None),
    };

    // Try front cover first, then any picture
    let picture = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first());

    match picture {
        Some(pic) => {
            let data = base64::engine::general_purpose::STANDARD.encode(pic.data());
            let mime_type = pic.mime_type().map(|m| m.to_string()).unwrap_or_else(|| "image/jpeg".to_string());
            Ok(Some(AlbumArt { data, mime_type }))
        }
        None => Ok(None),
    }
}
