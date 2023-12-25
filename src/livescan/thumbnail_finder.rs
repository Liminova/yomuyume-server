use tracing::error;
use zip::ZipArchive;

use super::blurhash::{Blurhash, BlurhashResult};
use std::{fs::File, path::PathBuf};

/// Finds a valid thumbnail image from the category dir
///
/// This requires a Blurhash instance to try encode the thumbnail
/// so that the returned image is actually a valid image
///
/// Only return the first match PathBuf to avoid wasteful iteration
pub struct CategoryThumbnailFinder<'a> {
    /// Where to look for the thumbnail image file
    pub(super) parent_dir: &'a PathBuf,
    /// Explicit name of the file to look for
    /// (with or without extension both work)
    pub(super) explicit_name: &'a Option<String>,
    /// List of implicit names of the file to look for
    /// (without extension)
    pub(super) implicit_names: &'a Vec<&'a str>,
    /// File formats to look for
    pub(super) formats: &'a Vec<String>,
    /// An instance of Blurhash to encode the thumbnail
    pub(super) blurhash: &'a Blurhash,
}

struct InternalCategoryResult {
    path: PathBuf,
    extension: String,
}

impl CategoryThumbnailFinder<'_> {
    /// The first second one is the full path to the thumbnail to be store in DB
    /// because no, I'm not adding another field to BlurhashResult
    pub fn find(&mut self) -> Option<(BlurhashResult, PathBuf)> {
        let result = self
            .both_explicit()
            .or_else(|| self.explicit_name())
            .or_else(|| self.both_implicit())
            .or_else(|| self.fuzzy())?;
        let blurhash_encoded = self.blurhash.encode(&result.path, &result.extension)?;
        Some((blurhash_encoded, result.path))
    }

    /// Explicit name with explicit extension
    fn both_explicit(&self) -> Option<InternalCategoryResult> {
        let file = self.explicit_name.as_ref()?;
        let parent_dir = self.parent_dir.to_path_buf();
        let path: PathBuf = parent_dir.join(file);
        if path.is_dir() {
            return None;
        }
        let format = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if self.formats.contains(&format) {
            return Some(InternalCategoryResult {
                path,
                extension: format,
            });
        }
        None
    }

    /// Explicit name with implicit extension
    fn explicit_name(&self) -> Option<InternalCategoryResult> {
        let file = self.explicit_name.as_ref()?;
        let path = self.parent_dir.join(file);
        if path.is_dir() {
            return None;
        }
        self.formats.iter().find_map(|format| {
            let mut path: PathBuf = path.clone();
            path.set_extension(format);
            path.is_file().then_some(InternalCategoryResult {
                path,
                extension: String::from(format),
            });
            None
        })
    }

    /// Implicit name with implicit extension
    fn both_implicit(&self) -> Option<InternalCategoryResult> {
        self.implicit_names
            .iter()
            .flat_map(|name| {
                self.formats
                    .iter()
                    .map(|format| {
                        // let mut path = self.parent_dir.clone();
                        // path.push(format!("{}.{}", name, format));
                        // path
                        let mut path = self.parent_dir.clone();
                        path.push(name);
                        path.set_extension(format);
                        InternalCategoryResult {
                            path,
                            extension: format.clone(),
                        }
                    })
                    .find(|result| result.path.is_file())
            })
            .next();
        None
    }

    /// similar to full implicit search, but instead of explicitly creating
    /// new <implicit_name>.<format> paths, we'll scan the whole parent_dir
    /// to find the closest match to <implicit_name>
    fn fuzzy(&self) -> Option<InternalCategoryResult> {
        let paths: Vec<PathBuf> = self
            .parent_dir
            .read_dir()
            .ok()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter(|entry| {
                self.formats.iter().any(|format| {
                    let file_extension = entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .to_ascii_lowercase();
                    let format = format.to_ascii_lowercase();
                    file_extension == format
                })
            })
            .map(|entry| entry.path())
            .collect();

        self.implicit_names
            .iter()
            .flat_map(|name| {
                paths.iter().find(|path| {
                    let file_stem = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .to_ascii_lowercase();
                    file_stem.contains(&name.to_ascii_lowercase())
                })
            })
            .next()
            .map(|path| {
                let extension = path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                InternalCategoryResult {
                    path: path.clone(),
                    extension,
                }
            })
    }
}

/// Find a valid thumbnail image from a Vec<BlurhashResult>
///
/// This guarantees that the returned image is actually a valid image
///
/// Only return the first match filename to avoid wasteful iteration
pub struct TitleThumbnailFinder<'a> {
    /// If a page exist in this vector, means it's actually a decode-able image
    /// and not a file/folder with an image extension.
    pub(super) blurhash_pages: &'a Vec<BlurhashResult>,
    /// Explicit name of the file to look for
    /// (with or without extension both work)
    pub(super) explicit_name: &'a Option<String>,
    /// List of implicit names of the file to look for
    /// (without extension)
    pub(super) implicit_names: &'a Vec<&'a str>,
    /// File formats to look for
    pub(super) formats: &'a Vec<String>,

    /// So that I don't have to extract the filename list in every function
    ///
    /// Insert an empty vector here if I'm creating a new instance of this struct
    pub(super) valid_pages: Vec<String>,
}

