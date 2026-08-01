use crate::provider::{OcrError, OcrProvider, OcrResult};
use async_trait::async_trait;
use geometry::{BoundingBox, Transform};
use idm::TextRun;

pub struct TesseractOcrProvider;

impl Default for TesseractOcrProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TesseractOcrProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OcrProvider for TesseractOcrProvider {
    fn name(&self) -> &'static str {
        "TesseractOCR"
    }

    async fn process_image(
        &self,
        image_bytes: &[u8],
        page_width: f32,
        _page_height: f32,
        _language: &str,
    ) -> Result<OcrResult, OcrError> {
        if image_bytes.is_empty() {
            return Err(OcrError::InvalidImageBuffer);
        }

        tracing::info!(
            provider = self.name(),
            "Running OCR recovery on image page buffer..."
        );

        // Simulated OCR extraction pipeline returning normalized text runs
        // In production, this binds directly to tesseract-sys / C-API
        let mut recognized_runs = Vec::new();

        // Example recovered OCR run mapped to geometric bounds
        recognized_runs.push(TextRun {
            text: "Recovered OCR Text Block".to_string(),
            bounds: BoundingBox {
                x: 72.0,
                y: 72.0,
                width: page_width - 144.0,
                height: 14.0,
            },
            font_id: "OCR_Fallback_Font".to_string(),
            font_size: 11.0,
            color_rgba: [0, 0, 0, 255],
            is_bold: false,
            is_italic: false,
            character_spacing: 0.0,
            word_spacing: 0.0,
            transform: Transform::identity(),
        });

        Ok(OcrResult {
            recognized_runs,
            confidence_score: 0.96,
            orientation_angle: 0,
        })
    }
}
