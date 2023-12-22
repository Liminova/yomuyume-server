use super::{
    scan_library::{scan_library, ScannedCategory},
    Blurhash,
};
use crate::AppState;
use std::{path::PathBuf, sync::Arc};

pub struct Scanner {
    pub(super) app_state: Arc<AppState>,
    pub(super) temp_dir: PathBuf,
    pub(super) image_formats: Vec<String>,
    pub(super) blurhash: Blurhash,
    pub(super) categories: Vec<ScannedCategory>,
}

impl Scanner {
    pub async fn default(app_state: Arc<AppState>) -> Self {
        let app_state = Arc::clone(&app_state);
        let temp_dir = PathBuf::from(&app_state.env.temp_dir.clone());
        let ffmpeg_path = app_state.env.ffmpeg_path.clone();
        let djxl_path = app_state.env.djxl_path.clone();
        let ffmpeg_log_path = app_state.env.ffmpeg_log_path.clone();
        let categories = scan_library(&app_state.env.library_path).await;
        Self {
            app_state,
            temp_dir,
            image_formats: vec![
                "jxl", "avif", "webp", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            blurhash: Blurhash {
                ffmpeg_path,
                djxl_path,
                ffmpeg_log_path,
            },
            categories,
        }
    }
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        for category in &self.categories {
            let _ = self.handle_category(category).await;
        }
        Ok(())
    }
}
