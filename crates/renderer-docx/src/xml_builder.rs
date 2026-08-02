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

        // Font Family - use actual font family name from resource if available
        let font_family = run.font_id.strip_prefix("font_").unwrap_or(&run.font_id);
        let _ = write!(xml, r#"<w:rFonts w:ascii="{}" w:hAnsi="{}" w:eastAsia="{}" w:cs="{}"/>"#, 
                      font_family, font_family, font_family, font_family);

        // Font Size (in half-points)
        let half_points = (run.font_size * 2.0) as u32;
        let _ = write!(xml, r#"<w:sz w:val="{}"/><w:szCs w:val="{}"/>"#, half_points, half_points);

        // Bold
        if run.is_bold {
            xml.push_str(r#"<w:b/><w:bCs/>"#);
        }

        // Italic
        if run.is_italic {
            xml.push_str(r#"<w:i/><w:iCs/>"#);
        }

        // Color - convert RGBA to RGB hex
        if run.color_rgba != [0, 0, 0, 255] {
            let _ = write!(xml, r#"<w:color w:val="{:02X}{:02X}{:02X}"/>"#, 
                          run.color_rgba[0], run.color_rgba[1], run.color_rgba[2]);
        }

        // Character spacing
        if run.character_spacing != 0.0 {
            let spacing_twips = (run.character_spacing * 20.0) as i32;
            let _ = write!(xml, r#"<w:spacing w:val="{}"/>"#, spacing_twips);
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

    fn render_image(xml: &mut String, img: &idm::ImageNode) {
        // Calculate image dimensions in EMUs (English Metric Units)
        // 1 inch = 914400 EMUs, 1 pixel ≈ 9525 EMUs (at 96 DPI)
        let width_emu = (img.bounds.width * 9525.0) as u64;
        let height_emu = (img.bounds.height * 9525.0) as u64;
        
        // Extract image index from resource_id (format: "img_0x...")
        // We'll use a simple counter approach - the renderer tracks this
        let img_idx = img.resource_id.trim_start_matches("img_").hash(std::hash::BuildHasher::hasher(&std::collections::hash_map::RandomState::new())) as u32 % 1000 + 1;
        
        xml.push_str(r#"<w:p>"#);
        xml.push_str(r#"<w:r>"#);
        xml.push_str(r#"<wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" distT="0" distB="0" distL="0" distR="0">"#);
        
        // Extent (size)
        let _ = write!(xml, r#"<wp:extent cx="{}" cy="{}"/>"#, width_emu, height_emu);
        
        // Effect extent
        let _ = write!(xml, r#"<wp:effectExtent l="0" t="0" r="0" b="0"/>"#);
        
        // Doc properties
        let _ = write!(xml, r#"<wp:docPr id="{}" name="Picture {}"/>"#, img_idx, img_idx);
        
        // Graphic frame
        xml.push_str(r#"<wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/></wp:cNvGraphicFramePr>"#);
        
        // Graphic with blip fill
        xml.push_str(r#"<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image">"#);
        xml.push_str(r#"<pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">"#);
        
        // Non-visual picture properties
        let _ = write!(xml, r#"<pic:nvPicPr><cNvPr id="{}" name="Picture {}"/><cNvPicPr><a:picLocks noChangeAspect="1" noChangeArrowheads="1"/></cNvPicPr></pic:nvPicPr>"#, img_idx, img_idx);
        
        // Blip fill with embed reference
        let _ = write!(xml, r#"<pic:blipFill><a:blip r:embed="rId{}"><a:stretch><a:fillRect/></a:stretch></a:blip><a:srcRect/><a:tile/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>"#, img_idx + 2);
        
        // Shape properties
        xml.push_str(r#"<pic:spPr bwMode="auto"><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/><a:ln><a:noFill/></a:ln></pic:spPr>"#);
        
        xml.push_str(r#"</pic:pic></a:graphicData></a:graphic>"#);
        xml.push_str(r#"</wp:inline>"#);
        xml.push_str(r#"</w:r>"#);
        xml.push_str(r#"</w:p>"#);
    }
}
