use crate::base_font::base_font_glyph;
use crate::domain::{GlyphCandidate, SampleImage, ScriptPack};
use crate::extraction::{ExtractionResult, extract_handwriting};
use crate::reference_bank::{ReferenceStyle, build_reference_glyph};
use image::{GrayImage, ImageReader, Luma};
use std::collections::HashMap;
use std::fmt::Write;
use std::io::Cursor;

const UNITS_PER_EM: u16 = 1000;
const ASCENDER: i16 = 820;
const DESCENDER: i16 = -220;
const LINE_GAP: i16 = 180;

#[derive(Debug, Clone)]
struct GlyphDefinition {
    advance_width: u16,
    left_side_bearing: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct GeneratedGlyph {
    character: char,
    advance_width: u16,
    left_side_bearing: i16,
    contours: Vec<Vec<(i16, i16)>>,
}

#[derive(Debug, Clone)]
struct TableRecord {
    tag: [u8; 4],
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct StyleProfile {
    slant: f32,
    width_scale: f32,
    body_height: f32,
    ascender_height: f32,
    descender_depth: f32,
    stroke_width: f32,
    waviness: f32,
    baseline_lift: f32,
}

#[derive(Debug, Clone)]
struct ExtractedShape {
    measures: ComponentMeasures,
    #[allow(dead_code)]
    outline: Vec<(i16, i16)>,
}

#[derive(Debug, Clone, Copy)]
struct StyleEmbedding {
    width_ratio: f32,
    height_ratio: f32,
    slant: f32,
    density: f32,
    baseline_offset: f32,
    ink_area: f32,
}

pub fn build_ttf(
    family_name: &str,
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
) -> Vec<u8> {
    let generated_glyphs = build_generated_glyphs(sample_image, script_pack, glyphs);
    let glyph_definitions = build_glyph_definitions_from_generated(&generated_glyphs);
    build_ttf_from_definitions(family_name, script_pack, &glyph_definitions)
}

#[must_use]
pub fn build_preview_svg(
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
    preview_text: &str,
) -> String {
    let generated_glyphs = build_generated_glyphs(sample_image, script_pack, glyphs);
    build_preview_svg_from_generated(&generated_glyphs, preview_text)
}

fn build_ttf_from_definitions(
    family_name: &str,
    script_pack: &ScriptPack,
    glyph_definitions: &[GlyphDefinition],
) -> Vec<u8> {
    let metrics = FontMetrics::from_glyphs(glyph_definitions);
    let hhea = build_hhea_table(metrics, glyph_definitions.len());
    let maxp = build_maxp_table(glyph_definitions);
    let hmtx = build_hmtx_table(glyph_definitions);
    let (glyf, loca, loca_format) = build_glyf_and_loca_tables(glyph_definitions);
    let head = build_head_table(metrics, loca_format);
    let cmap = build_cmap_table(script_pack);
    let name = build_name_table(family_name);
    let post = build_post_table();
    let os2 = build_os2_table(metrics);

    let tables = vec![
        TableRecord {
            tag: *b"OS/2",
            data: os2,
        },
        TableRecord {
            tag: *b"cmap",
            data: cmap,
        },
        TableRecord {
            tag: *b"glyf",
            data: glyf,
        },
        TableRecord {
            tag: *b"head",
            data: head,
        },
        TableRecord {
            tag: *b"hhea",
            data: hhea,
        },
        TableRecord {
            tag: *b"hmtx",
            data: hmtx,
        },
        TableRecord {
            tag: *b"loca",
            data: loca,
        },
        TableRecord {
            tag: *b"maxp",
            data: maxp,
        },
        TableRecord {
            tag: *b"name",
            data: name,
        },
        TableRecord {
            tag: *b"post",
            data: post,
        },
    ];

    build_font_file(tables)
}

fn build_generated_glyphs(
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
) -> Vec<GeneratedGlyph> {
    let base_seed = hash_bytes(&sample_image.bytes);
    let extraction_result = extract_handwriting(sample_image);
    let decoded_sheet = decode_sheet(sample_image);
    build_glyph_shapes(
        base_seed,
        script_pack,
        glyphs,
        decoded_sheet.as_ref(),
        extraction_result.as_ref(),
    )
}

#[derive(Debug, Clone, Copy)]
struct FontMetrics {
    advance_width_max: u16,
    min_left_side_bearing: i16,
    min_right_side_bearing: i16,
    x_max_extent: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    x_avg_char_width: i16,
}

impl FontMetrics {
    fn from_glyphs(glyphs: &[GlyphDefinition]) -> Self {
        let advance_width_max = glyphs
            .iter()
            .map(|glyph| glyph.advance_width)
            .max()
            .unwrap_or(UNITS_PER_EM);
        let min_left_side_bearing = glyphs
            .iter()
            .map(|glyph| glyph.left_side_bearing)
            .min()
            .unwrap_or(0);
        let min_right_side_bearing = glyphs
            .iter()
            .map(|glyph| {
                let advance_width = i32::from(glyph.advance_width);
                let right =
                    advance_width - i32::from(glyph.left_side_bearing) - i32::from(glyph.x_max);
                clamp_i32_to_i16(right)
            })
            .min()
            .unwrap_or(0);
        let x_max_extent = glyphs
            .iter()
            .map(|glyph| {
                clamp_i32_to_i16(i32::from(glyph.left_side_bearing) + i32::from(glyph.x_max))
            })
            .max()
            .unwrap_or(0);
        let x_min = glyphs.iter().map(|glyph| glyph.x_min).min().unwrap_or(0);
        let y_min = glyphs.iter().map(|glyph| glyph.y_min).min().unwrap_or(0);
        let x_max = glyphs.iter().map(|glyph| glyph.x_max).max().unwrap_or(0);
        let y_max = glyphs.iter().map(|glyph| glyph.y_max).max().unwrap_or(0);

        let total_width = glyphs.iter().fold(0_i32, |accumulator, glyph| {
            accumulator + i32::from(glyph.advance_width)
        });
        let glyph_count = i32::try_from(glyphs.len()).unwrap_or(1);
        let x_avg_char_width = clamp_i32_to_i16(total_width / glyph_count.max(1));

        Self {
            advance_width_max,
            min_left_side_bearing,
            min_right_side_bearing,
            x_max_extent,
            x_min,
            y_min,
            x_max,
            y_max,
            x_avg_char_width,
        }
    }
}

fn build_glyph_shapes(
    base_seed: u64,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
    decoded_sheet: Option<&GrayImage>,
    extraction_result: Option<&ExtractionResult>,
) -> Vec<GeneratedGlyph> {
    let extracted_shapes = extraction_result.map_or_else(
        || decoded_sheet.map(extract_shape_library).unwrap_or_default(),
        |result| {
            result
                .glyphs
                .iter()
                .map(|glyph| ExtractedShape {
                    outline: glyph.outline.clone(),
                    measures: ComponentMeasures {
                        width_ratio: glyph.width_ratio,
                        height_ratio: glyph.height_ratio,
                        slant: glyph.slant,
                        density: glyph.density,
                        baseline_offset: glyph.baseline_offset,
                        ink_area: glyph.ink_area,
                    },
                })
                .collect::<Vec<_>>()
        },
    );
    let style_profile = derive_style_profile(decoded_sheet, &extracted_shapes);
    let mut generated = Vec::with_capacity(glyphs.len());

    for (glyph_index, character) in script_pack.glyphs.iter().enumerate() {
        let candidate = glyphs
            .get(glyph_index)
            .map_or(*character, |glyph| glyph.character);
        let seed = mix_seed(base_seed, *character);
        let hinted_shape = select_shape_for_glyph(&extracted_shapes, candidate, glyph_index, seed);
        let reference_style = reference_style(style_profile, hinted_shape);
        let base_font_glyph = base_font_glyph(candidate)
            .map(|contours| deform_base_font_contours(candidate, &contours, reference_style, seed));
        let reference_glyph = build_reference_glyph(candidate, reference_style, seed);
        let advance_width = glyph_advance_width(candidate);
        let contours = match base_font_glyph {
            Some(contours) if !contours.is_empty() => contours,
            _ => match reference_glyph {
                Some(contours) if !contours.is_empty() => contours,
                _ => algorithmic_contours(candidate, advance_width, seed, style_profile),
            },
        };
        generated.push(GeneratedGlyph {
            character: candidate,
            advance_width,
            left_side_bearing: if candidate == ' ' { 0 } else { 32 },
            contours,
        });
    }

    generated
}

fn build_glyph_definitions_from_generated(
    generated_glyphs: &[GeneratedGlyph],
) -> Vec<GlyphDefinition> {
    let mut definitions = Vec::with_capacity(generated_glyphs.len() + 1);
    definitions.push(notdef_glyph());

    for glyph in generated_glyphs {
        if glyph.contours.is_empty() {
            definitions.push(GlyphDefinition {
                advance_width: glyph.advance_width,
                left_side_bearing: glyph.left_side_bearing,
                x_min: 0,
                y_min: 0,
                x_max: 0,
                y_max: 0,
                data: empty_glyph_data(),
            });
            continue;
        }

        definitions.push(build_multi_contour_glyph(
            glyph.advance_width,
            glyph.left_side_bearing,
            &glyph.contours,
        ));
    }

    definitions
}

fn notdef_glyph() -> GlyphDefinition {
    let points = [(80_i16, 0_i16), (80, 700), (520, 700), (520, 0)];
    build_simple_polygon_glyph(600, 40, &points)
}

fn derive_style_profile(
    decoded_sheet: Option<&GrayImage>,
    extracted_shapes: &[ExtractedShape],
) -> StyleProfile {
    let default = StyleProfile {
        slant: 44.0,
        width_scale: 1.0,
        body_height: 360.0,
        ascender_height: 650.0,
        descender_depth: 180.0,
        stroke_width: 54.0,
        waviness: 28.0,
        baseline_lift: 18.0,
    };

    if !extracted_shapes.is_empty() {
        return derive_style_profile_from_shapes(extracted_shapes).unwrap_or(default);
    }

    let Some(sheet) = decoded_sheet else {
        return default;
    };

    let Ok(sheet_width) = usize::try_from(sheet.width()) else {
        return default;
    };
    let Ok(sheet_height) = usize::try_from(sheet.height()) else {
        return default;
    };

    let Some(components) = extract_ink_components(sheet, sheet_width, sheet_height) else {
        return default;
    };
    if components.is_empty() {
        return default;
    }

    let mut width_ratios = Vec::new();
    let mut slants = Vec::new();
    let mut densities = Vec::new();
    let mut baseline_offsets = Vec::new();

    for component in &components {
        let Some(measures) = measure_component(component) else {
            continue;
        };
        width_ratios.push(measures.width_ratio);
        slants.push(measures.slant);
        densities.push(measures.density);
        baseline_offsets.push(measures.baseline_offset);
    }

    if width_ratios.is_empty() {
        return default;
    }

    let average_width_ratio = average(&width_ratios);
    let average_slant = average(&slants);
    let average_density = average(&densities);
    let average_baseline_offset = average(&baseline_offsets);

    StyleProfile {
        slant: clamp_f32(average_slant * 220.0, -110.0, 140.0),
        width_scale: clamp_f32(average_width_ratio.mul_add(0.75, 0.82), 0.72, 1.35),
        body_height: clamp_f32(average_density.mul_add(140.0, 300.0), 290.0, 430.0),
        ascender_height: clamp_f32(average_width_ratio.mul_add(170.0, 560.0), 520.0, 760.0),
        descender_depth: clamp_f32(average_baseline_offset.mul_add(320.0, 120.0), 90.0, 240.0),
        stroke_width: clamp_f32(average_density.mul_add(88.0, 34.0), 28.0, 110.0),
        waviness: clamp_f32((1.0 - average_density).mul_add(54.0, 14.0), 12.0, 64.0),
        baseline_lift: clamp_f32(average_baseline_offset * 64.0, -18.0, 72.0),
    }
}

fn derive_style_profile_from_shapes(extracted_shapes: &[ExtractedShape]) -> Option<StyleProfile> {
    if extracted_shapes.is_empty() {
        return None;
    }

    let mut width_ratios = Vec::with_capacity(extracted_shapes.len());
    let mut slants = Vec::with_capacity(extracted_shapes.len());
    let mut densities = Vec::with_capacity(extracted_shapes.len());
    let mut baseline_offsets = Vec::with_capacity(extracted_shapes.len());

    for shape in extracted_shapes {
        width_ratios.push(shape.measures.width_ratio);
        slants.push(shape.measures.slant);
        densities.push(shape.measures.density);
        baseline_offsets.push(shape.measures.baseline_offset);
    }

    let average_width_ratio = average(&width_ratios);
    let average_slant = average(&slants);
    let average_density = average(&densities);
    let average_baseline_offset = average(&baseline_offsets);

    Some(StyleProfile {
        slant: clamp_f32(average_slant * 220.0, -110.0, 140.0),
        width_scale: clamp_f32(average_width_ratio.mul_add(0.75, 0.82), 0.72, 1.35),
        body_height: clamp_f32(average_density.mul_add(140.0, 300.0), 290.0, 430.0),
        ascender_height: clamp_f32(average_width_ratio.mul_add(170.0, 560.0), 520.0, 760.0),
        descender_depth: clamp_f32(average_baseline_offset.mul_add(320.0, 120.0), 90.0, 240.0),
        stroke_width: clamp_f32(average_density.mul_add(88.0, 34.0), 28.0, 110.0),
        waviness: clamp_f32((1.0 - average_density).mul_add(54.0, 14.0), 12.0, 64.0),
        baseline_lift: clamp_f32(average_baseline_offset * 64.0, -18.0, 72.0),
    })
}

fn reference_style(
    style_profile: StyleProfile,
    hinted_shape: Option<&ExtractedShape>,
) -> ReferenceStyle {
    let hint_width_scale = hinted_shape.map_or(style_profile.width_scale, |shape| {
        clamp_f32(shape.measures.width_ratio.mul_add(0.42, 0.74), 0.74, 1.22)
    });
    let hint_slant = hinted_shape.map_or(style_profile.slant, |shape| {
        clamp_f32(shape.measures.slant * 260.0, -90.0, 130.0)
    });
    let hint_stroke_width = hinted_shape.map_or(style_profile.stroke_width, |shape| {
        clamp_f32(shape.measures.density.mul_add(92.0, 26.0), 24.0, 104.0)
    });

    ReferenceStyle {
        slant: (style_profile.slant + hint_slant) * 0.5,
        width_scale: (style_profile.width_scale + hint_width_scale) * 0.5,
        stroke_width: (style_profile.stroke_width + hint_stroke_width) * 0.5,
        waviness: style_profile.waviness,
        baseline_lift: style_profile.baseline_lift,
        body_height: style_profile.body_height,
        ascender_height: style_profile.ascender_height,
        descender_depth: style_profile.descender_depth,
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn deform_base_font_contours(
    character: char,
    contours: &[Vec<(i16, i16)>],
    style: ReferenceStyle,
    seed: u64,
) -> Vec<Vec<(i16, i16)>> {
    let glyph_kind = classify_glyph(character);
    let width_bias = match glyph_kind {
        GlyphKind::Uppercase => 1.08,
        GlyphKind::Lowercase => 0.98,
        GlyphKind::Descender => 1.02,
        GlyphKind::Digit => 0.94,
        GlyphKind::Punctuation => 0.72,
    };
    let vertical_bias = match glyph_kind {
        GlyphKind::Uppercase => 1.04,
        GlyphKind::Lowercase => 0.94,
        GlyphKind::Descender => 1.08,
        GlyphKind::Digit => 0.9,
        GlyphKind::Punctuation => 0.62,
    };
    let style_width = style
        .width_scale
        .mul_add(width_bias, random_unit(seed, 301) * 0.18);
    let style_slant = style.slant + random_unit(seed, 302) * 34.0;
    let bounce = style.baseline_lift + random_unit(seed, 303) * 34.0;
    let wobble = style.waviness.mul_add(0.85, 12.0);
    let body_scale =
        (style.body_height / 520.0).mul_add(vertical_bias, random_unit(seed, 304) * 0.1);
    let ascender_scale =
        (style.ascender_height / 760.0).mul_add(vertical_bias, random_unit(seed, 305) * 0.12);
    let descender_shift = style
        .descender_depth
        .mul_add(0.7, random_unit(seed, 306) * 26.0);

    contours
        .iter()
        .enumerate()
        .map(|(contour_index, contour)| {
            let deformed = contour
                .iter()
                .enumerate()
                .map(|(point_index, (x, y))| {
                    let center_x = 310.0_f32;
                    let normalized_x = f32::from(*x) - center_x;
                    let normalized_y = f32::from(*y);
                    let channel_base =
                        u32::try_from(contour_index.saturating_mul(97) + point_index).unwrap_or(0);
                    let progress_x = normalized_x / 260.0;
                    let progress_y = normalized_y / 700.0;
                    let wave_x = progress_y.sin() * wobble * 0.42;
                    let wave_y = progress_x.sin() * wobble * 0.58;
                    let jitter_x = random_unit(seed.rotate_left(11), channel_base) * wobble * 0.34;
                    let jitter_y = random_unit(seed.rotate_left(17), channel_base) * wobble * 0.4;
                    let vertical_scale = if normalized_y > style.body_height {
                        ascender_scale
                    } else {
                        body_scale
                    };
                    let descended_y = if normalized_y < 0.0 {
                        descender_shift.mul_add(-0.72, normalized_y)
                    } else {
                        normalized_y
                    };
                    let side_pull = progress_x * progress_x.abs() * 18.0;
                    let deformed_x = normalized_x.mul_add(
                        style_width,
                        style_slant.mul_add(descended_y / 620.0, center_x),
                    ) + wave_x
                        + jitter_x
                        + side_pull;
                    let deformed_y =
                        descended_y.mul_add(vertical_scale, bounce) + wave_y + jitter_y;
                    (
                        round_to_i16(clamp_f32(deformed_x, 12.0, 608.0)),
                        round_to_i16(clamp_f32(deformed_y, -240.0, 820.0)),
                    )
                })
                .collect::<Vec<_>>();
            simplify_contour(&deformed)
        })
        .filter(|contour| contour.len() >= 3)
        .collect::<Vec<_>>()
}

fn simplify_contour(points: &[(i16, i16)]) -> Vec<(i16, i16)> {
    if points.len() <= 96 {
        return dedupe_contour(points);
    }

    let deduped = dedupe_contour(points);
    if deduped.len() <= 96 {
        return deduped;
    }

    let stride = deduped.len().div_ceil(96);
    deduped.iter().step_by(stride).copied().collect::<Vec<_>>()
}

fn dedupe_contour(points: &[(i16, i16)]) -> Vec<(i16, i16)> {
    let mut deduped = Vec::with_capacity(points.len());

    for point in points {
        if deduped.last().copied() == Some(*point) {
            continue;
        }
        deduped.push(*point);
    }

    if deduped.len() >= 2 && deduped.first().copied() == deduped.last().copied() {
        let _ = deduped.pop();
    }

    deduped
}

fn extract_shape_library(sheet: &GrayImage) -> Vec<ExtractedShape> {
    let Ok(sheet_width) = usize::try_from(sheet.width()) else {
        return Vec::new();
    };
    let Ok(sheet_height) = usize::try_from(sheet.height()) else {
        return Vec::new();
    };

    let Some(components) = extract_ink_components(sheet, sheet_width, sheet_height) else {
        return Vec::new();
    };

    let mut shapes = components
        .iter()
        .filter_map(|component| {
            let measures = measure_component(component)?;
            let outline = build_shape_from_points(component)?;
            Some(ExtractedShape { measures, outline })
        })
        .collect::<Vec<_>>();

    shapes.sort_by(|left, right| {
        left.measures
            .ink_area
            .partial_cmp(&right.measures.ink_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    shapes
}

#[allow(dead_code)]
fn select_remixed_shape(
    shapes: &[ExtractedShape],
    character: char,
    glyph_index: usize,
    _glyph_count: usize,
    style_profile: StyleProfile,
    seed: u64,
) -> Option<Vec<(i16, i16)>> {
    let shape = select_shape_for_glyph(shapes, character, glyph_index, seed)?;
    remix_shape_outline(shape, character, style_profile, seed)
}

fn select_shape_for_glyph(
    shapes: &[ExtractedShape],
    character: char,
    glyph_index: usize,
    seed: u64,
) -> Option<&ExtractedShape> {
    let plausible_shapes = shapes
        .iter()
        .filter(|shape| is_plausible_shape_for_glyph(shape, character))
        .collect::<Vec<_>>();

    if plausible_shapes.is_empty() {
        return None;
    }

    let target_embedding = target_embedding(character);
    let mut ranked_shapes = plausible_shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| {
            let distance = embedding_distance(target_embedding, shape_embedding(shape));
            (index, *shape, distance)
        })
        .collect::<Vec<_>>();

    ranked_shapes.sort_by(|left, right| left.2.total_cmp(&right.2));
    let shortlist_len = ranked_shapes.len().min(4);
    let shortlist = &ranked_shapes[..shortlist_len];
    let spread_seed = usize::try_from(seed & 0xFFFF).unwrap_or(0);
    let selected_index = (glyph_index + spread_seed) % shortlist.len().max(1);
    shortlist.get(selected_index).map(|(_, shape, _)| *shape)
}

fn is_plausible_shape_for_glyph(shape: &ExtractedShape, character: char) -> bool {
    let measures = shape.measures;

    if shape.outline.len() < 12 {
        return false;
    }

    match classify_glyph(character) {
        GlyphKind::Uppercase | GlyphKind::Lowercase | GlyphKind::Descender | GlyphKind::Digit => {
            (0.22..=1.45).contains(&measures.width_ratio)
                && (0.68..=4.4).contains(&measures.height_ratio)
                && (0.08..=0.78).contains(&measures.density)
                && measures.baseline_offset <= 0.38
        }
        GlyphKind::Punctuation => {
            (0.08..=0.95).contains(&measures.width_ratio)
                && (0.18..=2.2).contains(&measures.height_ratio)
                && (0.05..=0.8).contains(&measures.density)
        }
    }
}

const fn shape_embedding(shape: &ExtractedShape) -> StyleEmbedding {
    StyleEmbedding {
        width_ratio: shape.measures.width_ratio,
        height_ratio: shape.measures.height_ratio,
        slant: shape.measures.slant,
        density: shape.measures.density,
        baseline_offset: shape.measures.baseline_offset,
        ink_area: shape.measures.ink_area,
    }
}

fn target_embedding(character: char) -> StyleEmbedding {
    match classify_glyph(character) {
        GlyphKind::Uppercase => StyleEmbedding {
            width_ratio: 0.72,
            height_ratio: 1.38,
            slant: 0.06,
            density: 0.34,
            baseline_offset: 0.08,
            ink_area: 0.038,
        },
        GlyphKind::Lowercase => StyleEmbedding {
            width_ratio: 0.86,
            height_ratio: 1.04,
            slant: 0.08,
            density: 0.3,
            baseline_offset: 0.1,
            ink_area: 0.046,
        },
        GlyphKind::Descender => StyleEmbedding {
            width_ratio: 0.82,
            height_ratio: 1.24,
            slant: 0.09,
            density: 0.31,
            baseline_offset: 0.28,
            ink_area: 0.05,
        },
        GlyphKind::Digit => StyleEmbedding {
            width_ratio: 0.78,
            height_ratio: 1.14,
            slant: 0.04,
            density: 0.36,
            baseline_offset: 0.1,
            ink_area: 0.044,
        },
        GlyphKind::Punctuation => StyleEmbedding {
            width_ratio: 0.38,
            height_ratio: 0.56,
            slant: 0.03,
            density: 0.24,
            baseline_offset: 0.06,
            ink_area: 0.014,
        },
    }
}

fn embedding_distance(left: StyleEmbedding, right: StyleEmbedding) -> f32 {
    let width_delta = (left.width_ratio - right.width_ratio).abs() * 1.7;
    let height_delta = (left.height_ratio - right.height_ratio).abs() * 1.4;
    let slant_delta = (left.slant - right.slant).abs() * 2.2;
    let density_delta = (left.density - right.density).abs() * 1.5;
    let baseline_delta = (left.baseline_offset - right.baseline_offset).abs() * 1.8;
    let ink_area_delta = (left.ink_area - right.ink_area).abs() * 2.1;

    width_delta + height_delta + slant_delta + density_delta + baseline_delta + ink_area_delta
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
#[allow(dead_code)]
fn remix_shape_outline(
    shape: &ExtractedShape,
    character: char,
    style_profile: StyleProfile,
    seed: u64,
) -> Option<Vec<(i16, i16)>> {
    let glyph_kind = classify_glyph(character);
    let target_width = f32::from(glyph_advance_width(character)) - 110.0;
    let width_scale = match glyph_kind {
        GlyphKind::Uppercase => 1.08,
        GlyphKind::Lowercase => 0.94,
        GlyphKind::Descender => 0.98,
        GlyphKind::Digit => 0.9,
        GlyphKind::Punctuation => 0.45,
    };
    let height_scale = match glyph_kind {
        GlyphKind::Uppercase => style_profile.ascender_height / 700.0,
        GlyphKind::Lowercase => style_profile.body_height / 520.0,
        GlyphKind::Descender => (style_profile.body_height + style_profile.descender_depth) / 700.0,
        GlyphKind::Digit => (style_profile.body_height + 140.0) / 700.0,
        GlyphKind::Punctuation => 0.42,
    };
    let baseline_drop = match glyph_kind {
        GlyphKind::Descender => -style_profile.descender_depth * 0.82,
        GlyphKind::Punctuation => 42.0,
        _ => style_profile.baseline_lift,
    };
    let jitter_amount = style_profile.waviness * 0.24;

    let transformed = shape
        .outline
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let normalized_x = f32::from(*x) - 310.0;
            let normalized_y = f32::from(*y);
            let jitter_channel = 40_u32 + u32::try_from(index).unwrap_or(0);
            let jitter = random_unit(seed, jitter_channel) * jitter_amount;
            let scaled_x = (normalized_x * width_scale).mul_add(
                style_profile.width_scale,
                style_profile
                    .slant
                    .mul_add(normalized_y / 700.0, target_width * 0.5),
            ) + jitter;
            let scaled_y = normalized_y.mul_add(height_scale, baseline_drop) + jitter;

            (
                round_to_i16(clamp_f32(scaled_x + 40.0, 30.0, target_width + 70.0)),
                round_to_i16(clamp_f32(scaled_y, -260.0, 780.0)),
            )
        })
        .collect::<Vec<_>>();

    if transformed.len() < 4 {
        return None;
    }

    Some(transformed)
}

#[derive(Debug, Clone, Copy)]
struct ComponentMeasures {
    width_ratio: f32,
    height_ratio: f32,
    slant: f32,
    density: f32,
    baseline_offset: f32,
    ink_area: f32,
}

#[allow(clippy::cast_precision_loss)]
fn measure_component(points: &[(usize, usize)]) -> Option<ComponentMeasures> {
    let min_x = points.iter().map(|(x, _)| *x).min()?;
    let max_x = points.iter().map(|(x, _)| *x).max()?;
    let min_y = points.iter().map(|(_, y)| *y).min()?;
    let max_y = points.iter().map(|(_, y)| *y).max()?;

    let width = (max_x - min_x + 1) as f32;
    let height = (max_y - min_y + 1) as f32;
    if width <= 1.0 || height <= 1.0 {
        return None;
    }

    let top_cutoff = min_y + ((max_y - min_y) / 3);
    let bottom_cutoff = max_y.saturating_sub((max_y - min_y) / 3);

    let top_center = centroid_x(points.iter().copied().filter(|(_, y)| *y <= top_cutoff))?;
    let bottom_center = centroid_x(points.iter().copied().filter(|(_, y)| *y >= bottom_cutoff))?;

    let ink_count = points.len() as f32;
    let box_area = width * height;
    let density = if box_area > 0.0 {
        ink_count / box_area
    } else {
        0.0
    };

    Some(ComponentMeasures {
        width_ratio: width / height,
        height_ratio: height / width.max(1.0),
        slant: (top_center - bottom_center) / width,
        density,
        baseline_offset: (max_y.saturating_sub(bottom_cutoff)) as f32 / height,
        ink_area: if box_area > 0.0 {
            ink_count / box_area
        } else {
            0.0
        },
    })
}

#[allow(clippy::cast_precision_loss)]
fn centroid_x<I>(points: I) -> Option<f32>
where
    I: Iterator<Item = (usize, usize)>,
{
    let mut total = 0.0_f32;
    let mut count = 0.0_f32;

    for (x, _) in points {
        total += x as f32;
        count += 1.0;
    }

    if count <= 0.0 {
        return None;
    }

    Some(total / count)
}

#[allow(clippy::cast_precision_loss)]
fn average(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    values.iter().copied().sum::<f32>() / (values.len() as f32)
}

const fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn build_handwritten_outline(
    character: char,
    advance_width: u16,
    seed: u64,
    style_profile: StyleProfile,
) -> Vec<(i16, i16)> {
    let glyph_kind = classify_glyph(character);
    let left_margin = 44.0_f32;
    let usable_width = (f32::from(advance_width) - 92.0) * style_profile.width_scale;
    let body_top = style_profile.body_height + random_unit(seed, 0) * 24.0;
    let baseline = style_profile.baseline_lift + random_unit(seed, 1) * 18.0;

    let top_peak = match glyph_kind {
        GlyphKind::Uppercase => style_profile.ascender_height,
        GlyphKind::Digit => style_profile.body_height + 170.0,
        GlyphKind::Descender => style_profile.body_height + 60.0,
        GlyphKind::Punctuation => 240.0,
        GlyphKind::Lowercase => body_top,
    };
    let bottom_depth = match glyph_kind {
        GlyphKind::Descender => -style_profile.descender_depth,
        GlyphKind::Punctuation => -18.0,
        _ => baseline,
    };

    let centerline = [
        (
            left_margin + random_unit(seed, 2) * 18.0,
            baseline + random_unit(seed, 3) * 12.0,
        ),
        (
            left_margin + (usable_width * 0.18),
            (body_top * 0.36) + random_unit(seed, 4) * style_profile.waviness,
        ),
        (
            left_margin + (usable_width * 0.34),
            top_peak + random_unit(seed, 5) * (style_profile.waviness * 0.8),
        ),
        (
            left_margin + (usable_width * 0.54),
            (body_top * 0.72) + random_unit(seed, 6) * style_profile.waviness,
        ),
        (
            left_margin + (usable_width * 0.76),
            (body_top * 0.46) + random_unit(seed, 7) * style_profile.waviness,
        ),
        (
            left_margin + usable_width + random_unit(seed, 8) * 14.0,
            bottom_depth + random_unit(seed, 9) * 18.0,
        ),
    ];

    let mut top_edge = Vec::with_capacity(centerline.len());
    let mut bottom_edge = Vec::with_capacity(centerline.len());

    for (index, point) in centerline.iter().enumerate() {
        let previous = if index == 0 {
            *point
        } else {
            centerline[index - 1]
        };
        let next = if index + 1 >= centerline.len() {
            *point
        } else {
            centerline[index + 1]
        };

        let tangent_x = next.0 - previous.0;
        let tangent_y = next.1 - previous.1;
        let length = tangent_x.hypot(tangent_y).max(1.0);
        let normal_x = -tangent_y / length;
        let normal_y = tangent_x / length;
        let index_ratio = (index as f32) / (centerline.len() as f32);
        let channel = 20_u32 + u32::try_from(index).unwrap_or(0);
        let stroke = style_profile.stroke_width
            * (index_ratio.mul_add(1.0, 0.72))
            * random_unit(seed, channel).mul_add(0.18, 1.0);
        let offset_x = normal_x * stroke;
        let offset_y = normal_y * stroke;
        let slanted_x = style_profile
            .slant
            .mul_add((point.1 - baseline) / 700.0, point.0);

        top_edge.push((
            round_to_i16(slanted_x + offset_x),
            round_to_i16(point.1 + offset_y),
        ));
        bottom_edge.push((
            round_to_i16(offset_x.mul_add(-0.78, slanted_x)),
            round_to_i16(offset_y.mul_add(-0.78, point.1)),
        ));
    }

    bottom_edge.reverse();
    top_edge.extend(bottom_edge);
    top_edge
}

#[derive(Debug, Clone, Copy)]
enum GlyphKind {
    Uppercase,
    Lowercase,
    Descender,
    Digit,
    Punctuation,
}

fn classify_glyph(character: char) -> GlyphKind {
    if character.is_ascii_uppercase() {
        return GlyphKind::Uppercase;
    }

    if character.is_ascii_digit() {
        return GlyphKind::Digit;
    }

    if matches!(character, 'g' | 'j' | 'p' | 'q' | 'y') {
        return GlyphKind::Descender;
    }

    if character.is_alphabetic() {
        return GlyphKind::Lowercase;
    }

    GlyphKind::Punctuation
}

fn random_unit(seed: u64, channel: u32) -> f32 {
    let value = seeded_unit(seed, channel);
    value.mul_add(2.0, -1.0)
}

#[allow(clippy::cast_precision_loss)]
fn seeded_unit(seed: u64, channel: u32) -> f32 {
    let shifted = seed.rotate_left(channel % 63);
    let masked = u16::try_from((shifted ^ u64::from(channel).wrapping_mul(0x9E37_79B9)) & 0xFFFF)
        .unwrap_or(0);
    f32::from(masked) / 65_535.0
}

#[allow(clippy::cast_possible_truncation)]
fn round_to_i16(value: f32) -> i16 {
    clamp_i32_to_i16(value.round() as i32)
}

fn algorithmic_contours(
    character: char,
    advance_width: u16,
    seed: u64,
    style_profile: StyleProfile,
) -> Vec<Vec<(i16, i16)>> {
    if character == ' ' {
        return Vec::new();
    }

    vec![build_handwritten_outline(
        character,
        advance_width,
        seed,
        style_profile,
    )]
}

const fn glyph_advance_width(character: char) -> u16 {
    match character {
        'A'..='Z' => 720,
        'a'..='z' | 'Ä' | 'Ö' | 'Ü' | 'ä' | 'ö' | 'ü' | 'ß' => 620,
        '0'..='9' => 600,
        _ => 460,
    }
}

fn decode_sheet(sample_image: &SampleImage) -> Option<GrayImage> {
    let cursor = Cursor::new(&sample_image.bytes);
    let reader = ImageReader::new(cursor).with_guessed_format().ok()?;
    let image = reader.decode().ok()?;
    Some(image.to_luma8())
}

#[allow(dead_code)]
fn extract_grid_glyph(sheet: &GrayImage, glyph_index: usize) -> Option<Vec<(i16, i16)>> {
    const SHEET_COLUMNS: usize = 17;
    const SHEET_ROWS: usize = 7;
    if glyph_index >= SHEET_COLUMNS * SHEET_ROWS {
        return None;
    }

    let sheet_width = usize::try_from(sheet.width()).ok()?;
    let sheet_height = usize::try_from(sheet.height()).ok()?;
    if sheet_width < SHEET_COLUMNS || sheet_height < SHEET_ROWS {
        return None;
    }

    let cell_column = glyph_index % SHEET_COLUMNS;
    let cell_row = glyph_index / SHEET_COLUMNS;

    let cell_width = sheet_width / SHEET_COLUMNS;
    let cell_height = sheet_height / SHEET_ROWS;
    if cell_width < 8 || cell_height < 8 {
        return None;
    }

    let margin_x = (cell_width / 8).max(2);
    let margin_y = (cell_height / 8).max(2);
    let start_x = cell_column * cell_width + margin_x;
    let start_y = cell_row * cell_height + margin_y;
    let end_x = ((cell_column + 1) * cell_width).saturating_sub(margin_x);
    let end_y = ((cell_row + 1) * cell_height).saturating_sub(margin_y);

    if end_x <= start_x || end_y <= start_y {
        return None;
    }

    let mut ink_points = Vec::new();
    for y in start_y..end_y {
        for x in start_x..end_x {
            if is_ink(*sheet.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?)) {
                ink_points.push((x, y));
            }
        }
    }

    if ink_points.len() < 24 {
        return None;
    }

    let min_x = ink_points.iter().map(|(x, _)| *x).min()?;
    let max_x = ink_points.iter().map(|(x, _)| *x).max()?;
    let min_y = ink_points.iter().map(|(_, y)| *y).min()?;
    let max_y = ink_points.iter().map(|(_, y)| *y).max()?;

    if max_x <= min_x || max_y <= min_y {
        return None;
    }

    let sample_columns = 18_usize;
    let width_span = (max_x - min_x + 1).max(sample_columns);
    let baseline_y = max_y;
    let height_span = (max_y - min_y + 1).max(1);

    let mut top_points = Vec::new();
    let mut bottom_points = Vec::new();

    for sample_index in 0..sample_columns {
        let local_x = min_x + ((sample_index * width_span) / sample_columns).min(width_span - 1);
        let column_range_end = (local_x + (width_span / sample_columns).max(1)).min(max_x + 1);

        let mut top_hit: Option<usize> = None;
        let mut bottom_hit: Option<usize> = None;

        for x in local_x..column_range_end {
            for y in min_y..=max_y {
                if is_ink(*sheet.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?)) {
                    top_hit = Some(top_hit.map_or(y, |current| current.min(y)));
                    bottom_hit = Some(bottom_hit.map_or(y, |current| current.max(y)));
                }
            }
        }

        let Some(top_y) = top_hit else {
            continue;
        };
        let bottom_y = bottom_hit.unwrap_or(top_y);

        let normalized_x = scale_to_units(local_x - min_x, width_span, 520) + 50;
        let normalized_top = scale_y_to_units(baseline_y.saturating_sub(top_y), height_span);
        let normalized_bottom = scale_y_to_units(baseline_y.saturating_sub(bottom_y), height_span);

        top_points.push((normalized_x, normalized_top));
        bottom_points.push((normalized_x, normalized_bottom.max(0)));
    }

    if top_points.len() < 4 {
        return None;
    }

    bottom_points.reverse();
    top_points.extend(bottom_points);

    Some(top_points)
}

#[allow(dead_code)]
fn extract_freeform_glyph(
    sheet: &GrayImage,
    glyph_index: usize,
    glyph_count: usize,
) -> Option<Vec<(i16, i16)>> {
    let sheet_width = usize::try_from(sheet.width()).ok()?;
    let sheet_height = usize::try_from(sheet.height()).ok()?;
    if sheet_width < 16 || sheet_height < 16 {
        return None;
    }

    let components = extract_ink_components(sheet, sheet_width, sheet_height)?;
    if components.is_empty() {
        return None;
    }

    let component_index = (glyph_index * components.len()) / glyph_count.max(1);
    let points = components.get(component_index.min(components.len().saturating_sub(1)))?;
    build_shape_from_points(points)
}

fn extract_ink_components(
    sheet: &GrayImage,
    sheet_width: usize,
    sheet_height: usize,
) -> Option<Vec<Vec<(usize, usize)>>> {
    let mut ink_points = Vec::new();
    let mut visited = vec![false; sheet_width.checked_mul(sheet_height)?];

    for y in 0..sheet_height {
        for x in 0..sheet_width {
            if !is_ink(*sheet.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?)) {
                continue;
            }

            let index = y.checked_mul(sheet_width)?.checked_add(x)?;
            if visited.get(index).copied().unwrap_or(true) {
                continue;
            }

            let component =
                flood_fill_component(sheet, sheet_width, sheet_height, x, y, &mut visited)?;
            if component.len() >= 24 {
                ink_points.push(component);
            }
        }
    }

    ink_points.sort_by_key(|points| {
        let min_y = points.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let min_x = points.iter().map(|(x, _)| *x).min().unwrap_or(0);
        (min_y / 24, min_x)
    });

    Some(ink_points)
}

fn flood_fill_component(
    sheet: &GrayImage,
    sheet_width: usize,
    sheet_height: usize,
    start_x: usize,
    start_y: usize,
    visited: &mut [bool],
) -> Option<Vec<(usize, usize)>> {
    let mut queue = std::collections::VecDeque::new();
    let mut points = Vec::new();

    queue.push_back((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        let index = y.checked_mul(sheet_width)?.checked_add(x)?;
        if *visited.get(index)? {
            continue;
        }

        visited[index] = true;
        if !is_ink(*sheet.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?)) {
            continue;
        }

        points.push((x, y));

        let x_start = x.saturating_sub(1);
        let x_end = (x + 1).min(sheet_width.saturating_sub(1));
        let y_start = y.saturating_sub(1);
        let y_end = (y + 1).min(sheet_height.saturating_sub(1));

        for next_y in y_start..=y_end {
            for next_x in x_start..=x_end {
                let next_index = next_y.checked_mul(sheet_width)?.checked_add(next_x)?;
                if !visited.get(next_index).copied().unwrap_or(true) {
                    queue.push_back((next_x, next_y));
                }
            }
        }
    }

    Some(points)
}

fn build_shape_from_points(points: &[(usize, usize)]) -> Option<Vec<(i16, i16)>> {
    if points.len() < 24 {
        return None;
    }

    let outline = sample_component_outline(points)?;
    let normalized_outline = normalize_outline_to_font(&outline)?;
    if normalized_outline.len() < 4 {
        return None;
    }

    Some(normalized_outline)
}

const fn is_ink(pixel: Luma<u8>) -> bool {
    pixel.0[0] < 210
}

fn sample_component_outline(points: &[(usize, usize)]) -> Option<Vec<(usize, usize)>> {
    let min_x = points.iter().map(|(x, _)| *x).min()?;
    let max_x = points.iter().map(|(x, _)| *x).max()?;
    let min_y = points.iter().map(|(_, y)| *y).min()?;
    let max_y = points.iter().map(|(_, y)| *y).max()?;
    if max_x <= min_x || max_y <= min_y {
        return None;
    }

    let sample_columns = 24_usize;
    let width_span = (max_x - min_x + 1).max(sample_columns);
    let mut top_points = Vec::new();
    let mut bottom_points = Vec::new();

    for sample_index in 0..sample_columns {
        let local_x = min_x + ((sample_index * width_span) / sample_columns).min(width_span - 1);
        let column_range_end = (local_x + (width_span / sample_columns).max(1)).min(max_x + 1);

        let mut top_hit: Option<usize> = None;
        let mut bottom_hit: Option<usize> = None;

        for x in local_x..column_range_end {
            for &(point_x, point_y) in points {
                if point_x != x {
                    continue;
                }

                top_hit = Some(top_hit.map_or(point_y, |current| current.min(point_y)));
                bottom_hit = Some(bottom_hit.map_or(point_y, |current| current.max(point_y)));
            }
        }

        let Some(top_y) = top_hit else {
            continue;
        };
        let bottom_y = bottom_hit.unwrap_or(top_y);

        top_points.push((local_x, top_y));
        bottom_points.push((local_x, bottom_y));
    }

    if top_points.len() < 4 {
        return None;
    }

    bottom_points.reverse();
    top_points.extend(bottom_points);
    Some(top_points)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::similar_names
)]
fn normalize_outline_to_font(points: &[(usize, usize)]) -> Option<Vec<(i16, i16)>> {
    let min_x = points.iter().map(|(x, _)| *x).min()?;
    let max_x = points.iter().map(|(x, _)| *x).max()?;
    let min_y = points.iter().map(|(_, y)| *y).min()?;
    let max_y = points.iter().map(|(_, y)| *y).max()?;
    if max_x <= min_x || max_y <= min_y {
        return None;
    }

    let width_span = max_x - min_x + 1;
    let height_span = max_y - min_y + 1;
    let target_width = 520_i32;
    let target_height = 700_i32;
    let width_i32 = i32::try_from(width_span).ok()?;
    let height_i32 = i32::try_from(height_span).ok()?;
    if width_i32 <= 0 || height_i32 <= 0 {
        return None;
    }

    let scale_x = (target_width as f32) / (width_i32 as f32);
    let scale_y = (target_height as f32) / (height_i32 as f32);
    let scale = scale_x.min(scale_y);
    let resized_width = ((width_i32 as f32) * scale).round() as i32;
    let resized_height = ((height_i32 as f32) * scale).round() as i32;
    let horizontal_padding = ((target_width - resized_width) / 2).max(0);
    let baseline = resized_height.max(1);

    let normalized = points
        .iter()
        .map(|(x, y)| {
            let local_x = i32::try_from(x.saturating_sub(min_x)).unwrap_or(0);
            let local_y = i32::try_from(y.saturating_sub(min_y)).unwrap_or(0);
            let mapped_x = ((local_x as f32) * scale).round() as i32;
            let mapped_y = ((local_y as f32) * scale).round() as i32;
            (
                clamp_i32_to_i16(horizontal_padding + mapped_x + 50),
                clamp_i32_to_i16((baseline - mapped_y).max(0)),
            )
        })
        .collect::<Vec<_>>();

    Some(normalized)
}

#[allow(dead_code)]
fn scale_to_units(value: usize, full_span: usize, target_span: i16) -> i16 {
    if full_span == 0 {
        return 0;
    }

    let numerator = i32::try_from(value).unwrap_or(0) * i32::from(target_span);
    let denominator = i32::try_from(full_span).unwrap_or(1);
    clamp_i32_to_i16(numerator / denominator)
}

#[allow(dead_code)]
fn scale_y_to_units(value: usize, full_span: usize) -> i16 {
    scale_to_units(value, full_span, 700)
}

fn build_simple_polygon_glyph(
    advance_width: u16,
    left_side_bearing: i16,
    points: &[(i16, i16)],
) -> GlyphDefinition {
    build_multi_contour_glyph(advance_width, left_side_bearing, &[points.to_vec()])
}

fn build_multi_contour_glyph(
    advance_width: u16,
    left_side_bearing: i16,
    contours: &[Vec<(i16, i16)>],
) -> GlyphDefinition {
    let (x_min, y_min, x_max, y_max) = contour_bounds(contours);
    let point_count = contours.iter().map(std::vec::Vec::len).sum::<usize>();

    let mut data = Vec::new();
    let contour_count = i16::try_from(contours.len()).unwrap_or(i16::MAX);
    push_i16(&mut data, contour_count);
    push_i16(&mut data, x_min);
    push_i16(&mut data, y_min);
    push_i16(&mut data, x_max);
    push_i16(&mut data, y_max);

    let mut point_offset = 0_usize;
    for contour in contours {
        point_offset = point_offset.saturating_add(contour.len());
        let end_point = u16::try_from(point_offset.saturating_sub(1)).unwrap_or(u16::MAX);
        push_u16(&mut data, end_point);
    }
    push_u16(&mut data, 0);

    data.extend(std::iter::repeat_n(0x01, point_count));

    let mut previous_x = 0_i16;
    for contour in contours {
        for (x, _) in contour {
            push_i16(&mut data, *x - previous_x);
            previous_x = *x;
        }
    }

    let mut previous_y = 0_i16;
    for contour in contours {
        for (_, y) in contour {
            push_i16(&mut data, *y - previous_y);
            previous_y = *y;
        }
    }

    if data.len() % 2 != 0 {
        data.push(0);
    }

    GlyphDefinition {
        advance_width,
        left_side_bearing,
        x_min,
        y_min,
        x_max,
        y_max,
        data,
    }
}

fn empty_glyph_data() -> Vec<u8> {
    let mut data = Vec::new();
    push_i16(&mut data, 0);
    push_i16(&mut data, 0);
    push_i16(&mut data, 0);
    push_i16(&mut data, 0);
    push_i16(&mut data, 0);
    data
}

fn contour_bounds(contours: &[Vec<(i16, i16)>]) -> (i16, i16, i16, i16) {
    let x_min = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(x, _)| *x))
        .min()
        .unwrap_or(0);
    let y_min = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(_, y)| *y))
        .min()
        .unwrap_or(0);
    let x_max = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(x, _)| *x))
        .max()
        .unwrap_or(0);
    let y_max = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(_, y)| *y))
        .max()
        .unwrap_or(0);
    (x_min, y_min, x_max, y_max)
}

