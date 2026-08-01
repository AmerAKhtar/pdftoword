pub mod provider;
pub mod tesseract_provider;

use idm::{ElementNode, IntermediateDocument, PageNode, SemanticType, TextGroup};
use provider::OcrProvider;

pub struct AdvancedRecoveryEngine {
    provider: Box<dyn OcrProvider>,
    text_density_threshold: usize,
}

impl AdvancedRecoveryEngine {
    pub fn new(provider: Box<dyn OcrProvider>) -> Self {
        Self {
            provider,
            text_density_threshold: 20, // Min character threshold to consider page as non-scanned
        }
    }

    /// Evaluates every page in the Intermediate Document Model and executes OCR on scanned pages
    pub async fn process_document_recovery(&self, doc: &mut IntermediateDocument) {
        for page in doc.pages.iter_mut() {
            if Self::is_scanned_or_empty_page(page, self.text_density_threshold) {
                tracing::warn!(
                    page_index = page.page_index,
                    "Page appears scanned or lacks text streams. Initiating OCR recovery..."
                );

                if let Some(image_bytes) = Self::extract_primary_page_image(page, &doc.resources) {
                    match self
                        .provider
                        .process_image(
                            &image_bytes,
                            page.bounds.width,
                            page.bounds.height,
                            "eng",
                        )
                        .await
                    {
                        Ok(ocr_result) => {
                            Self::inject_ocr_results(page, ocr_result);
                        }
                        Err(err) => {
                            tracing::error!(
                                page_index = page.page_index,
                                error = %err,
                                "OCR recovery failed for page"
                            );
                        }
                    }
                }
            }
        }
    }

    fn is_scanned_or_empty_page(page: &PageNode, threshold: usize) -> bool {
        let mut total_chars = 0;
        for elem in &page.elements {
            if let ElementNode::TextGroup(g) = elem {
                for run in &g.runs {
                    total_chars += run.text.len();
                }
            }
        }
        total_chars < threshold
    }

    fn extract_primary_page_image(
        page: &PageNode,
        resources: &idm::ResourceManifest,
    ) -> Option<Vec<u8>> {
        // Find largest image node on the page
        for elem in &page.elements {
            if let ElementNode::Image(img_node) = elem {
                if let Some(img_res) = resources.images.get(&img_node.resource_id) {
                    return Some(img_res.raw_data.clone());
                }
            }
        }
        None
    }

    fn inject_ocr_results(page: &mut PageNode, ocr_result: provider::OcrResult) {
        for (idx, run) in ocr_result.recognized_runs.into_iter().enumerate() {
            page.elements.push(ElementNode::TextGroup(TextGroup {
                bounds: run.bounds,
                reading_order: idx,
                runs: vec![run],
                semantic_type: SemanticType::Unstructured,
            }));
        }
    }
}
