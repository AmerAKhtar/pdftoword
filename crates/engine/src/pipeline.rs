use layout_engine::LayoutPipeline;
use ocr_engine::{provider::OcrProvider, AdvancedRecoveryEngine};
use pdf_parser::PdfAnalysisEngine;
use renderer_docx::DocxRenderer;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use validator::{QualityValidator, ValidationConfig, ValidationReport};

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parser error: {0}")]
    Parser(#[from] pdf_parser::PdfParseError),
    #[error("Render error: {0}")]
    Render(#[from] renderer_docx::DocxRenderError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ConversionPipeline {
    pdf_parser: Arc<PdfAnalysisEngine>,
    layout_engine: Arc<LayoutPipeline>,
    recovery_engine: Arc<AdvancedRecoveryEngine>,
    validator: QualityValidator,
}

impl ConversionPipeline {
    pub fn new(ocr_provider: Box<dyn OcrProvider>) -> Result<Self, EngineError> {
        Ok(Self {
            pdf_parser: Arc::new(PdfAnalysisEngine::new()?),
            layout_engine: Arc::new(LayoutPipeline::new()),
            recovery_engine: Arc::new(AdvancedRecoveryEngine::new(ocr_provider)),
            validator: QualityValidator::new(ValidationConfig::default()),
        })
    }

    /// Executes end-to-end PDF -> IDM -> Layout -> DOCX conversion
    pub async fn convert_pdf_to_docx(
        &self,
        input_pdf_path: &Path,
        output_docx_path: &Path,
    ) -> Result<ValidationReport, EngineError> {
        // Step 1: Parse PDF to initial IDM
        let mut idm = self.pdf_parser.extract_document(input_pdf_path)?;
        let source_snapshot = idm.clone();

        // Step 2: Advanced Recovery (OCR for image-only pages)
        self.recovery_engine.process_document_recovery(&mut idm).await;

        // Step 3: Layout & Semantic Understanding
        self.layout_engine.process_document(&mut idm);

        // Step 4: Render native DOCX package
        let file = File::create(output_docx_path)?;
        let writer = BufWriter::new(file);
        DocxRenderer::render_to_stream(&idm, writer)?;

        // Step 5: Quality Validation Reporting
        let validation_report = self.validator.validate_document(&source_snapshot, &idm);
        tracing::info!(
            score = validation_report.overall_quality_score,
            "Document conversion completed successfully"
        );

        Ok(validation_report)
    }
}
