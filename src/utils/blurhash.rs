use blurhash::encode;
use image::DynamicImage;
use image::{imageops::FilterType::Gaussian, GenericImageView};
use std::{fs, process::Command};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct Blurhash {
    pub ffmpeg_path: Option<String>,
    pub djxl_path: Option<String>,
    pub ffmpeg_log_path: Option<String>,
}

pub struct BlurhashResult {
    pub blurhash: String,
    pub width: u32,
    pub height: u32,
}

impl Blurhash {
    #[tracing::instrument]
    pub fn encode(&self, input_img_path: &str, format: &str) -> Option<BlurhashResult> {
        let img = self
            .transcode(input_img_path, format)?
            .resize(100, 100, Gaussian);

        let (width, height) = img.dimensions();
        let (x, y) = {
            if width < height {
                (3, (3.0 * height as f32 / width as f32).round() as u32)
            } else {
                ((3.0 * width as f32 / height as f32).round() as u32, 3)
            }
        };

        let encoded = encode(x, y, width, height, &img.to_rgba8().into_vec())
            .map_err(|_| error!("failed to encode to blurhash"))
            .ok()?
            .to_string();

        debug!("encoded to blurhash");

        Some(BlurhashResult {
            blurhash: encoded,
            width,
            height,
        })
    }

    #[tracing::instrument]
    pub fn transcode(&self, in_file: &str, format: &str) -> Option<DynamicImage> {
        let native_formats = ["bmp", "gif", "ico", "jpeg", "png", "tiff", "webp"];
        match format {
            format if native_formats.contains(&format) => {
                debug!("using native decoder");
                image::open(in_file)
                    .map_err(|err| {
                        error!("failed to open: {}", err);
                        err
                    })
                    .ok()
            }
            "jxl" => {
                debug!("using djxl");
                self.jpegxl(in_file)
            }
            _ => {
                debug!("using ffmpeg");
                self.ffmpeg(in_file)
            }
        }
    }

    #[tracing::instrument]
    fn ffmpeg(&self, in_file: &str) -> Option<DynamicImage> {
        let ffmpeg = match self.ffmpeg_path {
            Some(ref path) => path.clone(),
            None => {
                warn!("ffmpeg not found, please set the FFMPEG_PATH environment variable");
                return None;
            }
        };

        let output = Command::new(ffmpeg)
            .args([
                "-i",
                in_file,
                "-vf",
                "scale=100:-1",
                "-y",
                "-f",
                "image2pipe",
                "-vcodec",
                "png",
                "-",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            let decode_log = self.ffmpeg_log_path.as_ref();
            if let Some(decode_log) = decode_log {
                let err_msg = format!(
                    "ffmpeg failed with code {}",
                    output.status.code().unwrap_or(-1)
                );
                let err = String::from_utf8(output.stderr)
                    .map_err(|_| error!("{}", err_msg))
                    .ok();

                let err_msg = format!("failed to write ffmpeg decode log to {}", decode_log);
                fs::write(decode_log, err.unwrap_or_default())
                    .map_err(|_| err_msg)
                    .ok();
            }
        }

        image::load_from_memory(&output.stdout)
            .map_err(|err| {
                let err_msg = format!(
                    "might be decoded, but failed to load from ffmpeg stdout: {}",
                    err
                );
                error!("{}", err_msg);
            })
            .ok()
    }

    #[tracing::instrument]
    fn jpegxl(&self, in_file: &str) -> Option<DynamicImage> {
        let djxl = self
            .djxl_path
            .as_ref()
            .ok_or_else(|| warn!("djxl not found, please set the DJXL_PATH environment variable"))
            .ok()?;

        let output_path = format!("{}.png", in_file);

        let output = Command::new(djxl)
            .args([in_file, &output_path])
            .output()
            .ok()?;

        if !output.status.success() {
            error!(
                "djxl failed with code {}",
                output.status.code().unwrap_or(-1)
            );
            return None;
        }

        image::open(&output_path)
            .map_err(|err| {
                let err_msg = format!(
                    "might be decoded, but failed to load from temp file: {}",
                    err
                );
                error!(err_msg);
            })
            .ok()
    }
}
