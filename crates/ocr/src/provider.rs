use async_trait::async_trait;
use idm::TextRun;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("OCR Processing Failed: {0}")]
    ProcessingFailed(String),
    #[error("Language pack unavailable: {0}")]
    LanguageMissing(String),
    #[error("Invalid image buffer")]
    InvalidImageBuffer,
}

pub struct OcrResult {
    pub recognized_runs: Vec<TextRun>,
    pub confidence_score: f32,
    pub orientation_angle: u16,
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    /// Returns the provider name (e.g., "TesseractOCR", "CloudVisionPlugin")
    fn name(&self) -> &'static str;

    /// Processes a raw image buffer and returns extracted text runs with geometric coordinates
    async fn process_image(
        &self,
        image_bytes: &[u8],
        page_width: f32,
        page_height: f32,
        language: &str,
    ) -> Result<OcrResult, OcrError>;
}
