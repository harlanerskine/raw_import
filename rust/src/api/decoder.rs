use image::ImageEncoder;
use rawler::decoders::RawDecodeParams;
use rawler::imgop::develop::RawDevelop;
use rawler::RawFile;
use std::io::Cursor;
use std::panic;

/// Metadata extracted from a RAW file before full decode.
pub struct RawImageInfo {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Camera make (e.g. "Canon", "Nikon", "Sony").
    pub make: String,
    /// Camera model (e.g. "EOS R5", "Z 9", "A7R V").
    pub model: String,
    /// ISO sensitivity.
    pub iso: u32,
    /// Whether the format is supported for full decode.
    pub supported: bool,
    /// Detected RAW format name (e.g. "DNG", "CR3", "NEF", "ARW").
    pub format: String,
}

/// Result of decoding a RAW file to RGB pixels.
pub struct RawDecodeResult {
    /// Decoded pixel data as 8-bit sRGB (RGBRGBRGB..., no alpha).
    pub pixels: Vec<u8>,
    /// Image width after demosaic and orientation.
    pub width: u32,
    /// Image height after demosaic and orientation.
    pub height: u32,
    /// Camera make.
    pub make: String,
    /// Camera model.
    pub model: String,
    /// ISO sensitivity.
    pub iso: u32,
}

/// Probe a RAW file to extract metadata without full decode.
///
/// This reads only the file header and EXIF data — fast enough for
/// gallery thumbnails and format detection.
#[flutter_rust_bridge::frb(sync)]
pub fn probe_raw(file_bytes: Vec<u8>) -> Result<RawImageInfo, String> {
    // Wrap in catch_unwind to prevent rawler panics from crashing the app
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        probe_raw_inner(file_bytes)
    }));

    match result {
        Ok(inner) => inner,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during RAW probe".to_string()
            };
            Err(format!("RAW probe panic: {}", msg))
        }
    }
}

fn probe_raw_inner(file_bytes: Vec<u8>) -> Result<RawImageInfo, String> {
    let mut rawfile = RawFile::from(Cursor::new(file_bytes.clone()));
    let params = RawDecodeParams::default();

    let decoder = rawler::get_decoder(&mut rawfile)
        .map_err(|e| format!("No decoder found: {}", e))?;

    let metadata = decoder
        .raw_metadata(&mut rawfile, params.clone())
        .map_err(|e| format!("Failed to read metadata: {}", e))?;

    let iso = metadata.exif.iso_speed.unwrap_or(0) as u32;

    // Get dimensions from a quick raw_image decode with dummy=true
    // (reads dimensions without full pixel decode on many formats)
    let (width, height) = match decoder.raw_image(&mut rawfile, params, true) {
        Ok(raw) => (raw.width as u32, raw.height as u32),
        Err(_) => (0, 0),
    };

    Ok(RawImageInfo {
        width,
        height,
        make: metadata.make,
        model: metadata.model,
        iso,
        supported: true,
        format: detect_format(&file_bytes),
    })
}

/// Decode a RAW file to 8-bit sRGB pixel data.
///
/// Performs full Bayer demosaic, white balance, color space conversion,
/// and gamma correction. Returns packed RGB bytes (3 bytes per pixel,
/// row-major, no padding).
#[flutter_rust_bridge::frb(sync)]
pub fn decode_raw(file_bytes: Vec<u8>) -> Result<RawDecodeResult, String> {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        decode_raw_inner(file_bytes)
    }));

    match result {
        Ok(inner) => inner,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during RAW decode".to_string()
            };
            Err(format!("RAW decode panic: {}", msg))
        }
    }
}

