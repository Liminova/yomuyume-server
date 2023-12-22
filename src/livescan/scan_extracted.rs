use std::path::PathBuf;

pub struct ScannedPage {
    /// Path in the temporary directory, not the original path
    pub path: PathBuf,
    /// The file name of the page
    pub name: String,
    /// The file extension of the page
    pub ext: String,
    pub blurhash: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub description: Option<String>,
}

impl Default for ScannedPage {
    fn default() -> Self {
        ScannedPage {
            path: PathBuf::new(),
            name: String::new(),
            ext: String::new(),
            blurhash: None,
            width: None,
            height: None,
            description: None,
        }
    }
}

/// Scanning all pages inside an extracted title dir
pub async fn scan_extracted(temp_dir: &PathBuf, image_formats: &[String]) -> Vec<ScannedPage> {
    let mut images = Vec::new();
    let mut entries = match tokio::fs::read_dir(temp_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("error reading temp dir: {}\n", e);
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
            .unwrap_or_default()
            .to_lowercase();
        if image_formats.contains(&ext) {
            let ext: String = ext.to_string();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string();
            images.push(ScannedPage {
                path,
                name,
                ext,
                ..Default::default()
            });
        }
    }
    images
}
