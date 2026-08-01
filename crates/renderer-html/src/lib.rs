use idm::IntermediateDocument;

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn render_to_string(_idm: &IntermediateDocument) -> String {
        "<html><body></body></html>".to_string()
    }
}