fn decode_raw_inner(file_bytes: Vec<u8>) -> Result<RawDecodeResult, String> {
    let mut rawfile = RawFile::from(Cursor::new(file_bytes));
    let params = RawDecodeParams::default();

    let decoder = rawler::get_decoder(&mut rawfile)
        .map_err(|e| format!("No decoder found for this RAW format: {}", e))?;

    let rawimage = decoder
        .raw_image(&mut rawfile, params.clone(), false)
        .map_err(|e| format!("RAW decode failed: {}", e))?;

    let metadata = decoder
        .raw_metadata(&mut rawfile, params)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;

    let iso = metadata.exif.iso_speed.unwrap_or(0) as u32;

    // Develop the RAW image: demosaic, white balance, color calibration, sRGB
    let pipeline = RawDevelop::default();
    let intermediate = pipeline
        .develop_intermediate(&rawimage)
        .map_err(|e| format!("RAW development failed: {}", e))?;

    // Extract dimensions and convert to 8-bit sRGB packed RGB
    let (width, height, pixels) = intermediate_to_rgb8(intermediate);

    Ok(RawDecodeResult {
        pixels,
        width,
        height,
        make: metadata.make,
        model: metadata.model,
        iso,
    })
}

/// Decode a RAW file and return it as JPEG bytes.
///
/// Convenience function that decodes, then JPEG-compresses the result.
/// Useful when the caller wants to feed the image into Flutter's
/// `instantiateImageCodec` which handles JPEG natively.
#[flutter_rust_bridge::frb(sync)]
pub fn decode_raw_to_jpeg(
    file_bytes: Vec<u8>,
    quality: u8,
) -> Result<Vec<u8>, String> {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        decode_raw_to_jpeg_inner(file_bytes, quality)
    }));

    match result {
        Ok(inner) => inner,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during RAW-to-JPEG decode".to_string()
            };
            Err(format!("RAW decode panic: {}", msg))
        }
    }
}

fn decode_raw_to_jpeg_inner(file_bytes: Vec<u8>, quality: u8) -> Result<Vec<u8>, String> {
    let result = decode_raw_inner(file_bytes)?;

    let img = image::RgbImage::from_raw(result.width, result.height, result.pixels)
        .ok_or("Failed to create image buffer from decoded pixels")?;

    let mut jpeg_buf = Cursor::new(Vec::new());
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, quality)
        .write_image(
            img.as_raw(),
            result.width,
            result.height,
            image::ColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG encoding failed: {}", e))?;

    Ok(jpeg_buf.into_inner())
}

/// Extract the embedded JPEG preview/thumbnail from a RAW file.
///
/// Most RAW files contain an embedded full-size JPEG preview that was
/// generated by the camera. This is much faster than full decode and
/// is suitable for gallery display.
#[flutter_rust_bridge::frb(sync)]
pub fn extract_preview(file_bytes: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        extract_preview_inner(file_bytes)
    }));

    match result {
        Ok(inner) => inner,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during preview extraction".to_string()
            };
            Err(format!("Preview extraction panic: {}", msg))
        }
    }
}

fn extract_preview_inner(file_bytes: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
    let mut rawfile = RawFile::from(Cursor::new(file_bytes));

    let decoder = rawler::get_decoder(&mut rawfile)
        .map_err(|e| format!("No decoder found: {}", e))?;

    // Try full_image first (camera-generated full-size preview), then
    // fall back to preview_image, then thumbnail_image.
    // Each call borrows rawfile mutably, so we chain with sequential lets.
    let dynamic_image = decoder.full_image(&mut rawfile).unwrap_or(None);
    let dynamic_image = dynamic_image.or(decoder.preview_image(&mut rawfile).unwrap_or(None));
    let dynamic_image = dynamic_image.or(decoder.thumbnail_image(&mut rawfile).unwrap_or(None));

    match dynamic_image {
        Some(img) => {
            use image::GenericImageView;
            let rgb = img.to_rgb8();
            let (width, height) = rgb.dimensions();

            let mut jpeg_buf = Cursor::new(Vec::new());
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 90)
                .write_image(
                    rgb.as_raw(),
                    width,
                    height,
                    image::ColorType::Rgb8,
                )
                .map_err(|e| format!("JPEG encoding of preview failed: {}", e))?;

            Ok(Some(jpeg_buf.into_inner()))
        }
        None => Ok(None),
    }
}