fn build_preview_svg_from_generated(
    generated_glyphs: &[GeneratedGlyph],
    preview_text: &str,
) -> String {
    if preview_text.trim().is_empty() {
        return String::new();
    }

    let glyph_map = generated_glyphs
        .iter()
        .map(|glyph| (glyph.character, glyph))
        .collect::<HashMap<_, _>>();
    let preview_scale = 0.075_f32;
    let line_height = 108_i32;
    let baseline_offset = 88_i32;
    let left_padding = 40_i32;
    let max_line_width = 2400_i32;
    let mut x = left_padding;
    let mut y = baseline_offset;
    let mut max_x = left_padding;
    let mut max_y = baseline_offset + 80;
    let mut path_data = String::new();

    for line in preview_text.lines() {
        let segments = split_preview_segments(line);
        for segment in &segments {
            let segment_width = preview_segment_width(segment, &glyph_map, preview_scale);
            if !segment.chars().all(char::is_whitespace)
                && x > left_padding
                && x + segment_width > max_line_width
            {
                x = left_padding;
                y += line_height;
                max_y = max_y.max(y + 80);
            }

            for character in segment.chars() {
                let Some(glyph) = glyph_map.get(&character) else {
                    x += scale_preview_value(220, preview_scale);
                    max_x = max_x.max(x);
                    continue;
                };

                let advance_width =
                    scale_preview_value(i32::from(glyph.advance_width), preview_scale);

                for contour in &glyph.contours {
                    if contour.len() < 2 {
                        continue;
                    }

                    let mut contour_iter = contour.iter();
                    let Some((first_x, first_y)) = contour_iter.next() else {
                        continue;
                    };
                    let _ = write!(
                        path_data,
                        "M{} {}",
                        x + scale_preview_value(i32::from(*first_x), preview_scale),
                        y - scale_preview_value(i32::from(*first_y), preview_scale)
                    );
                    for (point_x, point_y) in contour_iter {
                        let _ = write!(
                            path_data,
                            " L{} {}",
                            x + scale_preview_value(i32::from(*point_x), preview_scale),
                            y - scale_preview_value(i32::from(*point_y), preview_scale)
                        );
                    }
                    path_data.push_str(" Z ");
                }

                x += advance_width;
                max_x = max_x.max(x);
                max_y = max_y.max(y + 120);
            }
        }

        x = left_padding;
        y += line_height;
        max_y = max_y.max(y + 80);
    }

    if path_data.trim().is_empty() {
        return String::new();
    }

    let svg_width = max_x.max(480) + 40;
    let svg_height = max_y.max(420) + 40;

    format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" ",
            "viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" ",
            "style=\"display:block;width:100%;height:auto\" ",
            "preserveAspectRatio=\"xMinYMin meet\" ",
            "role=\"img\" aria-label=\"Inkform preview\">",
            "<rect width=\"100%\" height=\"100%\" fill=\"#f7efe3\"/>",
            "<path d=\"{}\" fill=\"#1f1611\"/>",
            "</svg>"
        ),
        svg_width, svg_height, svg_width, svg_height, path_data
    )
}

