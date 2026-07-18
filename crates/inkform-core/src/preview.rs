use crate::domain::{FontArtifact, PreviewRequest, PreviewResponse};
use crate::error::{InkformError, InkformErrorKind};

pub const PREVIEW_VERSION: &str = "svg-v3";

/// Build a preview plan for text rendered with a generated font artifact.
///
/// # Errors
///
/// Returns an error when the preview text is empty or whitespace-only.
pub fn preview_text(
    font_artifact: &FontArtifact,
    preview_request: &PreviewRequest,
) -> Result<PreviewResponse, InkformError> {
    if preview_request.text.trim().is_empty() {
        return Err(InkformError::new(
            InkformErrorKind::InvalidInput,
            "preview text cannot be empty",
        ));
    }

    let unsupported_characters = preview_request
        .text
        .chars()
        .filter(|character| {
            !font_artifact
                .glyphs
                .iter()
                .any(|glyph| glyph.character == *character)
        })
        .collect::<Vec<_>>();

    Ok(PreviewResponse {
        render_plan: format!(
            "Preview '{}' with {} glyphs from {}.",
            preview_request.text,
            font_artifact.glyphs.len(),
            font_artifact.family_name
        ),
        unsupported_characters,
        preview_version: String::from(PREVIEW_VERSION),
        svg_markup: String::new(),
    })
}
