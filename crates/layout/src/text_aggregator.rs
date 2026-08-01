use geometry::BoundingBox;
use idm::{ElementNode, PageNode, SemanticType, TextGroup, TextRun};

pub struct TextAggregator {
    line_height_tolerance: f32,
    word_gap_threshold: f32,
}

impl Default for TextAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl TextAggregator {
    pub fn new() -> Self {
        Self {
            line_height_tolerance: 2.5, // Vertical distance threshold for same line
            word_gap_threshold: 4.0,    // Horizontal gap threshold for word separation
        }
    }

    /// Aggregates scattered text runs on a page into structured paragraphs and lines
    pub fn aggregate_page_text(&self, page: &mut PageNode) {
        let mut raw_runs = Vec::new();
        let mut non_text_elements = Vec::new();

        // Separate raw text groups from other elements (images, shapes, tables)
        for elem in page.elements.drain(..) {
            match elem {
                ElementNode::TextGroup(group) => {
                    for run in group.runs {
                        raw_runs.push(run);
                    }
                }
                other => non_text_elements.push(other),
            }
        }

        if raw_runs.is_empty() {
            page.elements = non_text_elements;
            return;
        }

        // 1. Sort runs vertically (Top-Y), then horizontally (Left-X)
        raw_runs.sort_by(|a, b| {
            if (a.bounds.y - b.bounds.y).abs() < self.line_height_tolerance {
                a.bounds.x.partial_cmp(&b.bounds.x).unwrap()
            } else {
                a.bounds.y.partial_cmp(&b.bounds.y).unwrap()
            }
        });

        // 2. Cluster runs into lines
        let lines = self.cluster_into_lines(raw_runs);

        // 3. Cluster lines into paragraph blocks
        let paragraphs = self.cluster_lines_into_paragraphs(lines);

        // Reassemble into page elements
        let mut reassembled_elements = non_text_elements;
        for (idx, para) in paragraphs.into_iter().enumerate() {
            reassembled_elements.push(ElementNode::TextGroup(TextGroup {
                bounds: para.bounds,
                reading_order: idx,
                runs: para.runs,
                semantic_type: para.semantic_type,
            }));
        }

        page.elements = reassembled_elements;
    }

    fn cluster_into_lines(&self, runs: Vec<TextRun>) -> Vec<LineCluster> {
        let mut lines: Vec<LineCluster> = Vec::new();

        for run in runs {
            if let Some(current_line) = lines.last_mut() {
                // Check if run belongs to the current line (similar Y position)
                if (current_line.y_center - (run.bounds.y + run.bounds.height / 2.0)).abs()
                    < self.line_height_tolerance
                {
                    current_line.add_run(run, self.word_gap_threshold);
                    continue;
                }
            }

            // Start a new line cluster
            lines.push(LineCluster::new(run));
        }

        lines
    }

    fn cluster_lines_into_paragraphs(&self, lines: Vec<LineCluster>) -> Vec<ParagraphCluster> {
        let mut paragraphs: Vec<ParagraphCluster> = Vec::new();

        for line in lines {
            if let Some(current_para) = paragraphs.last_mut() {
                let vertical_gap = line.bounds.y - (current_para.bounds.y + current_para.bounds.height);
                let expected_line_spacing = line.primary_font_size * 1.5;

                // Merge into paragraph if vertical gap matches normal paragraph line spacing
                if vertical_gap >= 0.0 && vertical_gap <= expected_line_spacing {
                    current_para.add_line(line);
                    continue;
                }
            }

            paragraphs.push(ParagraphCluster::new(line));
        }

        paragraphs
    }
}

pub struct LineCluster {
    pub bounds: BoundingBox,
    pub y_center: f32,
    pub primary_font_size: f32,
    pub runs: Vec<TextRun>,
}

impl LineCluster {
    pub fn new(run: TextRun) -> Self {
        let bounds = run.bounds;
        let y_center = bounds.y + bounds.height / 2.0;
        let font_size = run.font_size;

        Self {
            bounds,
            y_center,
            primary_font_size: font_size,
            runs: vec![run],
        }
    }

    pub fn add_run(&mut self, run: TextRun, word_gap_threshold: f32) {
        let last_x_end = self.bounds.x + self.bounds.width;
        let gap = run.bounds.x - last_x_end;

        // Insert space character if gap between text runs indicates a word boundary
        if gap > word_gap_threshold {
            let mut spaced_run = run.clone();
            spaced_run.text = format!(" {}", run.text);
            self.bounds.extend(&spaced_run.bounds);
            self.runs.push(spaced_run);
        } else {
            self.bounds.extend(&run.bounds);
            self.runs.push(run);
        }
    }
}

pub struct ParagraphCluster {
    pub bounds: BoundingBox,
    pub runs: Vec<TextRun>,
    pub semantic_type: SemanticType,
}

impl ParagraphCluster {
    pub fn new(line: LineCluster) -> Self {
        Self {
            bounds: line.bounds,
            runs: line.runs,
            semantic_type: SemanticType::Unstructured,
        }
    }

    pub fn add_line(&mut self, line: LineCluster) {
        self.bounds.extend(&line.bounds);
        self.runs.extend(line.runs);
    }
}
