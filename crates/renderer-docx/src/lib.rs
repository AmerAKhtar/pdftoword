pub mod xml_builder;

use idm::IntermediateDocument;
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

        // 1. Write [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(Self::content_types_xml().as_bytes())?;

        // 2. Write _rels/.rels
        zip.add_directory("_rels", options)?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(Self::root_rels_xml().as_bytes())?;

        // 3. Write word/_rels/document.xml.rels
        zip.add_directory("word/_rels", options)?;
        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(Self::document_rels_xml().as_bytes())?;

        // 4. Write word/styles.xml
        zip.start_file("word/styles.xml", options)?;
        zip.write_all(Self::styles_xml().as_bytes())?;

        // 5. Write main word/document.xml from IDM
        let document_xml = OoxmlDocumentBuilder::build_document_xml(idm);
        zip.start_file("word/document.xml", options)?;
        zip.write_all(document_xml.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    fn content_types_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
    <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#
    }

    fn root_rels_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#
    }

    fn document_rels_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#
    }

    fn styles_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:style w:type="paragraph" w:styleId="Normal" w:default="1">
        <w:name w:val="Normal"/>
        <w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading1">
        <w:name w:val="heading 1"/>
        <w:rPr><w:b/><w:sz w:val="32"/></w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading2">
        <w:name w:val="heading 2"/>
        <w:rPr><w:b/><w:sz w:val="28"/></w:rPr>
    </w:style>
</w:styles>"#
    }
}
