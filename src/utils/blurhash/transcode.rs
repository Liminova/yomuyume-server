use image::DynamicImage;
use std::{fs, process::Command};
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct Transcoder {
    pub input_img_path: String,
    pub ffmpeg_path: Option<String>,
    pub djxl_path: Option<String>,
    pub ffmpeg_log_path: Option<String>,
}

impl Transcoder {
    #[tracing::instrument]
    pub fn transcode(&self) -> Option<DynamicImage> {
        let native_formats = [
            "bmp", "dds", "dxt", "exr", "gif", "hdr", "ico", "jpeg", "png", "pnm", "qoi", "tga",
            "tiff", "webp",
        ];

        let ext = self.input_img_path.split('.').last();
        match ext {
            None => {
                warn!("no file extension found in {}", self.input_img_path);
                None
            }
            Some(format) if native_formats.contains(&format) => {
                info!("{} is a native image format", self.input_img_path);
                image::open(&self.input_img_path)
                    .map_err(|err| {
                        error!("failed to open image file {}: {}", self.input_img_path, err);
                        err
                    })
                    .ok()
            }
            Some("jxl") => self.jpegxl(),
            _ => self.ffmpeg(),
        }
    }

    #[tracing::instrument]
    fn ffmpeg(&self) -> Option<DynamicImage> {
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
                &self.input_img_path,
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
            .map_err(|_| format!("ffmpeg failed to decode {} to png", self.input_img_path))
            .ok()?;

        if !output.status.success() {
            let decode_log = self.ffmpeg_log_path.as_ref();
            if let Some(decode_log) = decode_log {
                let err_msg = format!(
                    "failed to decode {}, ffmpeg exited with code {}",
                    self.input_img_path,
                    output.status.code().unwrap_or(-1)
                );
                let err = String::from_utf8(output.stderr)
                    .map_err(|_| error!("{}", err_msg))
                    .ok();

                let err_msg = format!(
                    "failed to decode {}, failed to write ffmpeg stderr to {}",
                    self.input_img_path, decode_log
                );
                fs::write(decode_log, err.unwrap_or_default())
                    .map_err(|_| err_msg)
                    .ok();
            }
        }

        image::load_from_memory(&output.stdout)
            .map_err(|err| {
                let err_msg = format!(
                    "{} might be decoded to png w/ ffmpeg, but failed to load from memory: {}",
                    self.input_img_path, err
                );
                error!("{}", err_msg);
            })
            .ok()
    }

    #[tracing::instrument]
    fn jpegxl(&self) -> Option<DynamicImage> {
        let djxl = self
            .djxl_path
            .as_ref()
            .ok_or(warn!(
                "djxl not found, please set the DJXL_PATH environment variable"
            ))
            .ok()?;

        let output_path = format!("{}.png", self.input_img_path);

        let output = Command::new(djxl)
            .args([self.input_img_path.clone(), output_path.clone()])
            .output()
            .map_err(|_| format!("djxl failed to decode {} to png", self.input_img_path))
            .ok()?;

        if !output.status.success() {
            error!(
                "djxl failed to transcode {} to png, djxl exited with code {}",
                self.input_img_path,
                output.status.code().unwrap_or(-1)
            );
            return None;
        }

        image::open(&output_path)
            .map_err(|err| {
                let err_msg = format!(
                    "{} might be decoded to png w/ djxl, but failed to load from disk: {}",
                    self.input_img_path, err
                );
                error!(err_msg);
            })
            .ok()
    }
}