impl TitleThumbnailFinder<'_> {
    /// This will always return something if the Vec<BlurhashResult> contains
    /// something, I'm just wrapping it in an Option for consistency
    pub fn find(&mut self) -> Option<BlurhashResult> {
        self.valid_pages = self
            .blurhash_pages
            .iter()
            .map(|page| page.filename.clone())
            .collect();

        let found_page = self
            .both_explicit()
            .or_else(|| self.explicit_name())
            .or_else(|| self.implicit())
            .or_else(|| self.fuzzy());

        if let Some(found_page) = found_page {
            return self
                .blurhash_pages
                .iter()
                .find(|page| page.filename == found_page)
                .cloned();
        }
        self.blurhash_pages.first().cloned()
    }

    fn both_explicit(&self) -> Option<String> {
        let file_name = self.explicit_name.as_ref()?;
        self.valid_pages
            .contains(file_name)
            .then(|| file_name.to_string())
    }

    fn explicit_name(&self) -> Option<String> {
        let file_stem = self.explicit_name.as_ref()?;
        self.formats.iter().find_map(|format| {
            let mut path = PathBuf::from(file_stem);
            path.set_extension(format);
            let path = path.to_string_lossy().to_string();

            self.valid_pages.contains(&path).then_some(path)
        })
    }

    fn implicit(&self) -> Option<String> {
        self.implicit_names
            .iter()
            .flat_map(|name| {
                self.formats
                    .iter()
                    .map(|format| format!("{}.{}", name, format))
                    .find(|path| self.valid_pages.contains(path))
            })
            .next()
    }

    fn fuzzy(&self) -> Option<String> {
        self.implicit_names.iter().find_map(|name| {
            self.valid_pages
                .iter()
                .find(|path| {
                    path.to_ascii_lowercase()
                        .contains(&name.to_ascii_lowercase())
                })
                .map(|path| path.to_string())
        })
    }
}

/// Get called when the thumbnail field in <title>.toml changed
pub struct TitleThumbnailChangeFinder<'a> {
    pub(super) temp_dir: &'a PathBuf,
    pub(super) title_path: &'a PathBuf,
    pub(super) explicit_name: &'a Option<String>,
    pub(super) implicit_names: &'a Vec<&'a str>,
    pub(super) formats: &'a Vec<String>,
    pub(super) blurhash: &'a Blurhash,
}

impl TitleThumbnailChangeFinder<'_> {
    pub async fn find(&mut self) -> Option<BlurhashResult> {
        /* #region - read, extract zip file */
        let title_temp_dir = {
            let mut title_temp_dir = PathBuf::from(&self.temp_dir);
            title_temp_dir.push(self.title_path.file_stem()?.to_string_lossy().to_string());
            title_temp_dir
        };
        let title_file = File::open(self.title_path)
            .map_err(|e| {
                error!("error openning title: {}", e);
                e
            })
            .ok()?;
        ZipArchive::new(title_file)
            .map_err(|e| {
                error!("error reading title: {}", e);
                e
            })
            .ok()?
            .extract(&title_temp_dir)
            .map_err(|e| {
                let temp_dir = title_temp_dir.to_string_lossy();
                error!("error extracting title to {}: {}", temp_dir, e);
                e
            })
            .ok()?;
        /* #endregion */

        // Hear me out, Category thumbnail finder finds a thumbnail file given a
        // parent directyory, which is excatly what we have here. Instead of
        // re-encode the blurhash for every single pages, this only encodes the
        // first file it founds.
        CategoryThumbnailFinder {
            parent_dir: &title_temp_dir,
            explicit_name: self.explicit_name,
            implicit_names: self.implicit_names,
            formats: self.formats,
            blurhash: self.blurhash,
        }
        .find()
        // Return if found something
        .map(|(blurhash, _)| blurhash)
        // Return the first page if nothing found
        .or_else(|| {
            let first_page = title_temp_dir
                .read_dir()
                .ok()?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .filter(|entry| {
                    self.formats.iter().any(|format| {
                        let file_extension = entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                            .to_ascii_lowercase();
                        let format = format.to_ascii_lowercase();
                        file_extension == format
                    })
                })
                .map(|entry| entry.path())
                .next()?;
            self.blurhash.encode(
                &first_page,
                &first_page.extension().unwrap_or_default().to_string_lossy(),
            )
        })
    }
}