fn split_preview_segments(line: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0_usize;
    let mut in_whitespace = false;

    for (index, character) in line.char_indices() {
        if index == 0 {
            in_whitespace = character.is_whitespace();
            continue;
        }

        if character.is_whitespace() == in_whitespace {
            continue;
        }

        segments.push(&line[start..index]);
        start = index;
        in_whitespace = character.is_whitespace();
    }

    if start < line.len() {
        segments.push(&line[start..]);
    }

    segments
}

fn preview_segment_width(
    segment: &str,
    glyph_map: &HashMap<char, &GeneratedGlyph>,
    preview_scale: f32,
) -> i32 {
    segment.chars().fold(0_i32, |accumulator, character| {
        let next_width = glyph_map
            .get(&character)
            .map_or(220, |glyph| i32::from(glyph.advance_width));
        accumulator + scale_preview_value(next_width, preview_scale)
    })
}

#[allow(clippy::cast_precision_loss)]
fn scale_preview_value(value: i32, scale: f32) -> i32 {
    round_to_i16((value as f32) * scale).into()
}

fn build_head_table(metrics: FontMetrics, loca_format: i16) -> Vec<u8> {
    let mut table = Vec::new();
    push_u32(&mut table, 0x0001_0000);
    push_u32(&mut table, 0x0001_0000);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0x5F0F_3CF5);
    push_u16(&mut table, 0x000B);
    push_u16(&mut table, UNITS_PER_EM);
    push_u64(&mut table, 0);
    push_u64(&mut table, 0);
    push_i16(&mut table, metrics.x_min);
    push_i16(&mut table, metrics.y_min);
    push_i16(&mut table, metrics.x_max);
    push_i16(&mut table, metrics.y_max);
    push_u16(&mut table, 0);
    push_u16(&mut table, 8);
    push_i16(&mut table, 2);
    push_i16(&mut table, loca_format);
    push_i16(&mut table, 0);
    table
}

