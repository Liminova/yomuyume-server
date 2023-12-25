pub mod version;

pub fn thumbnail_filestem<'a>() -> Vec<&'a str> {
    vec!["thumbnail", "cover", "_", "folder"]
}

pub fn extended_img_formats<'a>() -> Vec<&'a str> {
    vec![
        "jxl", "avif", "webp", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif",
    ]
}

pub fn native_img_formats<'a>() -> Vec<&'a str> {
    vec!["png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif"]
}
