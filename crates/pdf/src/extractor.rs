use geometry::{BoundingBox, Transform};
use idm::{
    DocumentMetadata, ElementNode, FontResource, ImageNode, ImageResource, IntermediateDocument,
    PageNode, ResourceManifest, TextGroup, TextRun, VectorShapeNode,
};
use pdfium_render::prelude::*;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfParseError {
    #[error("Failed to load PDF document: {0}")]
    LoadFailed(String),
    #[error("Failed to render or extract page {0}: {1}")]
    PageExtractionFailed(usize, String),
    #[error("Font extraction error: {0}")]
    FontError(String),
}

pub struct PdfAnalysisEngine {
    pdfium: Pdfium,
}

impl PdfAnalysisEngine {
    pub fn new() -> Result<Self, PdfParseError> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .map_err(|e| PdfParseError::LoadFailed(e.to_string()))?,
        );
        Ok(Self { pdfium })
    }

    pub fn extract_document(&self, file_path: &Path) -> Result<IntermediateDocument, PdfParseError> {
        let document = self
            .pdfium
            .load_pdf_from_file(file_path, None)
            .map_err(|e| PdfParseError::LoadFailed(e.to_string()))?;

        let mut pages = Vec::new();
        let mut resource_manifest = ResourceManifest::default();

        let metadata = DocumentMetadata {
            title: document.metadata().get(PdfDocumentMetadataTagType::Title).map(|t| t.value().to_string()),
            author: document.metadata().get(PdfDocumentMetadataTagType::Author).map(|t| t.value().to_string()),
            producer: document.metadata().get(PdfDocumentMetadataTagType::Producer).map(|t| t.value().to_string()),
            creation_date: document.metadata().get(PdfDocumentMetadataTagType::CreationDate).map(|t| t.value().to_string()),
            page_count: document.pages().len() as usize,
        };

        for (index, page) in document.pages().iter().enumerate() {
            let page_node = self.extract_page(&page, index, &mut resource_manifest)?;
            pages.push(page_node);
        }

        Ok(IntermediateDocument {
            metadata,
            pages,
            resources: resource_manifest,
        })
    }

    fn extract_page(
        &self,
        page: &PdfPage,
        page_index: usize,
        resources: &mut ResourceManifest,
    ) -> Result<PageNode, PdfParseError> {
        let width = page.width().value;
        let height = page.height().value;
        let bounds = BoundingBox::new(0.0, 0.0, width, height);

        let mut elements = Vec::new();

        for object in page.objects().iter() {
            match object {
                PdfPageObject::Text(text_obj) => {
                    if let Some(text_run) = self.extract_text_run(&text_obj, height, resources) {
                        let text_group = TextGroup {
                            bounds: text_run.bounds.clone(),
                            runs: vec![text_run],
                            reading_order: elements.len(),
                            is_heading: false,
                            heading_level: None,
                        };
                        elements.push(ElementNode::TextGroup(text_group));
                    }
                }
                PdfPageObject::Image(image_obj) => {
                    if let Some(image_node) = self.extract_image(&image_obj, height, resources) {
                        elements.push(ElementNode::Image(image_node));
                    }
                }
                PdfPageObject::Path(path_obj) => {
                    if let Some(vector_node) = self.extract_path(&path_obj, height) {
                        if vector_node.bounds.width > 0.0 && vector_node.bounds.height > 0.0 {
                            elements.push(ElementNode::VectorShape(vector_node));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(PageNode {
            page_index,
            bounds,
            rotation: page.rotation().map(|r| r.as_degrees() as u16).unwrap_or(0),
            layers: Vec::new(),
            elements,
        })
    }

    fn extract_text_run(
        &self,
        text_obj: &PdfPageTextObject,
        page_height: f32,
        resources: &mut ResourceManifest,
    ) -> Option<TextRun> {
        let text = text_obj.text();
        if text.trim().is_empty() {
            return None;
        }

        let bounds = text_obj.bounds().ok()?;
        // Convert bottom-left PDF coordinate system to top-left document coordinate system
        let top_y = page_height - bounds.top.value;

        let font_name = text_obj.font().name();
        let font_id = format!("font_{}", font_name);

        if !resources.fonts.contains_key(&font_id) {
            resources.fonts.insert(
                font_id.clone(),
                FontResource {
                    family_name: font_name,
                    is_embedded: true,
                    data: Vec::new(), // Raw embedded stream payload
                },
            );
        }

        Some(TextRun {
            text,
            bounds: BoundingBox {
                x: bounds.left.value,
                y: top_y,
                width: bounds.right.value - bounds.left.value,
                height: bounds.top.value - bounds.bottom.value,
            },
            font_id,
            font_size: text_obj.unscaled_font_size().value,
            color_rgba: [0, 0, 0, 255], // Colorspace translation logic
            is_bold: false,
            is_italic: false,
            character_spacing: 0.0,
            word_spacing: 0.0,
            transform: Transform::identity(),
        })
    }

    fn extract_image(
        &self,
        image_obj: &PdfPageImageObject,
        page_height: f32,
        resources: &mut ResourceManifest,
    ) -> Option<ImageNode> {
        let bounds = image_obj.bounds().ok()?;
        let top_y = page_height - bounds.top.value;
        let resource_id = format!("img_{:p}", image_obj);

        if !resources.images.contains_key(&resource_id) {
            if let Ok(raw_image) = image_obj.get_raw_image() {
                resources.images.insert(
                    resource_id.clone(),
                    ImageResource {
                        mime_type: "image/png".to_string(),
                        width: raw_image.width() as u32,
                        height: raw_image.height() as u32,
                        raw_data: raw_image.into_bytes(),
                    },
                );
            }
        }

        Some(ImageNode {
            resource_id,
            bounds: BoundingBox {
                x: bounds.left.value,
                y: top_y,
                width: bounds.right.value - bounds.left.value,
                height: bounds.top.value - bounds.bottom.value,
            },
            transform: Transform::identity(),
            soft_mask_id: None,
            is_inline: false,
        })
    }

    fn extract_path(&self, path_obj: &PdfPagePathObject, page_height: f32) -> Option<VectorShapeNode> {
        let bounds = path_obj.bounds().ok()?;
        let top_y = page_height - bounds.top.value;

        Some(VectorShapeNode {
            bounds: BoundingBox {
                x: bounds.left.value,
                y: top_y,
                width: bounds.right.value - bounds.left.value,
                height: bounds.top.value - bounds.bottom.value,
            },
            path_commands: Vec::new(), // Iterate path segments and extract coordinates
            stroke_color: Some([0, 0, 0, 255]),
            fill_color: None,
            stroke_width: 1.0,
        })
    }
}
