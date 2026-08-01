pub mod semantic_classifier;
pub mod text_aggregator;

use idm::IntermediateDocument;
use semantic_classifier::SemanticClassifier;
use text_aggregator::TextAggregator;

pub struct LayoutPipeline {
    aggregator: TextAggregator,
}

impl Default for LayoutPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutPipeline {
    pub fn new() -> Self {
        Self {
            aggregator: TextAggregator::new(),
        }
    }

    pub fn process_document(&self, doc: &mut IntermediateDocument) {
        // Calculate statistical body font size mode across the document
        let body_font_size_mode = self.calculate_body_font_mode(doc);

        for page in doc.pages.iter_mut() {
            // Step 1: Cluster raw PDF character fragments into cohesive lines/paragraphs
            self.aggregator.aggregate_page_text(page);

            // Step 2: Assign semantic meaning (Headings, Headers, Footers, Lists, Paragraphs)
            SemanticClassifier::classify_page_semantics(page, body_font_size_mode);
        }
    }

    fn calculate_body_font_mode(&self, doc: &IntermediateDocument) -> f32 {
        let mut font_sizes = Vec::new();
        for page in &doc.pages {
            for elem in &page.elements {
                if let idm::ElementNode::TextGroup(g) = elem {
                    for run in &g.runs {
                        font_sizes.push((run.font_size * 10.0) as u32);
                    }
                }
            }
        }

        if font_sizes.is_empty() {
            return 11.0; // Default fallback body font size
        }

        // Find statistical mode
        let mut counts = std::collections::HashMap::new();
        for sz in font_sizes {
            *counts.entry(sz).or_insert(0) += 1;
        }

        let mode_sz = counts.into_iter().max_by_key(|&(_, count)| count).map(|(sz, _)| sz).unwrap_or(110);
        (mode_sz as f32) / 10.0
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn infer_structure(doc: &mut IntermediateDocument) {
        let pipeline = LayoutPipeline::new();
        pipeline.process_document(doc);
    }
}
