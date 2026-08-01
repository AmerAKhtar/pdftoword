use idm::IntermediateDocument;
use std::path::Path;

pub struct DocxRenderer;

impl DocxRenderer {
    pub fn render_to_file(_idm: &IntermediateDocument, _output_path: &Path) -> Result<(), String> {
        Ok(())
    }
}
