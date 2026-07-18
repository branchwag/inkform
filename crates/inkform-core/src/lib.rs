mod base_font;
mod domain;
mod error;
mod extraction;
mod generation;
mod preview;
mod reference_bank;
mod ttf;
mod validation;

pub use crate::domain::{
    FontArtifact, GenerationReport, GlyphCandidate, PreviewRequest, PreviewResponse,
    ProcessingStage, SampleImage, SampleQuality, ScriptPack, ValidationReport,
};
pub use crate::error::{InkformError, InkformErrorKind};
pub use crate::generation::generate_font;
pub use crate::preview::{PREVIEW_VERSION, preview_text};
pub use crate::ttf::build_preview_svg;
pub use crate::validation::validate_sample;

#[cfg(test)]
mod tests {
    use crate::{
        PreviewRequest, SampleImage, SampleQuality, ScriptPack, generate_font, preview_text,
        validate_sample,
    };
    use ttf_parser::{Face, OutlineBuilder};

    #[derive(Default)]
    struct CountingOutlineBuilder {
        move_events: usize,
    }

    impl OutlineBuilder for CountingOutlineBuilder {
        fn move_to(&mut self, _x: f32, _y: f32) {
            self.move_events += 1;
        }

        fn line_to(&mut self, _x: f32, _y: f32) {}

        fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {}

        fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {}

        fn close(&mut self) {}
    }

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
        assert_eq!(&artifact.binary[0..4], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn generates_a_ttf_parser_readable_font() {
        let artifact_result = generate_font(&sample_image(), &ScriptPack::latin_extended());
        assert!(
            artifact_result.is_ok(),
            "font generation should succeed: {artifact_result:?}"
        );
        let artifact = match artifact_result {
            Ok(artifact) => artifact,
            Err(error) => panic!("unexpected generation error: {error}"),
        };

        let face_result = Face::parse(&artifact.binary, 0);
        assert!(
            face_result.is_ok(),
            "generated font should parse with ttf-parser: {face_result:?}"
        );
        let face = match face_result {
            Ok(face) => face,
            Err(error) => panic!("unexpected parser error: {error:?}"),
        };

        let expected_glyph_count = u16::try_from(artifact.glyphs.len())
            .unwrap_or(u16::MAX)
            .saturating_add(1);
        assert_eq!(face.number_of_glyphs(), expected_glyph_count);
        let glyph_a = face.glyph_index('A');
        assert!(glyph_a.is_some());
        assert!(face.glyph_index('ß').is_some());

        let Some(glyph_a) = glyph_a else {
            panic!("expected glyph id for 'A'");
        };
        let bbox = face.glyph_bounding_box(glyph_a);
        assert!(bbox.is_some(), "expected bounding box for 'A'");
        let mut builder = CountingOutlineBuilder::default();
        let outline = face.outline_glyph(glyph_a, &mut builder);
        assert!(outline.is_some(), "expected outline for 'A'");
        assert!(
            builder.move_events > 0,
            "expected drawable contours for 'A'"
        );
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
