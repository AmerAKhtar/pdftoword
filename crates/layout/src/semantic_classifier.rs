use idm::{ElementNode, PageNode, SemanticType};

pub struct SemanticClassifier;

impl SemanticClassifier {
    /// Classifies all aggregated text groups on a page into semantic structural nodes
    pub fn classify_page_semantics(page: &mut PageNode, body_font_size_mode: f32) {
        let page_height = page.bounds.height;
        let top_header_region = page_height * 0.08;   // Top 8% of page
        let bottom_footer_region = page_height * 0.92; // Bottom 8% of page

        for elem in page.elements.iter_mut() {
            if let ElementNode::TextGroup(group) = elem {
                if group.runs.is_empty() {
                    continue;
                }

                let first_run = &group.runs[0];
                let font_size = first_run.font_size;
                let text_content = group
                    .runs
                    .iter()
                    .map(|r| r.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");

                let trimmed = text_content.trim();

                // 1. Detect Headers & Footers by page position
                if group.bounds.y <= top_header_region {
                    group.semantic_type = SemanticType::Header;
                    continue;
                } else if group.bounds.y >= bottom_footer_region {
                    group.semantic_type = SemanticType::Footer;
                    continue;
                }

                // 2. Detect Lists by prefix bullet or numbering pattern
                if Self::is_list_item(trimmed) {
                    group.semantic_type = SemanticType::ListItem { indent_level: 0 };
                    continue;
                }

                // 3. Detect Headings based on font scale relative to body text mode
                if font_size >= body_font_size_mode * 1.8 {
                    group.semantic_type = SemanticType::Heading { level: 1 };
                } else if font_size >= body_font_size_mode * 1.4 {
                    group.semantic_type = SemanticType::Heading { level: 2 };
                } else if font_size >= body_font_size_mode * 1.2 {
                    group.semantic_type = SemanticType::Heading { level: 3 };
                } else {
                    group.semantic_type = SemanticType::Paragraph;
                }
            }
        }
    }

    fn is_list_item(text: &str) -> bool {
        text.starts_with('•')
            || text.starts_with('-')
            || text.starts_with('*')
            || (text.len() > 2
                && text.chars().next().unwrap().is_ascii_digit()
                && (text.chars().nth(1) == Some('.') || text.chars().nth(1) == Some(')')))
    }
}
