use idm::{
    ElementNode, IntermediateDocument, PageNode, SemanticType, TextGroup, TextRun,
};
use std::fmt::Write;

pub struct OoxmlDocumentBuilder;

impl OoxmlDocumentBuilder {
    /// Generates the main word/document.xml content string from the IDM
    pub fn build_document_xml(idm: &IntermediateDocument) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#);
        xml.push_str(r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#);
        xml.push_str(r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">"#);
        xml.push_str(r#"<w:body>"#);

        for page in &idm.pages {
            Self::render_page_elements(&mut xml, page);
        }

        // Section properties (margins: 1 inch = 1440 twips)
        xml.push_str(r#"<w:sectPr>"#);
        xml.push_str(r#"<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>"#);
        xml.push_str(r#"</w:sectPr>"#);

        xml.push_str(r#"</w:body></w:document>"#);
        xml
    }

    fn render_page_elements(xml: &mut String, page: &PageNode) {
        for elem in &page.elements {
            match elem {
                ElementNode::TextGroup(group) => {
                    Self::render_paragraph(xml, group);
                }
                ElementNode::Table(table) => {
                    Self::render_table(xml, table);
                }
                ElementNode::Image(img) => {
                    Self::render_image(xml, img);
                }
                _ => {}
            }
        }
    }

    fn render_paragraph(xml: &mut String, group: &TextGroup) {
        xml.push_str(r#"<w:p>"#);

        // Paragraph Properties & Styles
        xml.push_str(r#"<w:pPr>"#);
        match group.semantic_type {
            SemanticType::Heading { level } => {
                let _ = write!(xml, r#"<w:pStyle w:val="Heading{}"/>"#, level);
            }
            SemanticType::ListItem { .. } => {
                xml.push_str(r#"<w:pStyle w:val="ListParagraph"/>"#);
            }
            SemanticType::Header => {
                xml.push_str(r#"<w:pStyle w:val="Header"/>"#);
            }
            SemanticType::Footer => {
                xml.push_str(r#"<w:pStyle w:val="Footer"/>"#);
            }
            _ => {
                xml.push_str(r#"<w:pStyle w:val="Normal"/>"#);
            }
        }
        xml.push_str(r#"</w:pPr>"#);

        // Render Runs
        for run in &group.runs {
            Self::render_run(xml, run);
        }

        xml.push_str(r#"</w:p>"#);
    }

    fn render_run(xml: &mut String, run: &TextRun) {
        xml.push_str(r#"<w:r>"#);
        xml.push_str(r#"<w:rPr>"#);

        // Font Family
        let _ = write!(xml, r#"<w:rFonts w:ascii="{}" w:hAnsi="{}"/>"#, run.font_id, run.font_id);

        // Font Size (in half-points)
        let half_points = (run.font_size * 2.0) as u32;
        let _ = write!(xml, r#"<w:sz w:val="{}"/>"#, half_points);

        if run.is_bold {
            xml.push_str(r#"<w:b/>"#);
        }
        if run.is_italic {
            xml.push_str(r#"<w:i/>"#);
        }

        xml.push_str(r#"</w:rPr>"#);

        // Text Content
        xml.push_str(r#"<w:t xml:space="preserve">"#);
        let _ = write!(xml, "{}", quick_xml::escape::escape(&run.text));
        xml.push_str(r#"</w:t>"#);

        xml.push_str(r#"</w:r>"#);
    }

    fn render_table(xml: &mut String, table: &idm::TableNode) {
        xml.push_str(r#"<w:tbl>"#);
        xml.push_str(r#"<w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders>"#);
        xml.push_str(r#"<w:top w:val="single" w:sz="4" w:space="0" w:color="E0E0E0"/>"#);
        xml.push_str(r#"<w:bottom w:val="single" w:sz="4" w:space="0" w:color="E0E0E0"/>"#);
        xml.push_str(r#"<w:left w:val="none"/><w:right w:val="none"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="E0E0E0"/><w:insideV w:val="none"/>"#);
        xml.push_str(r#"</w:tblBorders></w:tblPr>"#);

        for row in &table.rows {
            xml.push_str(r#"<w:tr>"#);
            for cell in &row.cells {
                xml.push_str(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>"#);
                for elem in &cell.content {
                    if let ElementNode::TextGroup(group) = elem {
                        Self::render_paragraph(xml, group);
                    }
                }
                xml.push_str(r#"</w:tc>"#);
            }
            xml.push_str(r#"</w:tr>"#);
        }

        xml.push_str(r#"</w:tbl>"#);
    }

    fn render_image(xml: &mut String, _img: &idm::ImageNode) {
        // Placeholder for DrawingML image inline rendering block
        xml.push_str(r#"<w:p><w:r><w:t>[Image Object]</w:t></w:r></w:p>"#);
    }
}
