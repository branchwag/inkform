use crate::domain::SampleImage;
use image::{GrayImage, ImageReader, Luma};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;

const MAX_BOUNDARY_POINTS: usize = 96;

#[derive(Debug, Clone)]
pub struct ExtractedGlyph {
    pub outline: Vec<(i16, i16)>,
    pub width_ratio: f32,
    pub height_ratio: f32,
    pub slant: f32,
    pub density: f32,
    pub baseline_offset: f32,
    pub ink_area: f32,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub glyphs: Vec<ExtractedGlyph>,
}

pub fn extract_handwriting(sample_image: &SampleImage) -> Option<ExtractionResult> {
    let image = decode_image(sample_image)?;
    let ink_threshold = estimate_ink_threshold(&image);
    let components = extract_ink_components(&image, ink_threshold)?;

    let mut glyphs = Vec::new();
    for points in components {
        if !is_handwriting_component(&points) {
            continue;
        }

        let outline = normalize_outline_to_font(&trace_component_boundary(&points)?)?;
        let measures = measure_component(&points)?;
        glyphs.push(ExtractedGlyph {
            outline,
            width_ratio: measures.width_ratio,
            height_ratio: measures.height_ratio,
            slant: measures.slant,
            density: measures.density,
            baseline_offset: measures.baseline_offset,
            ink_area: measures.ink_area,
        });
    }

    if glyphs.is_empty() {
        return None;
    }

    Some(ExtractionResult { glyphs })
}

fn decode_image(sample_image: &SampleImage) -> Option<GrayImage> {
    let cursor = Cursor::new(&sample_image.bytes);
    let reader = ImageReader::new(cursor).with_guessed_format().ok()?;
    let image = reader.decode().ok()?;
    Some(image.to_luma8())
}

fn extract_ink_components(
    image: &GrayImage,
    ink_threshold: u8,
) -> Option<Vec<Vec<(usize, usize)>>> {
    let width = usize::try_from(image.width()).ok()?;
    let height = usize::try_from(image.height()).ok()?;
    let mut visited = vec![false; width.checked_mul(height)?];
    let mut components = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let index = y.checked_mul(width)?.checked_add(x)?;
            if visited.get(index).copied().unwrap_or(true)
                || !is_ink(
                    *image.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?),
                    ink_threshold,
                )
            {
                continue;
            }

            let component = flood_fill_component(image, ink_threshold, x, y, &mut visited)?;
            if component.len() >= 24 {
                components.push(component);
            }
        }
    }

    components.sort_by_key(|points| {
        let min_y = points.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let min_x = points.iter().map(|(x, _)| *x).min().unwrap_or(0);
        (min_y, min_x)
    });
    Some(components)
}

fn flood_fill_component(
    image: &GrayImage,
    ink_threshold: u8,
    start_x: usize,
    start_y: usize,
    visited: &mut [bool],
) -> Option<Vec<(usize, usize)>> {
    let width = usize::try_from(image.width()).ok()?;
    let height = usize::try_from(image.height()).ok()?;
    let mut queue = std::collections::VecDeque::from([(start_x, start_y)]);
    let mut points = Vec::new();

    while let Some((x, y)) = queue.pop_front() {
        let index = y.checked_mul(width)?.checked_add(x)?;
        if visited.get(index).copied().unwrap_or(true) {
            continue;
        }
        visited[index] = true;
        if !is_ink(
            *image.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?),
            ink_threshold,
        ) {
            continue;
        }

        points.push((x, y));
        for next_y in y.saturating_sub(1)..=(y + 1).min(height.saturating_sub(1)) {
            for next_x in x.saturating_sub(1)..=(x + 1).min(width.saturating_sub(1)) {
                let next_index = next_y.checked_mul(width)?.checked_add(next_x)?;
                if !visited.get(next_index).copied().unwrap_or(true) {
                    queue.push_back((next_x, next_y));
                }
            }
        }
    }

    Some(points)
}

#[allow(clippy::cast_precision_loss)]
fn is_handwriting_component(points: &[(usize, usize)]) -> bool {
    let Some(min_x) = points.iter().map(|(x, _)| *x).min() else {
        return false;
    };
    let Some(max_x) = points.iter().map(|(x, _)| *x).max() else {
        return false;
    };
    let Some(min_y) = points.iter().map(|(_, y)| *y).min() else {
        return false;
    };
    let Some(max_y) = points.iter().map(|(_, y)| *y).max() else {
        return false;
    };

    let width = max_x.saturating_sub(min_x) + 1;
    let height = max_y.saturating_sub(min_y) + 1;
    let area = width.saturating_mul(height);
    let density = if area == 0 {
        1.0
    } else {
        points.len() as f32 / area as f32
    };

    let looks_like_dense_dot = width < 30 && height < 30 && density > 0.42;
    width >= 14 && height >= 14 && !looks_like_dense_dot
}