fn build_hhea_table(metrics: FontMetrics, glyph_count: usize) -> Vec<u8> {
    let mut table = Vec::new();
    push_u32(&mut table, 0x0001_0000);
    push_i16(&mut table, ASCENDER);
    push_i16(&mut table, DESCENDER);
    push_i16(&mut table, LINE_GAP);
    push_u16(&mut table, metrics.advance_width_max);
    push_i16(&mut table, metrics.min_left_side_bearing);
    push_i16(&mut table, metrics.min_right_side_bearing);
    push_i16(&mut table, metrics.x_max_extent);
    push_i16(&mut table, 1);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    push_i16(&mut table, 0);
    let number_of_h_metrics = u16::try_from(glyph_count).unwrap_or(u16::MAX);
    push_u16(&mut table, number_of_h_metrics);
    table
}

fn build_maxp_table(glyphs: &[GlyphDefinition]) -> Vec<u8> {
    let max_points = glyphs.iter().map(max_point_count).max().unwrap_or(0);
    let max_contours = glyphs.iter().map(contour_count).max().unwrap_or(0);

    let mut table = Vec::new();
    push_u32(&mut table, 0x0001_0000);
    push_u16(&mut table, u16::try_from(glyphs.len()).unwrap_or(u16::MAX));
    push_u16(&mut table, max_points);
    push_u16(&mut table, max_contours);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 2);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    push_u16(&mut table, 0);
    table
}

