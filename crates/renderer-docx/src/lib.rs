pub mod xml_builder;

use idm::{FontResource, ImageResource, IntermediateDocument};
use std::io::{Seek, Write};
use std::path::Path;
use thiserror::Error;
use xml_builder::OoxmlDocumentBuilder;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Error, Debug)]
pub enum DocxRenderError {
    #[error("ZIP Archive Generation Error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct DocxRenderer;

impl DocxRenderer {
    pub fn render_to_file(idm: &IntermediateDocument, output_path: &Path) -> Result<(), DocxRenderError> {
        let file = std::fs::File::create(output_path)?;
        Self::render_to_stream(idm, file)
    }

    pub fn render_to_stream<W: Write + Seek>(
        idm: &IntermediateDocument,
        writer: W,
    ) -> Result<(), DocxRenderError> {
        let mut zip = ZipWriter::new(writer);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // 1. Write [Content_Types].xml with dynamic content types for fonts and images
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(Self::content_types_xml(idm).as_bytes())?;

        // 2. Write _rels/.rels
        zip.add_directory("_rels", options)?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(Self::root_rels_xml().as_bytes())?;

        // 3. Write word/_rels/document.xml.rels with image relationships
        zip.add_directory("word/_rels", options)?;
        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(Self::document_rels_xml(idm).as_bytes())?;

        // 4. Write word/styles.xml with font definitions
        zip.start_file("word/styles.xml", options)?;
        zip.write_all(Self::styles_xml(idm).as_bytes())?;

        // 5. Write word/fontTable.xml for embedded fonts
        if !idm.resources.fonts.is_empty() {
            zip.start_file("word/fontTable.xml", options)?;
            zip.write_all(Self::font_table_xml(idm).as_bytes())?;
        }

        // 6. Write embedded font files
        for (font_id, font_resource) in &idm.resources.fonts {
            if font_resource.is_embedded && !font_resource.data.is_empty() {
                let font_filename = format!("word/fonts/{}.ttf", font_id);
                zip.start_file(&font_filename, options)?;
                zip.write_all(&font_resource.data)?;
            }
        }

        // 7. Write embedded images
        let mut image_idx = 1;
        for (resource_id, image_resource) in &idm.resources.images {
            let extension = match image_resource.mime_type.as_str() {
                "image/png" => "png",
                "image/jpeg" | "image/jpg" => "jpg",
                "image/gif" => "gif",
                "image/bmp" => "bmp",
                _ => "png",
            };
            let image_filename = format!("word/media/image{}.{}", image_idx, extension);
            zip.start_file(&image_filename, options)?;
            zip.write_all(&image_resource.raw_data)?;
            image_idx += 1;
        }

        // 8. Write main word/document.xml from IDM
        let document_xml = OoxmlDocumentBuilder::build_document_xml(idm);
        zip.start_file("word/document.xml", options)?;
        zip.write_all(document_xml.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    fn content_types_xml(idm: &IntermediateDocument) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#);
        xml.push_str(r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#);
        xml.push_str(r#"<Default Extension="xml" ContentType="application/xml"/>"#);
        
        // Add font content type
        xml.push_str(r#"<Default Extension="ttf" ContentType="application/vnd.ms-package.embeddedopentype"/>"#);
        
        // Add image content types dynamically based on what's in the document
        let mut has_png = false;
        let mut has_jpg = false;
        let mut has_gif = false;
        let mut has_bmp = false;
        
        for img_res in idm.resources.images.values() {
            match img_res.mime_type.as_str() {
                "image/png" => has_png = true,
                "image/jpeg" | "image/jpg" => has_jpg = true,
                "image/gif" => has_gif = true,
                "image/bmp" => has_bmp = true,
                _ => {}
            }
        }
        
        if has_png {
            xml.push_str(r#"<Override PartName="/word/media/image1.png" ContentType="image/png"/>"#);
        }
        if has_jpg {
            xml.push_str(r#"<Override PartName="/word/media/image1.jpg" ContentType="image/jpeg"/>"#);
        }
        if has_gif {
            xml.push_str(r#"<Override PartName="/word/media/image1.gif" ContentType="image/gif"/>"#);
        }
        if has_bmp {
            xml.push_str(r#"<Override PartName="/word/media/image1.bmp" ContentType="image/bmp"/>"#);
        }
        
        xml.push_str(r#"<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#);
        xml.push_str(r#"<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#);
        
        if !idm.resources.fonts.is_empty() {
            xml.push_str(r#"<Override PartName="/word/fontTable.xml" ContentType="application/vnd.ms-wordml.fontTable+xml"/>"#);
        }
        
        xml.push_str(r#"</Types>"#);
        xml
    }

    fn root_rels_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#
    }

    fn document_rels_xml(idm: &IntermediateDocument) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#);
        xml.push_str(r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#);
        
        if !idm.resources.fonts.is_empty() {
            xml.push_str(r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/>"#);
        }
        
        // Add image relationships
        let mut img_idx = 1;
        for (resource_id, image_resource) in &idm.resources.images {
            let extension = match image_resource.mime_type.as_str() {
                "image/png" => "png",
                "image/jpeg" | "image/jpg" => "jpg",
                "image/gif" => "gif",
                "image/bmp" => "bmp",
                _ => "png",
            };
            let _ = write!(&mut xml, r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image{}.{}"/>"#, 
                          img_idx + 2, img_idx, extension);
            img_idx += 1;
        }
        
        xml.push_str(r#"</Relationships>"#);
        xml
    }

    fn styles_xml(idm: &IntermediateDocument) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#);
        
        // Collect unique font families
        let mut font_families: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for font_res in idm.resources.fonts.values() {
            *font_families.entry(font_res.family_name.clone()).or_insert(0) += 1;
        }
        
        // Default Normal style
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Normal" w:default="1">"#);
        xml.push_str(r#"<w:name w:val="Normal"/>"#);
        if let Some(first_font) = font_families.keys().next() {
            let _ = write!(&mut xml, r#"<w:rPr><w:rFonts w:ascii="{}" w:hAnsi="{}" w:eastAsia="{}" w:cs="{}"/><w:sz w:val="22"/></w:rPr>"#, 
                          first_font, first_font, first_font, first_font);
        } else {
            xml.push_str(r#"<w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/><w:sz w:val="22"/></w:rPr>"#);
        }
        xml.push_str(r#"</w:style>"#);
        
        // Heading styles
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Heading1">"#);
        xml.push_str(r#"<w:name w:val="heading 1"/>"#);
        xml.push_str(r#"<w:rPr><w:b/><w:sz w:val="32"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Heading2">"#);
        xml.push_str(r#"<w:name w:val="heading 2"/>"#);
        xml.push_str(r#"<w:rPr><w:b/><w:sz w:val="28"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Heading3">"#);
        xml.push_str(r#"<w:name w:val="heading 3"/>"#);
        xml.push_str(r#"<w:rPr><w:b/><w:sz w:val="24"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        // List paragraph style
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="ListParagraph">"#);
        xml.push_str(r#"<w:name w:val="List Paragraph"/>"#);
        xml.push_str(r#"<w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        // Header and Footer styles
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Header">"#);
        xml.push_str(r#"<w:name w:val="Header"/>"#);
        xml.push_str(r#"<w:rPr><w:sz w:val="20"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        xml.push_str(r#"<w:style w:type="paragraph" w:styleId="Footer">"#);
        xml.push_str(r#"<w:name w:val="Footer"/>"#);
        xml.push_str(r#"<w:rPr><w:sz w:val="20"/></w:rPr>"#);
        xml.push_str(r#"</w:style>"#);
        
        xml.push_str(r#"</w:styles>"#);
        xml
    }

    fn font_table_xml(idm: &IntermediateDocument) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#);
        
        for font_res in idm.resources.fonts.values() {
            xml.push_str(r#"<w:font>"#);
            let _ = write!(&mut xml, r#"<w:name w:val="{}"/>"#, font_res.family_name);
            xml.push_str(r#"<w:panose1 w:val="000000000000000000000000"/>"#);
            xml.push_str(r#"<w:charset w:val="00"/>"#);
            xml.push_str(r#"<w:family w:val="auto"/>"#);
            
            if font_res.is_embedded && !font_res.data.is_empty() {
                let font_id = format!("font_{}", font_res.family_name.replace(" ", "_").replace("+", ""));
                let _ = write!(&mut xml, r#"<w:embedRegular w:val="{}"/>"#, font_id);
            }
            
            xml.push_str(r#"</w:font>"#);
        }
        
        xml.push_str(r#"</w:fonts>"#);
        xml
    }
}
