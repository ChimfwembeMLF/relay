use base64::{engine::general_purpose::STANDARD, Engine};
use qrcode::QrCode;

use crate::error::AppError;

pub fn generate_qr_png_base64(url: &str) -> Result<String, AppError> {
    let code = QrCode::new(url.as_bytes())
        .map_err(|e| AppError::Internal(format!("QR generation failed: {e}")))?;

    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(256, 256)
        .build();

    let mut bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);
    image::DynamicImage::ImageLuma8(image).write_to(
        &mut cursor,
        image::ImageFormat::Png,
    )
    .map_err(|e| AppError::Internal(format!("QR PNG encode failed: {e}")))?;

    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
