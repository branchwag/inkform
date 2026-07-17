mod domain;
mod error;
mod generation;
mod preview;
mod validation;

pub use crate::domain::{
    FontArtifact, GenerationReport, GlyphCandidate, PreviewRequest, PreviewResponse,
    ProcessingStage, SampleImage, SampleQuality, ScriptPack, ValidationReport,
};
pub use crate::error::{InkformError, InkformErrorKind};
pub use crate::generation::generate_font;
pub use crate::preview::preview_text;
pub use crate::validation::validate_sample;

#[cfg(test)]
mod tests {
    use crate::{
        PreviewRequest, SampleImage, SampleQuality, ScriptPack, generate_font, preview_text,
        validate_sample,
    };

    fn sample_image() -> SampleImage {
        SampleImage {
            width: 1600,
            height: 2200,
            bytes: vec![2; 1600],
            quality: SampleQuality::Clean,
        }
    }

    #[test]
    fn validates_a_clean_sample() {
        let report_result = validate_sample(&sample_image(), &ScriptPack::latin_extended());
        assert!(
            report_result.is_ok(),
            "clean sample should validate: {report_result:?}"
        );
        let report = match report_result {
            Ok(report) => report,
            Err(error) => panic!("unexpected validation error: {error}"),
        };

        assert!(report.accepted);
        assert_eq!(
            report.glyph_target_count,
            ScriptPack::latin_extended().glyph_count()
        );
    }

    #[test]
    fn generates_a_font_report() {
        let artifact_result = generate_font(&sample_image(), &ScriptPack::latin_extended());
        assert!(
            artifact_result.is_ok(),
            "font generation should succeed: {artifact_result:?}"
        );
        let artifact = match artifact_result {
            Ok(artifact) => artifact,
            Err(error) => panic!("unexpected generation error: {error}"),
        };

        assert_eq!(artifact.script_pack_id, "latin-extended");
        assert!(artifact.glyphs.len() >= 32);
    }

    #[test]
    fn previews_text_with_supported_characters() {
        let artifact_result = generate_font(&sample_image(), &ScriptPack::latin_extended());
        assert!(
            artifact_result.is_ok(),
            "font generation should succeed: {artifact_result:?}"
        );
        let artifact = match artifact_result {
            Ok(artifact) => artifact,
            Err(error) => panic!("unexpected generation error: {error}"),
        };
        let response_result = preview_text(
            &artifact,
            &PreviewRequest {
                text: String::from("Grüße aus Inkform"),
            },
        );
        assert!(
            response_result.is_ok(),
            "preview should succeed: {response_result:?}"
        );
        let response = match response_result {
            Ok(response) => response,
            Err(error) => panic!("unexpected preview error: {error}"),
        };

        assert_eq!(response.unsupported_characters.len(), 0);
        assert!(response.render_plan.contains("Grüße"));
    }
}
