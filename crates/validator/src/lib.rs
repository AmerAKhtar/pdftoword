pub mod engine;
pub mod report;

pub use engine::QualityValidator;
pub use report::{DeviationSeverity, DeviationType, LayoutDeviation, ValidationConfig, ValidationReport};

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::BoundingBox;
    use idm::{
        DocumentMetadata, ElementNode, IntermediateDocument, PageNode, ResourceManifest,
        SemanticType, TextGroup, TextRun,
    };

    fn create_dummy_doc(text: &str, x: f32, y: f32) -> IntermediateDocument {
        IntermediateDocument {
            metadata: DocumentMetadata {
                title: None,
                author: None,
                producer: None,
                creation_date: None,
                page_count: 1,
            },
            pages: vec![PageNode {
                page_index: 0,
                bounds: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 612.0,
                    height: 792.0,
                },
                rotation: 0,
                layers: vec![],
                elements: vec![ElementNode::TextGroup(TextGroup {
                    bounds: BoundingBox {
                        x,
                        y,
                        width: 100.0,
                        height: 12.0,
                    },
                    reading_order: 0,
                    runs: vec![TextRun {
                        text: text.to_string(),
                        bounds: BoundingBox {
                            x,
                            y,
                            width: 100.0,
                            height: 12.0,
                        },
                        font_id: "Helvetica".into(),
                        font_size: 12.0,
                        color_rgba: [0, 0, 0, 255],
                        is_bold: false,
                        is_italic: false,
                        character_spacing: 0.0,
                        word_spacing: 0.0,
                        transform: Default::default(),
                    }],
                    semantic_type: SemanticType::Paragraph,
                })],
            }],
            resources: ResourceManifest::default(),
        }
    }

    #[test]
    fn test_perfect_match_validation() {
        let src = create_dummy_doc("Hello World", 50.0, 50.0);
        let recon = create_dummy_doc("Hello World", 50.0, 50.0);

        let validator = QualityValidator::default();
        let report = validator.validate_document(&src, &recon);

        assert_eq!(report.overall_quality_score, 1.0);
        assert_eq!(report.text_fidelity_score, 1.0);
        assert!(report.deviations.is_empty());
    }

    #[test]
    fn test_spatial_drift_detection() {
        let src = create_dummy_doc("Hello World", 50.0, 50.0);
        let recon = create_dummy_doc("Hello World", 55.0, 50.0); // 5.0 pt drift

        let validator = QualityValidator::default(); // max_spatial_drift_pt = 2.0
        let report = validator.validate_document(&src, &recon);

        assert_eq!(report.deviations.len(), 1);
        assert_eq!(report.deviations[0].deviation_type, DeviationType::BoundingBoxDrift);
        assert_eq!(report.deviations[0].severity, DeviationSeverity::Low);
    }
}