fn max_point_count(glyph: &GlyphDefinition) -> u16 {
    if glyph.data.len() <= 10 {
        return 0;
    }

    let flag_count = glyph.data.len().saturating_sub(14) / 5;
    u16::try_from(flag_count).unwrap_or(u16::MAX)
}

fn contour_count(glyph: &GlyphDefinition) -> u16 {
    if glyph.data.len() < 2 {
        return 0;
    }

    let contour_total = i16::from_be_bytes([glyph.data[0], glyph.data[1]]);
    if contour_total <= 0 {
        return 0;
    }

    u16::try_from(contour_total).unwrap_or(0)
}

fn build_hmtx_table(glyphs: &[GlyphDefinition]) -> Vec<u8> {
    let mut table = Vec::new();
    for glyph in glyphs {
        push_u16(&mut table, glyph.advance_width);
        push_i16(&mut table, glyph.left_side_bearing);
    }
    table
}

fn build_glyf_and_loca_tables(glyphs: &[GlyphDefinition]) -> (Vec<u8>, Vec<u8>, i16) {
    let mut glyf = Vec::new();
    let mut offsets = Vec::with_capacity(glyphs.len() + 1);

    for glyph in glyphs {
        offsets.push(glyf.len());
        glyf.extend_from_slice(&glyph.data);
        if glyf.len() % 2 != 0 {
            glyf.push(0);
        }
    }
    offsets.push(glyf.len());

    let use_short_loca = offsets
        .iter()
        .all(|offset| offset % 2 == 0 && u16::try_from(*offset / 2).is_ok());

    let mut loca = Vec::new();
    if use_short_loca {
        for offset in offsets {
            push_u16(&mut loca, u16::try_from(offset / 2).unwrap_or(u16::MAX));
        }
        (glyf, loca, 0)
    } else {
        for offset in offsets {
            push_u32(&mut loca, u32::try_from(offset).unwrap_or(u32::MAX));
        }
        (glyf, loca, 1)
    }
}