/// Check if a file's bytes look like a supported RAW format.
#[flutter_rust_bridge::frb(sync)]
pub fn is_supported_raw(file_bytes: Vec<u8>) -> bool {
    panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut rawfile = RawFile::from(Cursor::new(file_bytes));
        rawler::get_decoder(&mut rawfile).is_ok()
    }))
    .unwrap_or(false)
}

// ── Internal helpers ─────────────────────────────────────────────────────

/// Convert a rawler Intermediate to packed 8-bit sRGB RGB bytes.
///
/// Returns `(width, height, rgb_bytes)`.
///
/// `Color2D<f32, N>` stores data as `Vec<[f32; N]>` — one array per pixel.
/// `Pix2D<f32>` (PixF32) stores data as `Vec<f32>` — one scalar per pixel.
fn intermediate_to_rgb8(
    intermediate: rawler::imgop::develop::Intermediate,
) -> (u32, u32, Vec<u8>) {
    use rawler::imgop::develop::Intermediate;

    match intermediate {
        Intermediate::ThreeColor(pixels) => {
            let w = pixels.width as u32;
            let h = pixels.height as u32;
            let mut rgb = Vec::with_capacity(pixels.data.len() * 3);

            for px in &pixels.data {
                rgb.push((px[0].clamp(0.0, 1.0) * 255.0) as u8);
                rgb.push((px[1].clamp(0.0, 1.0) * 255.0) as u8);
                rgb.push((px[2].clamp(0.0, 1.0) * 255.0) as u8);
            }

            (w, h, rgb)
        }
        Intermediate::Monochrome(pixels) => {
            let w = pixels.width as u32;
            let h = pixels.height as u32;
            let mut rgb = Vec::with_capacity(pixels.data.len() * 3);

            for &v in &pixels.data {
                let byte = (v.clamp(0.0, 1.0) * 255.0) as u8;
                rgb.push(byte);
                rgb.push(byte);
                rgb.push(byte);
            }

            (w, h, rgb)
        }
        Intermediate::FourColor(pixels) => {
            let w = pixels.width as u32;
            let h = pixels.height as u32;
            let mut rgb = Vec::with_capacity(pixels.data.len() * 3);

            // RGBE or similar — take first three channels
            for px in &pixels.data {
                rgb.push((px[0].clamp(0.0, 1.0) * 255.0) as u8);
                rgb.push((px[1].clamp(0.0, 1.0) * 255.0) as u8);
                rgb.push((px[2].clamp(0.0, 1.0) * 255.0) as u8);
            }

            (w, h, rgb)
        }
    }
}

