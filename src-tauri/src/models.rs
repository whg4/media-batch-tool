use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Video,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub kind: MediaKind,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
    pub thumb: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    pub text: Option<String>,
    /// absolute path to an image overlay (png with alpha recommended)
    pub image: Option<String>,
    /// top-left | top-right | bottom-left | bottom-right | center
    pub position: String,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateKind {
    Slim,
    Convert,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub kind: TemplateKind,
    /// target format extension for conversion: jpg/png/webp/avif/mp4/...
    pub target_format: Option<String>,
    /// image quality 1-100 (jpeg/webp/avif)
    pub quality: Option<u8>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    /// video codec: h264 | hevc | av1
    pub video_codec: Option<String>,
    /// video quality / crf 0-51 (lower = better)
    pub video_crf: Option<u8>,
    /// max dimension (longest side) for video
    pub video_max_dim: Option<u32>,
    /// strip audio: true/false
    pub strip_audio: Option<bool>,
    pub watermark: Option<WatermarkConfig>,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessedItem {
    pub id: String,
    pub name: String,
    pub output_path: Option<String>,
    pub output_size: Option<u64>,
    pub saved: Option<u64>,
    pub skipped: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub saved_bytes: u64,
    pub items: Vec<ProcessedItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub exported: usize,
    pub errors: Vec<(String, String)>, // (file name, error)
}