fn build_cmap_table(script_pack: &ScriptPack) -> Vec<u8> {
    let mut sorted_pairs = script_pack
        .glyphs
        .iter()
        .enumerate()
        .map(|(index, character)| {
            let codepoint = u16::try_from(u32::from(*character)).unwrap_or(0);
            let glyph_index = u16::try_from(index + 1).unwrap_or(u16::MAX);
            (codepoint, glyph_index)
        })
        .collect::<Vec<_>>();
    sorted_pairs.sort_by_key(|(codepoint, _)| *codepoint);

    let seg_count = u16::try_from(sorted_pairs.len() + 1).unwrap_or(u16::MAX);
    let seg_count_x2 = seg_count.saturating_mul(2);
    let search_params = search_params(seg_count);
    let format4_length = 16_u16.saturating_add(seg_count.saturating_mul(8));

    let mut subtable = Vec::new();
    push_u16(&mut subtable, 4);
    push_u16(&mut subtable, format4_length);
    push_u16(&mut subtable, 0);
    push_u16(&mut subtable, seg_count_x2);
    push_u16(&mut subtable, search_params.search_range);
    push_u16(&mut subtable, search_params.entry_selector);
    push_u16(&mut subtable, search_params.range_shift);

    for (codepoint, _) in &sorted_pairs {
        push_u16(&mut subtable, *codepoint);
    }
    push_u16(&mut subtable, 0xFFFF);
    push_u16(&mut subtable, 0);

    for (codepoint, _) in &sorted_pairs {
        push_u16(&mut subtable, *codepoint);
    }
    push_u16(&mut subtable, 0xFFFF);

    for (codepoint, glyph_index) in &sorted_pairs {
        let delta = glyph_index.wrapping_sub(*codepoint);
        push_u16(&mut subtable, delta);
    }
    push_u16(&mut subtable, 1);

    for _ in 0..usize::from(seg_count) {
        push_u16(&mut subtable, 0);
    }

    let subtable_offset = 4_u32 + (2 * 8) as u32;
    let mut table = Vec::new();
    push_u16(&mut table, 0);
    push_u16(&mut table, 2);

    push_u16(&mut table, 0);
    push_u16(&mut table, 3);
    push_u32(&mut table, subtable_offset);

    push_u16(&mut table, 3);
    push_u16(&mut table, 1);
    push_u32(&mut table, subtable_offset);

    table.extend_from_slice(&subtable);
    table
}

