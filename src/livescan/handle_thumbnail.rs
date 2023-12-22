use std::path::PathBuf;

use super::{blurhash::BlurhashResult, EximFnFinder, Scanner};

pub struct HandleThumbnailResult {
    pub blurhash_result: BlurhashResult,
    pub full_path: PathBuf,
}

impl Scanner {
    /// Handling every case of thumbnail naming
    ///
    /// # Returns
    ///
    /// * BlurhashResult
    /// * The full path to the thumbnail, don't use this if the thumbnail is for
    /// a title since it'll be in the temp_dir, use BlurhashResult.filename instead
    pub async fn handle_thumbnail(
        &self,
        explicit_name: Option<String>,
        parent_dir: &PathBuf,
    ) -> Option<HandleThumbnailResult> {
        let mut implicit_names: Vec<&str> = vec!["thumbnail", "cover", "_", "folder"];

        let parent_dir_stem = parent_dir
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        implicit_names.push(&parent_dir_stem);

        let mut finder = EximFnFinder {
            parent_dir,
            explicit_name,
            implicit_names: &implicit_names,
            formats: &self.image_formats,
            extension: String::from(""),
        };
        let path = finder
            .find()
            .ok_or_else(|| {
                tracing::warn!("error finding thumbnail: {:?}\n", parent_dir);
            })
            .ok()?;

        self.blurhash
            .encode(&path, &finder.extension)
            .map(|result| HandleThumbnailResult {
                blurhash_result: result,
                full_path: path,
            })
    }
}
