pub fn screenshot_to_device(x: f64, y: f64, scale: f64) -> Result<(f64, f64), super::HelperError> {
    if scale <= 0.0 {
        return Err(super::HelperError::failed("scale must be positive"));
    }
    Ok((x / scale, y / scale))
}

pub fn device_to_screenshot(x: f64, y: f64, scale: f64) -> Result<(f64, f64), super::HelperError> {
    if scale <= 0.0 {
        return Err(super::HelperError::failed("scale must be positive"));
    }
    Ok((x * scale, y * scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let (x, y) = screenshot_to_device(200.0, 100.0, 2.0).unwrap();
        assert_eq!((x, y), (100.0, 50.0));
        let back = device_to_screenshot(x, y, 2.0).unwrap();
        assert_eq!(back, (200.0, 100.0));
    }
}
