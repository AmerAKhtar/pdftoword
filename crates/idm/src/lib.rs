use geometry::{BoundingBox, Point, Transform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root Intermediate Document Model (IDM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateDocument {
    pub metadata: DocumentMetadata,
    pub pages: Vec<PageNode>,
    pub resources: ResourceManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub page_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNode {
    pub page_index: usize,
    pub bounds: BoundingBox,
    pub rotation: u16, // 0, 90, 180, 270
    pub layers: Vec<LayerNode>,
    pub elements: Vec<ElementNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerNode {
    pub id: String,
    pub name: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementNode {
    TextGroup(TextGroup),
    Image(ImageNode),
    VectorShape(VectorShapeNode),
    Table(TableNode),
    Annotation(AnnotationNode),
}

// ==========================================
// Text Primitives
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGroup {
    pub bounds: BoundingBox,
    pub reading_order: usize,
    pub runs: Vec<TextRun>,
    pub semantic_type: SemanticType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SemanticType {
    Paragraph,
    Heading { level: u8 },
    Header,
    Footer,
    ListItem { indent_level: u8 },
    Caption,
    Unstructured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub bounds: BoundingBox,
    pub font_id: String,
    pub font_size: f32,
    pub color_rgba: [u8; 4],
    pub is_bold: bool,
    pub is_italic: bool,
    pub character_spacing: f32,
    pub word_spacing: f32,
    pub transform: Transform,
}

// ==========================================
// Image Primitives
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageNode {
    pub resource_id: String,
    pub bounds: BoundingBox,
    pub transform: Transform,
    pub soft_mask_id: Option<String>,
    pub is_inline: bool,
}

// ==========================================
// Vector Primitives
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorShapeNode {
    pub bounds: BoundingBox,
    pub path_commands: Vec<PathCommand>,
    pub stroke_color: Option<[u8; 4]>,
    pub fill_color: Option<[u8; 4]>,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CurveTo { control1: Point, control2: Point, end: Point },
    Close,
}

// ==========================================
// Table Primitives
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNode {
    pub bounds: BoundingBox,
    pub rows: Vec<TableRow>,
    pub col_widths: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub bounds: BoundingBox,
    pub col_span: usize,
    pub row_span: usize,
    pub content: Vec<ElementNode>,
    pub border_styles: CellBorders,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellBorders {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationNode {
    pub bounds: BoundingBox,
    pub annotation_type: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceManifest {
    pub fonts: HashMap<String, FontResource>,
    pub images: HashMap<String, ImageResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResource {
    pub family_name: String,
    pub is_embedded: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResource {
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub raw_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idm_serialization() {
        let doc = IntermediateDocument {
            metadata: DocumentMetadata {
                title: Some("Test Doc".into()),
                author: Some("Author".into()),
                producer: None,
                creation_date: None,
                page_count: 1,
            },
            pages: vec![],
            resources: ResourceManifest::default(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: IntermediateDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.metadata.title, Some("Test Doc".into()));
    }
}
