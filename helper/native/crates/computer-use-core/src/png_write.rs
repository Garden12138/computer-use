use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use super::HelperError;

pub fn write_png_rgb(path: &Path, width: u32, height: u32, rgb: &[u8]) -> Result<(), HelperError> {
    let expected = width as usize * height as usize * 3;
    if rgb.len() != expected {
        return Err(HelperError::failed(format!(
            "rgb length {} != {}",
            rgb.len(),
            expected
        )));
    }
    let file = File::create(path).map_err(|e| HelperError::failed(e.to_string()))?;
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| HelperError::failed(e.to_string()))?;
    writer
        .write_image_data(rgb)
        .map_err(|e| HelperError::failed(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn writes_png() {
        let dir = env::temp_dir();
        let path = dir.join("computer-use-core-test.png");
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        write_png_rgb(&path, 2, 2, &rgb).unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(path);
    }
}
