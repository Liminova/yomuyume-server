use crate::config::Config;
use blurhash::encode;
use image::{imageops::FilterType::Gaussian, GenericImageView};
use tracing::error;

use self::transcode::Transcoder;

mod transcode;

pub fn _blurhash(config: Config, input_img_path: String) -> Option<String> {
    let input_img_path = input_img_path.to_string();
    let transcoder = Transcoder {
        ffmpeg_path: config.ffmpeg_path,
        djxl_path: config.djxl_path,
        ffmpeg_log_path: config.decode_log,
        input_img_path: input_img_path.clone(),
    };

    let img = transcoder
        .transcode()
        .ok_or(error!("failed to transcode image"))
        .ok()?
        .resize(100, 100, Gaussian);

    let (width, height) = img.dimensions();
    let (x, y) = {
        if width < height {
            (3, (3.0 * height as f32 / width as f32).round() as u32)
        } else {
            ((3.0 * width as f32 / height as f32).round() as u32, 3)
        }
    };

    Some(
        encode(x, y, width, height, &img.to_rgba8().into_vec())
            .map_err(|_| error!("failed to encode {} to blurhash", input_img_path.clone()))
            .ok()?
            .to_string(),
    )
}
