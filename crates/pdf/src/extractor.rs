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
            title: document.get_metadata_value(PdfDocumentMetadataTag::Title),
            author: document.get_metadata_value(PdfDocumentMetadataTag::Author),
            producer: document.get_metadata_value(PdfDocumentMetadataTag::Producer),
            creation_date: document.get_metadata_value(PdfDocumentMetadataTag::CreationDate),
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
        let page_width = page.width().value;
        let page_height = page.height().value;

        let bounds = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: page_width,
            height: page_height,
        };

        let mut elements = Vec::new();

        // Iterate through all objects embedded on the page
        for object in page.objects().iter() {
            match object.object_type() {
                PdfPageObjectType::Text => {
                    if let Some(text_object) = object.as_text_object() {
                        if let Some(run) = self.extract_text_run(&text_object, page_height, resources) {
                            elements.push(ElementNode::TextGroup(TextGroup {
                                bounds: run.bounds,
                                reading_order: elements.len(),
                                runs: vec![run],
                                semantic_type: idm::SemanticType::Unstructured,
                            }));
                        }
                    }
                }
                PdfPageObjectType::Image => {
                    if let Some(image_object) = object.as_image_object() {
                        if let Some(image_node) = self.extract_image(&image_object, page_height, resources) {
                            elements.push(ElementNode::Image(image_node));
                        }
                    }
                }
                PdfPageObjectType::Path => {
                    if let Some(path_object) = object.as_path_object() {
                        if let Some(vector_node) = self.extract_path(&path_object, page_height) {
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
            rotation: page.rotation().map(|r| r.as_degrees()).unwrap_or(0),
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
        let font_id = format!("font_{}", font_name.replace(" ", "_").replace("+", ""));

        if !resources.fonts.contains_key(&font_id) {
            // Try to extract embedded font data
            let font_data = text_obj.font().embedded_data().unwrap_or_default();
            resources.fonts.insert(
                font_id.clone(),
                FontResource {
                    family_name: font_name.clone(),
                    is_embedded: !font_data.is_empty(),
                    data: font_data,
                },
            );
        }

        // Extract color information
        let color_rgba = self.extract_color(text_obj);
        
        // Detect font style flags
        let is_bold = text_obj.font().is_bold() || font_name.to_lowercase().contains("bold");
        let is_italic = text_obj.font().is_italic() || font_name.to_lowercase().contains("italic");

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
            color_rgba,
            is_bold,
            is_italic,
            character_spacing: text_obj.character_spacing().value,
            word_spacing: text_obj.word_spacing().value,
            transform: Transform::identity(),
        })
    }

    fn extract_color(&self, text_obj: &PdfPageTextObject) -> [u8; 4] {
        // Extract fill color from text object
        if let Some(colorspace) = text_obj.fill_colorspace() {
            match colorspace {
                PdfColorSpace::DeviceGray => {
                    if let Some(gray) = text_obj.fill_color() {
                        let g = (gray[0] * 255.0) as u8;
                        return [g, g, g, 255];
                    }
                }
                PdfColorSpace::DeviceRgb => {
                    if let Some(rgb) = text_obj.fill_color() {
                        let r = (rgb[0] * 255.0) as u8;
                        let g = (rgb[1] * 255.0) as u8;
                        let b = (rgb[2] * 255.0) as u8;
                        return [r, g, b, 255];
                    }
                }
                PdfColorSpace::DeviceCmyk => {
                    if let Some(cmyk) = text_obj.fill_color() {
                        let c = cmyk[0];
                        let m = cmyk[1];
                        let y = cmyk[2];
                        let k = cmyk[3];
                        // CMYK to RGB conversion
                        let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                        let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                        let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                        return [r, g, b, 255];
                    }
                }
                _ => {}
            }
        }
        // Default to black
        [0, 0, 0, 255]
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

        // Extract path commands from PDF path object
        let mut path_commands = Vec::new();
        for segment in path_obj.path_segments() {
            match segment.segment_type() {
                PdfPathSegmentType::MoveTo => {
                    path_commands.push(idm::PathCommand::MoveTo(geometry::Point {
                        x: segment.end_point().x.value,
                        y: page_height - segment.end_point().y.value,
                    }));
                }
                PdfPathSegmentType::LineTo => {
                    path_commands.push(idm::PathCommand::LineTo(geometry::Point {
                        x: segment.end_point().x.value,
                        y: page_height - segment.end_point().y.value,
                    }));
                }
                PdfPathSegmentType::BezierTo => {
                    if let (Some(cp1), Some(cp2)) = (segment.control_point_1(), segment.control_point_2()) {
                        path_commands.push(idm::PathCommand::CurveTo {
                            control1: geometry::Point {
                                x: cp1.x.value,
                                y: page_height - cp1.y.value,
                            },
                            control2: geometry::Point {
                                x: cp2.x.value,
                                y: page_height - cp2.y.value,
                            },
                            end: geometry::Point {
                                x: segment.end_point().x.value,
                                y: page_height - segment.end_point().y.value,
                            },
                        });
                    }
                }
                PdfPathSegmentType::Close => {
                    path_commands.push(idm::PathCommand::Close);
                }
                _ => {}
            }
        }

        // Extract stroke and fill colors
        let stroke_color = self.extract_path_stroke_color(path_obj);
        let fill_color = self.extract_path_fill_color(path_obj);
        let stroke_width = path_obj.stroke_width().map(|w| w.value).unwrap_or(1.0);

        Some(VectorShapeNode {
            bounds: BoundingBox {
                x: bounds.left.value,
                y: top_y,
                width: bounds.right.value - bounds.left.value,
                height: bounds.top.value - bounds.bottom.value,
            },
            path_commands,
            stroke_color,
            fill_color,
            stroke_width,
        })
    }

    fn extract_path_stroke_color(&self, path_obj: &PdfPagePathObject) -> Option<[u8; 4]> {
        if let Some(colorspace) = path_obj.stroke_colorspace() {
            match colorspace {
                PdfColorSpace::DeviceGray => {
                    if let Some(gray) = path_obj.stroke_color() {
                        let g = (gray[0] * 255.0) as u8;
                        return Some([g, g, g, 255]);
                    }
                }
                PdfColorSpace::DeviceRgb => {
                    if let Some(rgb) = path_obj.stroke_color() {
                        let r = (rgb[0] * 255.0) as u8;
                        let g = (rgb[1] * 255.0) as u8;
                        let b = (rgb[2] * 255.0) as u8;
                        return Some([r, g, b, 255]);
                    }
                }
                PdfColorSpace::DeviceCmyk => {
                    if let Some(cmyk) = path_obj.stroke_color() {
                        let c = cmyk[0];
                        let m = cmyk[1];
                        let y = cmyk[2];
                        let k = cmyk[3];
                        let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                        let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                        let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                        return Some([r, g, b, 255]);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn extract_path_fill_color(&self, path_obj: &PdfPagePathObject) -> Option<[u8; 4]> {
        if let Some(colorspace) = path_obj.fill_colorspace() {
            match colorspace {
                PdfColorSpace::DeviceGray => {
                    if let Some(gray) = path_obj.fill_color() {
                        let g = (gray[0] * 255.0) as u8;
                        return Some([g, g, g, 255]);
                    }
                }
                PdfColorSpace::DeviceRgb => {
                    if let Some(rgb) = path_obj.fill_color() {
                        let r = (rgb[0] * 255.0) as u8;
                        let g = (rgb[1] * 255.0) as u8;
                        let b = (rgb[2] * 255.0) as u8;
                        return Some([r, g, b, 255]);
                    }
                }
                PdfColorSpace::DeviceCmyk => {
                    if let Some(cmyk) = path_obj.fill_color() {
                        let c = cmyk[0];
                        let m = cmyk[1];
                        let y = cmyk[2];
                        let k = cmyk[3];
                        let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                        let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                        let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                        return Some([r, g, b, 255]);
                    }
                }
                _ => {}
            }
        }
        None
    }
}