fn estimate_ink_threshold(image: &GrayImage) -> u8 {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for pixel in image.pixels() {
        total = total.saturating_add(u64::from(pixel.0[0]));
        count = count.saturating_add(1);
    }

    if count == 0 {
        return 210;
    }

    let average = u8::try_from(total / count).unwrap_or(210);
    average.saturating_sub(28).clamp(96, 216)
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
        ink_area: density,
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

const fn is_ink(pixel: Luma<u8>, ink_threshold: u8) -> bool {
    pixel.0[0] <= ink_threshold
}

fn trace_component_boundary(points: &[(usize, usize)]) -> Option<Vec<(usize, usize)>> {
    type Point = (usize, usize);

    let ink = points.iter().copied().collect::<HashSet<_>>();
    if ink.is_empty() {
        return None;
    }

    let mut edges = Vec::new();
    for &(x, y) in &ink {
        if !ink.contains(&(x, y.saturating_sub(1))) {
            edges.push(((x, y), (x.saturating_add(1), y)));
        }
        if !ink.contains(&(x.saturating_add(1), y)) {
            edges.push((
                (x.saturating_add(1), y),
                (x.saturating_add(1), y.saturating_add(1)),
            ));
        }
        if !ink.contains(&(x, y.saturating_add(1))) {
            edges.push((
                (x.saturating_add(1), y.saturating_add(1)),
                (x, y.saturating_add(1)),
            ));
        }
        if !ink.contains(&(x.saturating_sub(1), y)) {
            edges.push(((x, y.saturating_add(1)), (x, y)));
        }
    }

    let mut outgoing: HashMap<Point, Vec<usize>> = HashMap::new();
    for (index, (start, _)) in edges.iter().enumerate() {
        outgoing.entry(*start).or_default().push(index);
    }

    let mut used = vec![false; edges.len()];
    let mut largest_loop = Vec::new();
    for edge_index in 0..edges.len() {
        if used.get(edge_index).copied().unwrap_or(true) {
            continue;
        }

        let start = edges.get(edge_index)?.0;
        let mut current = start;
        let mut loop_points = Vec::new();
        loop {
            let candidates = outgoing.get(&current)?;
            let next_edge = candidates
                .iter()
                .copied()
                .find(|candidate| !used.get(*candidate).copied().unwrap_or(true));
            let Some(next_edge) = next_edge else {
                break;
            };

            used[next_edge] = true;
            loop_points.push(current);
            current = edges.get(next_edge)?.1;
            if current == start {
                break;
            }
        }

        if loop_points.len() > largest_loop.len() {
            largest_loop = loop_points;
        }
    }

    simplify_outline(&largest_loop)
}

fn simplify_outline(points: &[(usize, usize)]) -> Option<Vec<(usize, usize)>> {
    let mut deduplicated = Vec::with_capacity(points.len());
    for point in points {
        if deduplicated.last().copied() != Some(*point) {
            deduplicated.push(*point);
        }
    }

    if deduplicated.len() < 4 {
        return None;
    }

    if deduplicated.len() <= MAX_BOUNDARY_POINTS {
        return Some(deduplicated);
    }

    let stride = deduplicated.len().div_ceil(MAX_BOUNDARY_POINTS);
    let simplified = deduplicated
        .iter()
        .step_by(stride)
        .copied()
        .collect::<Vec<_>>();
    if simplified.len() < 4 {
        return None;
    }

    Some(simplified)
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

fn clamp_i32_to_i16(value: i32) -> i16 {
    i16::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i16::MIN
        } else {
            i16::MAX
        }
    })
}

#[cfg(test)]
mod tests {
    use super::extract_handwriting;
    use crate::domain::{SampleImage, SampleQuality};
    use image::{ColorType, GrayImage, ImageEncoder, Luma, codecs::png::PngEncoder};

    #[test]
    fn extracts_multiple_glyph_shapes_from_synthetic_image() {
        let mut image = GrayImage::from_pixel(240, 120, Luma([255_u8]));
        draw_rect(&mut image, 24, 20, 22, 58);
        draw_rect(&mut image, 60, 18, 14, 62);
        draw_rect(&mut image, 106, 24, 34, 42);
        draw_rect(&mut image, 160, 60, 44, 18);

        let bytes = encode_png(&image);
        let sample = SampleImage {
            width: image.width(),
            height: image.height(),
            bytes,
            quality: SampleQuality::Clean,
        };

        let result = extract_handwriting(&sample);
        assert!(result.is_some(), "synthetic handwriting should extract");
        let Some(extracted) = result else {
            panic!("missing extraction result");
        };

        assert!(
            extracted.glyphs.len() >= 3,
            "expected at least 3 extracted glyphs, got {}",
            extracted.glyphs.len()
        );
        assert!(
            extracted.glyphs.iter().any(|glyph| glyph.width_ratio > 1.2),
            "expected at least one wide extracted glyph"
        );
    }

    fn draw_rect(image: &mut GrayImage, x: u32, y: u32, width: u32, height: u32) {
        for row in y..(y + height) {
            for column in x..(x + width) {
                image.put_pixel(column, row, Luma([0_u8]));
            }
        }
    }

    fn encode_png(image: &GrayImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        let result = encoder.write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ColorType::L8.into(),
        );
        assert!(result.is_ok(), "png encoding should succeed: {result:?}");
        bytes
    }
}