fn build_name_table(family_name: &str) -> Vec<u8> {
    let records = [
        (1_u16, family_name),
        (2_u16, "Regular"),
        (4_u16, family_name),
        (6_u16, "InkformPreview-Regular"),
    ];

    let mut storage = Vec::new();
    let mut record_bytes = Vec::new();

    for (name_id, value) in records {
        let encoded = encode_utf16be(value);
        push_u16(&mut record_bytes, 3);
        push_u16(&mut record_bytes, 1);
        push_u16(&mut record_bytes, 0x0409);
        push_u16(&mut record_bytes, name_id);
        push_u16(
            &mut record_bytes,
            u16::try_from(encoded.len()).unwrap_or(u16::MAX),
        );
        push_u16(
            &mut record_bytes,
            u16::try_from(storage.len()).unwrap_or(u16::MAX),
        );
        storage.extend_from_slice(&encoded);
    }

    let mut table = Vec::new();
    push_u16(&mut table, 0);
    push_u16(&mut table, u16::try_from(records.len()).unwrap_or(0));
    let storage_offset = 6_u16.saturating_add(u16::try_from(record_bytes.len()).unwrap_or(0));
    push_u16(&mut table, storage_offset);
    table.extend_from_slice(&record_bytes);
    table.extend_from_slice(&storage);
    table
}

