use crate::utils::BlurhashResult;
use crate::{utils::Blurhash, AppState};
use async_recursion::async_recursion;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs::File, io::Write};
use tracing::warn;
use zip::ZipArchive;

/// Scanning all categories dirs inside library
async fn scan_library(library_path: &str) -> Vec<PathBuf> {
    let mut categories = Vec::new();
    let mut entries = match tokio::fs::read_dir(library_path).await {
        Ok(entries) => entries,
        Err(e) => {
            warn!("rrror reading category dir: {}\n", e);
            return Vec::new();
        }
    };
    'next_category: while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
        let path = entry.path();
        if path.is_dir() {
            match path {
                p if p.to_str().unwrap_or_default().is_empty() => continue 'next_category,
                p => categories.push(p),
            }
        }
    }
    categories
}

/// Scanning all title inside a category
#[async_recursion]
async fn scan_category(item_dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut entries = match tokio::fs::read_dir(item_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            warn!("error reading item file: {}\n", e);
            return Vec::new();
        }
    };

    'next_title: while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(scan_category(&path).await);
        }
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if ext == "zip" {
            match path {
                p if p.to_str().unwrap_or_default().is_empty() => continue 'next_title,
                p => files.push(p),
            }
        }
    }
    files
}

pub struct ValidPage {
    pub path: PathBuf,
    pub ext: String,
}

/// Scanning all pages inside an extracted title dir
async fn scan_extracted(temp_dir: &PathBuf, image_formats: &[&str]) -> Vec<ValidPage> {
    let mut images = Vec::new();
    let mut entries = match tokio::fs::read_dir(temp_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            warn!("error reading temp dir: {}\n", e);
            return Vec::new();
        }
    };
    'next_image: while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
        let path = entry.path();
        if path.is_dir() {
            continue 'next_image;
        }
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if image_formats.contains(&ext) {
            let ext: String = ext.to_string();
            images.push(ValidPage { path, ext });
        }
    }
    images
}

pub async fn scanner(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = app_state.env.temp_dir.clone();

    let image_formats: Vec<&str> = vec![
        "jxl", "avif", "webp", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif",
    ];

    let blurhash = Blurhash {
        ffmpeg_path: app_state.env.ffmpeg_path.clone(),
        djxl_path: app_state.env.djxl_path.clone(),
        ffmpeg_log_path: app_state.env.ffmpeg_log_path.clone(),
    };

    let categories = scan_library(&app_state.env.library_path).await;
    for category in categories {
        println!("category: {}", category.to_string_lossy());
        let titles = scan_category(&category).await;
        'next_title: for title in titles {
            let file = match File::open(&title) {
                Ok(file) => file,
                Err(e) => {
                    warn!("error openning title: {}\n", e);
                    continue 'next_title;
                }
            };
            let mut zip_archive = match ZipArchive::new(file) {
                Ok(zip_archive) => zip_archive,
                Err(e) => {
                    warn!("error reading title: {}\n", e);
                    continue 'next_title;
                }
            };

            let mut title_temp_dir: PathBuf = PathBuf::from(&temp_dir);
            title_temp_dir.push(&title);

            match zip_archive.extract(&title_temp_dir) {
                Ok(_) => (),
                Err(e) => {
                    warn!("error extracting zip: {}\n", e);
                    continue 'next_title;
                }
            };

            let pages = scan_extracted(&title_temp_dir, &image_formats).await;

            let blurhashes: Vec<BlurhashResult> = pages
                .par_iter()
                .filter_map(|page| blurhash.encode(&page.path.to_string_lossy(), &page.ext))
                .collect();

            match tokio::fs::remove_dir_all(&title_temp_dir).await {
                Ok(_) => (),
                Err(e) => {
                    warn!("error removing temp dir: {}\n", e);
                    continue 'next_title;
                }
            };

            let mut f = File::create("./blurhash.txt").unwrap();
            for blurhash in blurhashes {
                match f.write_fmt(format_args!("{}\n", blurhash.blurhash)) {
                    Ok(_) => (),
                    Err(e) => {
                        warn!("error writing blurhash: {}\n", e);
                        continue 'next_title;
                    }
                }
            }
        }
    }

    Ok(())
}
