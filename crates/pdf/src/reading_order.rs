use idm::{ElementNode, PageNode};

pub struct ReadingOrderAnalyzer;

impl ReadingOrderAnalyzer {
    pub fn sort_page_elements(page: &mut PageNode) {
        page.elements.sort_by(|a, b| {
            let bounds_a = Self::element_bounds(a);
            let bounds_b = Self::element_bounds(b);

            // Compare Y coordinates first with a tolerance threshold (for horizontal alignment)
            let y_diff = (bounds_a.y - bounds_b.y).abs();
            if y_diff < 3.0 {
                bounds_a.x.partial_cmp(&bounds_b.x).unwrap()
            } else {
                bounds_a.y.partial_cmp(&bounds_b.y).unwrap()
            }
        });

        // Re-index reading order
        for (idx, elem) in page.elements.iter_mut().enumerate() {
            if let ElementNode::TextGroup(ref mut group) = elem {
                group.reading_order = idx;
            }
        }
    }

    fn element_bounds(elem: &ElementNode) -> &geometry::BoundingBox {
        match elem {
            ElementNode::TextGroup(g) => &g.bounds,
            ElementNode::Image(i) => &i.bounds,
            ElementNode::VectorShape(v) => &v.bounds,
            ElementNode::Table(t) => &t.bounds,
            ElementNode::Annotation(a) => &a.bounds,
        }
    }
}