/// Detect RAW format from file magic bytes.
fn detect_format(bytes: &[u8]) -> String {
    if bytes.len() < 12 {
        return "unknown".to_string();
    }

    // TIFF-based formats (DNG, NEF, ARW, CR2)
    if (bytes[0] == 0x49 && bytes[1] == 0x49 && bytes[2] == 0x2A && bytes[3] == 0x00)
        || (bytes[0] == 0x4D && bytes[1] == 0x4D && bytes[2] == 0x00 && bytes[3] == 0x2A)
    {
        return "DNG/TIFF-RAW".to_string();
    }

    // Canon CR3 (ISO Base Media File Format container)
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        if &bytes[8..12] == b"crx " {
            return "CR3".to_string();
        }
    }

    // Fujifilm RAF
    if bytes.len() >= 16 && &bytes[0..16] == b"FUJIFILMCCD-RAW " {
        return "RAF".to_string();
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_format tests ──────────────────────────────────────────

    #[test]
    fn detect_format_tiff_little_endian() {
        // II (little-endian) + 0x002A = TIFF magic
        let mut bytes = vec![0x49, 0x49, 0x2A, 0x00];
        bytes.resize(12, 0);
        assert_eq!(detect_format(&bytes), "DNG/TIFF-RAW");
    }

    #[test]
    fn detect_format_tiff_big_endian() {
        // MM (big-endian) + 0x002A = TIFF magic
        let mut bytes = vec![0x4D, 0x4D, 0x00, 0x2A];
        bytes.resize(12, 0);
        assert_eq!(detect_format(&bytes), "DNG/TIFF-RAW");
    }

    #[test]
    fn detect_format_cr3() {
        // ISO BMFF: size(4) + "ftyp" + "crx "
        let mut bytes = vec![0x00, 0x00, 0x00, 0x20]; // size
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"crx ");
        assert_eq!(detect_format(&bytes), "CR3");
    }

    #[test]
    fn detect_format_raf() {
        let bytes = b"FUJIFILMCCD-RAW extra data here";
        assert_eq!(detect_format(bytes), "RAF");
    }

    #[test]
    fn detect_format_unknown_bytes() {
        let bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0]; // JPEG magic
        assert_eq!(detect_format(&bytes), "unknown");
    }

    #[test]
    fn detect_format_too_short() {
        assert_eq!(detect_format(&[0x49, 0x49]), "unknown");
        assert_eq!(detect_format(&[]), "unknown");
        assert_eq!(detect_format(&[0x00; 11]), "unknown");
    }

    #[test]
    fn detect_format_exactly_12_bytes() {
        let mut bytes = vec![0x49, 0x49, 0x2A, 0x00];
        bytes.resize(12, 0);
        assert_eq!(detect_format(&bytes), "DNG/TIFF-RAW");
    }

    #[test]
    fn detect_format_ftyp_but_not_crx() {
        // ftyp container but not Canon CR3 (e.g. HEIF)
        let mut bytes = vec![0x00, 0x00, 0x00, 0x20];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"heic");
        assert_eq!(detect_format(&bytes), "unknown");
    }

    #[test]
    fn detect_format_raf_exact_16_bytes() {
        let bytes = b"FUJIFILMCCD-RAW ";
        assert_eq!(bytes.len(), 16);
        assert_eq!(detect_format(bytes), "RAF");
    }

    #[test]
    fn detect_format_raf_too_short() {
        // Only 15 bytes of the RAF magic
        let bytes = b"FUJIFILMCCD-RAW";
        assert_eq!(bytes.len(), 15);
        assert_eq!(detect_format(bytes), "unknown");
    }

    // ── intermediate_to_rgb8 tests ───────────────────────────────────

    #[test]
    fn intermediate_three_color_basic() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Color2D;

        let pixels = Color2D::new_with(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], 2, 1);

        let (w, h, rgb) = intermediate_to_rgb8(Intermediate::ThreeColor(pixels));
        assert_eq!(w, 2);
        assert_eq!(h, 1);
        assert_eq!(rgb.len(), 6); // 2 pixels × 3 channels
        assert_eq!(rgb[0], 255); // R
        assert_eq!(rgb[1], 0);   // G
        assert_eq!(rgb[2], 0);   // B
        assert_eq!(rgb[3], 0);   // R
        assert_eq!(rgb[4], 255); // G
        assert_eq!(rgb[5], 0);   // B
    }

    #[test]
    fn intermediate_three_color_clamps_values() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Color2D;

        let pixels = Color2D::new_with(vec![[-0.5, 1.5, 0.5]], 1, 1);

        let (_, _, rgb) = intermediate_to_rgb8(Intermediate::ThreeColor(pixels));
        assert_eq!(rgb[0], 0);   // clamped from -0.5
        assert_eq!(rgb[1], 255); // clamped from 1.5
        assert_eq!(rgb[2], 127); // 0.5 * 255 ≈ 127
    }

    #[test]
    fn intermediate_monochrome_basic() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Pix2D;

        let pixels = Pix2D::new_with(vec![0.0, 0.5, 1.0], 3, 1);

        let (w, h, rgb) = intermediate_to_rgb8(Intermediate::Monochrome(pixels));
        assert_eq!(w, 3);
        assert_eq!(h, 1);
        assert_eq!(rgb.len(), 9); // 3 pixels × 3 channels (R=G=B)
        // Black pixel
        assert_eq!(rgb[0], 0);
        assert_eq!(rgb[1], 0);
        assert_eq!(rgb[2], 0);
        // Mid-grey pixel
        assert_eq!(rgb[3], 127);
        assert_eq!(rgb[4], 127);
        assert_eq!(rgb[5], 127);
        // White pixel
        assert_eq!(rgb[6], 255);
        assert_eq!(rgb[7], 255);
        assert_eq!(rgb[8], 255);
    }

    #[test]
    fn intermediate_monochrome_clamps_values() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Pix2D;

        let pixels = Pix2D::new_with(vec![-1.0, 2.0], 2, 1);

        let (_, _, rgb) = intermediate_to_rgb8(Intermediate::Monochrome(pixels));
        assert_eq!(rgb[0], 0);   // clamped from -1.0
        assert_eq!(rgb[3], 255); // clamped from 2.0
    }

    #[test]
    fn intermediate_four_color_takes_first_three_channels() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Color2D;

        let pixels = Color2D::new_with(vec![[0.2, 0.4, 0.6, 0.8]], 1, 1);

        let (w, h, rgb) = intermediate_to_rgb8(Intermediate::FourColor(pixels));
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(rgb.len(), 3); // only RGB, no alpha
        assert_eq!(rgb[0], 51);  // 0.2 * 255 = 51
        assert_eq!(rgb[1], 102); // 0.4 * 255 = 102
        assert_eq!(rgb[2], 153); // 0.6 * 255 = 153
    }

    #[test]
    fn intermediate_empty_image() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Color2D;

        let pixels: Color2D<f32, 3> = Color2D::new_with(vec![], 0, 0);

        let (w, h, rgb) = intermediate_to_rgb8(Intermediate::ThreeColor(pixels));
        assert_eq!(w, 0);
        assert_eq!(h, 0);
        assert_eq!(rgb.len(), 0);
    }

    #[test]
    fn intermediate_multi_row_dimensions() {
        use rawler::imgop::develop::Intermediate;
        use rawler::pixarray::Color2D;

        let pixels = Color2D::new_with(vec![[0.5; 3]; 6], 3, 2);

        let (w, h, rgb) = intermediate_to_rgb8(Intermediate::ThreeColor(pixels));
        assert_eq!(w, 3);
        assert_eq!(h, 2);
        assert_eq!(rgb.len(), 18); // 6 pixels × 3 channels
    }

    // ── RawImageInfo struct tests ────────────────────────────────────

    #[test]
    fn raw_image_info_fields() {
        let info = RawImageInfo {
            width: 6000,
            height: 4000,
            make: "Canon".to_string(),
            model: "EOS R5".to_string(),
            iso: 400,
            supported: true,
            format: "CR3".to_string(),
        };

        assert_eq!(info.width, 6000);
        assert_eq!(info.height, 4000);
        assert_eq!(info.make, "Canon");
        assert_eq!(info.model, "EOS R5");
        assert_eq!(info.iso, 400);
        assert!(info.supported);
        assert_eq!(info.format, "CR3");
    }

    // ── RawDecodeResult struct tests ─────────────────────────────────

    #[test]
    fn raw_decode_result_fields() {
        let result = RawDecodeResult {
            pixels: vec![255, 128, 0],
            width: 1,
            height: 1,
            make: "Nikon".to_string(),
            model: "Z 9".to_string(),
            iso: 64,
        };

        assert_eq!(result.pixels.len(), 3);
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 1);
        assert_eq!(result.make, "Nikon");
        assert_eq!(result.model, "Z 9");
        assert_eq!(result.iso, 64);
    }
}
