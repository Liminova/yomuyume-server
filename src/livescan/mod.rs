mod blurhash;
mod handle_category;
mod handle_thumbnail;
mod handle_title;
mod scan_category;
mod scan_extracted;
mod scan_library;
mod scanner;
use std::path::{Path, PathBuf};

use blurhash::{Blurhash, BlurhashResult};

// use self::scan_library::ScannedCategory;
// use crate::AppState;
// use blurhash::{Blurhash, BlurhashResult};
// use scan_library::scan_library;
// use std::path::PathBuf;
// use std::sync::Arc;
pub use scanner::Scanner;

/// Explicit/Implicit file name finder
pub struct EximFnFinder<'a> {
    parent_dir: &'a PathBuf,
    explicit_name: Option<String>,
    implicit_names: &'a Vec<&'a str>,
    formats: &'a Vec<String>,
    extension: String,
}

impl EximFnFinder<'_> {
    pub fn find(&mut self) -> Option<PathBuf> {
        let result = self
            .explicit_name_explicit_ext()
            .or_else(|| self.explicit_name_implicit_ext())
            .or_else(|| self.implicit())
            .or_else(|| self.fuzzy());
        self.extension = match &result {
            Some(path) => Self::get_ext(path),
            None => String::new(),
        };
        result
    }

    fn explicit_name_explicit_ext(&self) -> Option<PathBuf> {
        let explicit_name = match &self.explicit_name {
            Some(name) => name,
            None => return None,
        };
        let path: PathBuf = self.parent_dir.join(explicit_name);
        if path.is_dir() {
            return None;
        }
        if self.formats.contains(&Self::get_ext(&path)) {
            return Some(path);
        }
        None
    }

    fn explicit_name_implicit_ext(&self) -> Option<PathBuf> {
        let explicit_name = match &self.explicit_name {
            Some(name) => name,
            None => return None,
        };
        let path = self.parent_dir.join(explicit_name);
        if path.is_dir() {
            return None;
        }
        for format in self.formats {
            let mut path = path.clone();
            path.set_extension(format);
            if path.is_file() {
                return Some(path);
            }
        }
        None
    }

    fn implicit(&self) -> Option<PathBuf> {
        self.implicit_names
            .iter()
            .flat_map(|name| {
                self.formats
                    .iter()
                    .map(|format| {
                        let mut path = self.parent_dir.clone();
                        path.push(format!("{}.{}", name, format));
                        path
                    })
                    .find(|path| path.is_file())
                // .next()
            })
            .next();
        None
    }

    /// similar to full implicit search, but instead of explicitly creating
    /// new <implicit_name>.<format> paths, we'll scan the whole parent_dir
    /// to find the closest match to <implicit_name>
    fn fuzzy(&self) -> Option<PathBuf> {
        let paths: Vec<PathBuf> = self
            .parent_dir
            .read_dir()
            .ok()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter(|entry| {
                self.formats.iter().any(|format| {
                    let path_ext_lower = entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .to_ascii_lowercase();
                    let format_lower = format.to_ascii_lowercase();
                    path_ext_lower == format_lower
                })
            })
            .map(|entry| entry.path())
            .collect();

        self.implicit_names
            .iter()
            .flat_map(|name| {
                paths
                    .iter()
                    .find(|path| path.to_string_lossy().to_ascii_lowercase().contains(name))
            })
            .next()
            .map(|path| path.to_path_buf())
    }

    fn get_ext(path: &Path) -> String {
        path.extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_lowercase()
    }
}