fn build_post_table() -> Vec<u8> {
    let mut table = Vec::new();
    push_u32(&mut table, 0x0003_0000);
    push_u32(&mut table, 0);
    push_i16(&mut table, -75);
    push_i16(&mut table, 50);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    table
}

fn build_os2_table(metrics: FontMetrics) -> Vec<u8> {
    let mut table = Vec::new();
    push_u16(&mut table, 0);
    push_i16(&mut table, metrics.x_avg_char_width);
    push_u16(&mut table, 400);
    push_u16(&mut table, 5);
    push_u16(&mut table, 0);
    push_i16(&mut table, 650);
    push_i16(&mut table, 699);
    push_i16(&mut table, 0);
    push_i16(&mut table, 140);
    push_i16(&mut table, 650);
    push_i16(&mut table, 699);
    push_i16(&mut table, 0);
    push_i16(&mut table, 140);
    push_i16(&mut table, 80);
    push_i16(&mut table, 50);
    push_i16(&mut table, 0);
    table.extend_from_slice(&[2, 11, 6, 3, 5, 4, 5, 2, 3, 4]);
    push_u32(&mut table, 0xE000_02FF);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    push_u32(&mut table, 0);
    table.extend_from_slice(b"INKF");
    push_u16(&mut table, 0x0040);
    push_u16(&mut table, 32);
    push_u16(&mut table, 255);
    push_i16(&mut table, ASCENDER);
    push_i16(&mut table, DESCENDER);
    push_i16(&mut table, LINE_GAP);
    push_u16(&mut table, u16::try_from(ASCENDER).unwrap_or(0));
    push_u16(&mut table, u16::try_from(-DESCENDER).unwrap_or(0));
    table
}

fn build_font_file(mut tables: Vec<TableRecord>) -> Vec<u8> {
    tables.sort_by_key(|table| table.tag);
    let num_tables = u16::try_from(tables.len()).unwrap_or(0);
    let params = search_params(num_tables);

    let mut directory = Vec::new();
    let mut body = Vec::new();
    let mut head_offset = 0_usize;

    let mut current_offset = 12_usize + (tables.len() * 16);

    for table in &tables {
        let checksum = table_checksum(&table.data);
        let length = table.data.len();
        let padded_length = align_to(length, 4);

        directory.extend_from_slice(&table.tag);
        push_u32(&mut directory, checksum);
        push_u32(&mut directory, u32::try_from(current_offset).unwrap_or(0));
        push_u32(&mut directory, u32::try_from(length).unwrap_or(0));

        if table.tag == *b"head" {
            head_offset = current_offset;
        }

        body.extend_from_slice(&table.data);
        while body.len() % 4 != 0 {
            body.push(0);
        }

        current_offset += padded_length;
    }

    let mut font = Vec::new();
    push_u32(&mut font, 0x0001_0000);
    push_u16(&mut font, num_tables);
    push_u16(&mut font, params.search_range);
    push_u16(&mut font, params.entry_selector);
    push_u16(&mut font, params.range_shift);
    font.extend_from_slice(&directory);
    font.extend_from_slice(&body);

    let checksum_adjustment_offset = head_offset + 8;
    write_u32_at(&mut font, checksum_adjustment_offset, 0);
    let checksum = table_checksum(&font);
    let adjustment = 0xB1B0_AFBA_u32.wrapping_sub(checksum);
    write_u32_at(&mut font, checksum_adjustment_offset, adjustment);

    font
}

#[derive(Debug, Clone, Copy)]
struct SearchParams {
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
}

fn search_params(item_count: u16) -> SearchParams {
    let mut power = 1_u16;
    let mut selector = 0_u16;

    while power.saturating_mul(2) <= item_count.max(1) {
        power = power.saturating_mul(2);
        selector = selector.saturating_add(1);
    }

    let search_range = power.saturating_mul(16);
    let range_shift = item_count.saturating_mul(16).saturating_sub(search_range);

    SearchParams {
        search_range,
        entry_selector: selector,
        range_shift,
    }
}

fn table_checksum(data: &[u8]) -> u32 {
    let padded_length = align_to(data.len(), 4);
    let mut checksum = 0_u32;

    for chunk_index in (0..padded_length).step_by(4) {
        let mut word = [0_u8; 4];
        let remaining = data.len().saturating_sub(chunk_index).min(4);
        if remaining > 0 {
            word[..remaining].copy_from_slice(&data[chunk_index..chunk_index + remaining]);
        }
        checksum = checksum.wrapping_add(u32::from_be_bytes(word));
    }

    checksum
}

fn encode_utf16be(value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for unit in value.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    bytes
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        hash.wrapping_mul(0x1000_0000_01b3)
            .wrapping_add(u64::from(*byte))
    })
}

fn mix_seed(base_seed: u64, character: char) -> u64 {
    base_seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(u32::from(character)))
}

const fn align_to(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}

fn write_u32_at(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn clamp_i32_to_i16(value: i32) -> i16 {
    if value > i32::from(i16::MAX) {
        i16::MAX
    } else if value < i32::from(i16::MIN) {
        i16::MIN
    } else {
        i16::try_from(value).map_or(0, |value| value)
    }
}
