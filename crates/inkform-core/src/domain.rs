#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleQuality {
    Clean,
    Noisy,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleImage {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
    pub quality: SampleQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingStage {
    Ingest,
    Preprocess,
    Extract,
    Normalize,
    Assemble,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphCandidate {
    pub character: char,
    pub confidence_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub accepted: bool,
    pub stage: ProcessingStage,
    pub notes: Vec<String>,
    pub glyph_target_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationReport {
    pub stage: ProcessingStage,
    pub accepted_glyphs: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontArtifact {
    pub family_name: String,
    pub script_pack_id: String,
    pub glyphs: Vec<GlyphCandidate>,
    pub binary: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRequest {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewResponse {
    pub render_plan: String,
    pub unsupported_characters: Vec<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptPack {
    pub id: String,
    pub display_name: String,
    pub glyphs: Vec<char>,
}

impl ScriptPack {
    #[must_use]
    pub fn latin_extended() -> Self {
        const GLYPHS: &str = concat!(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "abcdefghijklmnopqrstuvwxyz",
            "0123456789",
            " .,;:!?'-_\"()[]{}@/#&%+*=<>",
            "ÄÖÜäöüßÀÁÂÃÅÆÇÈÉÊËÌÍÎÏÑÒÓÔÕØÙÚÛÝ",
            "àáâãåæçèéêëìíîïñòóôõøùúûýÿ"
        );

        Self {
            id: String::from("latin-extended"),
            display_name: String::from("Latin Extended"),
            glyphs: GLYPHS.chars().collect(),
        }
    }

    #[must_use]
    pub const fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }
}
