use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_pages_checked: usize,
    pub overall_quality_score: f32, // 0.0 to 1.0
    pub text_fidelity_score: f32,
    pub visual_fidelity_score: f32,
    pub structure_fidelity_score: f32,
    pub deviations: Vec<LayoutDeviation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDeviation {
    pub page_index: usize,
    pub deviation_type: DeviationType,
    pub severity: DeviationSeverity,
    pub description: String,
    pub delta_value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviationType {
    MissingText,
    MissingImage,
    FontSubstitution,
    BoundingBoxDrift,
    TableStructureMismatch,
    ReadingOrderShift,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ValidationConfig {
    pub max_spatial_drift_pt: f32,   // Maximum allowed position drift in points (e.g., 2.0 pt)
    pub min_acceptable_score: f32,   // Minimum threshold score (e.g., 0.95)
    pub enforce_strict_fonts: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_spatial_drift_pt: 2.0,
            min_acceptable_score: 0.95,
            enforce_strict_fonts: false,
        }
    }
}
