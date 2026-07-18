use crate::domain::SampleImage;
use image::{GrayImage, ImageReader, Luma};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;

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
    let crop = ink_bounds(&image, ink_threshold)?;
    let line_spans = segment_lines(&image, crop, ink_threshold);
    if line_spans.is_empty() {
        return None;
    }

    let mut glyphs = Vec::new();
    for (row_start, row_end) in line_spans {
        let spans = segment_line_spans(&image, crop, row_start, row_end, ink_threshold);
        for (column_start, column_end) in spans {
            let points = collect_ink_points(
                &image,
                column_start,
                column_end,
                row_start,
                row_end,
                ink_threshold,
            )?;
            if points.len() < 24 {
                continue;
            }

            let outline = normalize_outline_to_font(&trace_component_outline(&points)?)?;
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

fn ink_bounds(image: &GrayImage, ink_threshold: u8) -> Option<(usize, usize, usize, usize)> {
    let width = usize::try_from(image.width()).ok()?;
    let height = usize::try_from(image.height()).ok()?;
    let mut min_x = width;
    let mut max_x = 0_usize;
    let mut min_y = height;
    let mut max_y = 0_usize;
    let mut found = false;

    for y in 0..height {
        for x in 0..width {
            if !is_ink(
                *image.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?),
                ink_threshold,
            ) {
                continue;
            }

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            found = true;
        }
    }

    if !found {
        return None;
    }

    Some((
        min_x.saturating_sub(6),
        (max_x + 6).min(width.saturating_sub(1)),
        min_y.saturating_sub(6),
        (max_y + 6).min(height.saturating_sub(1)),
    ))
}

fn segment_lines(
    image: &GrayImage,
    crop: (usize, usize, usize, usize),
    ink_threshold: u8,
) -> Vec<(usize, usize)> {
    let (min_x, max_x, min_y, max_y) = crop;
    let width = max_x.saturating_sub(min_x) + 1;
    let min_row_ink = (width / 80).max(2);
    let mut spans = Vec::new();
    let mut active_start: Option<usize> = None;

    for y in min_y..=max_y {
        let mut row_ink = 0_usize;
        for x in min_x..=max_x {
            let pixel =
                image.get_pixel(u32::try_from(x).unwrap_or(0), u32::try_from(y).unwrap_or(0));
            if is_ink(*pixel, ink_threshold) {
                row_ink = row_ink.saturating_add(1);
            }
        }

        if row_ink >= min_row_ink {
            if active_start.is_none() {
                active_start = Some(y);
            }
            continue;
        }

        if let Some(start) = active_start.take()
            && y.saturating_sub(start) >= 8
        {
            spans.push((start.saturating_sub(4), y.min(max_y)));
        }
    }

    if let Some(start) = active_start {
        spans.push((start.saturating_sub(4), max_y));
    }

    spans
}

fn segment_line_spans(
    image: &GrayImage,
    crop: (usize, usize, usize, usize),
    row_start: usize,
    row_end: usize,
    ink_threshold: u8,
) -> Vec<(usize, usize)> {
    let (min_x, max_x, _, _) = crop;
    let band_height = row_end.saturating_sub(row_start).max(1);
    let mut projection = Vec::new();

    for x in min_x..=max_x {
        let mut column_ink = 0_usize;
        for y in row_start..=row_end {
            let pixel =
                image.get_pixel(u32::try_from(x).unwrap_or(0), u32::try_from(y).unwrap_or(0));
            if is_ink(*pixel, ink_threshold) {
                column_ink = column_ink.saturating_add(1);
            }
        }
        projection.push(column_ink);
    }

    let mut spans = Vec::new();
    let mut active_start: Option<usize> = None;
    for (offset, ink_count) in projection.iter().enumerate() {
        if *ink_count > 0 {
            if active_start.is_none() {
                active_start = Some(offset);
            }
            continue;
        }

        if let Some(start) = active_start.take() {
            let end = offset.saturating_sub(1);
            spans.extend(split_span_by_valleys(
                &projection,
                min_x,
                start,
                end,
                band_height,
            ));
        }
    }

    if let Some(start) = active_start {
        spans.extend(split_span_by_valleys(
            &projection,
            min_x,
            start,
            projection.len().saturating_sub(1),
            band_height,
        ));
    }

    spans
}

fn split_span_by_valleys(
    projection: &[usize],
    min_x: usize,
    start_offset: usize,
    end_offset: usize,
    band_height: usize,
) -> Vec<(usize, usize)> {
    let span_width = end_offset.saturating_sub(start_offset) + 1;
    let estimated_glyphs = (span_width / band_height.max(1)).clamp(1, 6);
    if estimated_glyphs <= 1 || span_width < band_height.saturating_mul(2) {
        return vec![(min_x + start_offset, min_x + end_offset)];
    }

    let mut cuts = Vec::new();
    for split_index in 1..estimated_glyphs {
        let anchor = start_offset + ((span_width * split_index) / estimated_glyphs);
        let window_start = anchor.saturating_sub((band_height / 4).max(2));
        let window_end = (anchor + (band_height / 4).max(2)).min(end_offset);

        let mut best_offset = anchor;
        let mut best_value = usize::MAX;
        for offset in window_start..=window_end {
            let value = projection.get(offset).copied().unwrap_or(0);
            if value < best_value {
                best_value = value;
                best_offset = offset;
            }
        }

        if best_offset > start_offset && best_offset < end_offset {
            cuts.push(best_offset);
        }
    }

    if cuts.is_empty() {
        return vec![(min_x + start_offset, min_x + end_offset)];
    }

    let mut spans = Vec::new();
    let mut current_start = start_offset;
    for cut in cuts {
        if cut <= current_start {
            continue;
        }
        spans.push((min_x + current_start, min_x + cut));
        current_start = cut.saturating_add(1);
    }
    if current_start <= end_offset {
        spans.push((min_x + current_start, min_x + end_offset));
    }

    spans
}

fn collect_ink_points(
    image: &GrayImage,
    start_x: usize,
    end_x: usize,
    start_y: usize,
    end_y: usize,
    ink_threshold: u8,
) -> Option<Vec<(usize, usize)>> {
    let mut points = Vec::new();
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let pixel = image.get_pixel(u32::try_from(x).ok()?, u32::try_from(y).ok()?);
            if is_ink(*pixel, ink_threshold) {
                points.push((x, y));
            }
        }
    }

    if points.len() < 24 {
        return None;
    }

    Some(points)
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

fn trace_component_outline(points: &[(usize, usize)]) -> Option<Vec<(usize, usize)>> {
    type Vertex = (usize, usize);
    type Edge = (Vertex, Vertex);

    let occupied = points.iter().copied().collect::<HashSet<_>>();
    if occupied.is_empty() {
        return None;
    }

    let mut boundary_edges = HashSet::<Edge>::new();
    for &(x, y) in &occupied {
        let edges = [
            ((x, y), (x + 1, y)),
            ((x + 1, y), (x + 1, y + 1)),
            ((x, y + 1), (x + 1, y + 1)),
            ((x, y), (x, y + 1)),
        ];

        for edge in edges {
            let canonical = canonical_edge(edge);
            if !boundary_edges.insert(canonical) {
                boundary_edges.remove(&canonical);
            }
        }
    }

    if boundary_edges.is_empty() {
        return None;
    }

    let mut adjacency = HashMap::<Vertex, Vec<Vertex>>::new();
    for &(start, end) in &boundary_edges {
        adjacency.entry(start).or_default().push(end);
        adjacency.entry(end).or_default().push(start);
    }

    let start = adjacency.keys().min().copied()?;
    let mut outline = Vec::new();
    let mut current = start;
    let mut previous: Option<Vertex> = None;
    loop {
        outline.push(current);
        let neighbors = adjacency.get(&current)?;
        let next = if neighbors.len() == 1 {
            neighbors[0]
        } else {
            choose_next_vertex(current, previous, neighbors)?
        };

        previous = Some(current);
        current = next;

        if current == start {
            break;
        }

        if outline.len() > boundary_edges.len().saturating_mul(2) {
            return None;
        }
    }

    let simplified = remove_collinear_points(&outline);
    if simplified.len() < 4 {
        return None;
    }

    Some(resample_outline(&simplified, 160))
}

fn canonical_edge(edge: ((usize, usize), (usize, usize))) -> ((usize, usize), (usize, usize)) {
    if edge.0 <= edge.1 {
        edge
    } else {
        (edge.1, edge.0)
    }
}

fn choose_next_vertex(
    current: (usize, usize),
    previous: Option<(usize, usize)>,
    neighbors: &[(usize, usize)],
) -> Option<(usize, usize)> {
    if previous.is_none() {
        return neighbors.iter().copied().min_by_key(|neighbor| {
            let dx = neighbor.0.abs_diff(current.0);
            let dy = neighbor.1.abs_diff(current.1);
            (dy, dx, *neighbor)
        });
    }

    let previous = previous?;
    neighbors
        .iter()
        .copied()
        .filter(|neighbor| *neighbor != previous)
        .min_by_key(|neighbor| {
            let dx = neighbor.0.abs_diff(current.0);
            let dy = neighbor.1.abs_diff(current.1);
            (dy, dx, *neighbor)
        })
        .or(Some(previous))
}

fn remove_collinear_points(points: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut simplified = Vec::with_capacity(points.len());
    for index in 0..points.len() {
        let previous = points[(index + points.len() - 1) % points.len()];
        let current = points[index];
        let next = points[(index + 1) % points.len()];

        let delta_prev_x =
            i64::try_from(current.0).unwrap_or(0) - i64::try_from(previous.0).unwrap_or(0);
        let delta_prev_y =
            i64::try_from(current.1).unwrap_or(0) - i64::try_from(previous.1).unwrap_or(0);
        let delta_next_x =
            i64::try_from(next.0).unwrap_or(0) - i64::try_from(current.0).unwrap_or(0);
        let delta_next_y =
            i64::try_from(next.1).unwrap_or(0) - i64::try_from(current.1).unwrap_or(0);

        if delta_prev_x * delta_next_y != delta_prev_y * delta_next_x {
            simplified.push(current);
        }
    }

    simplified
}

fn resample_outline(points: &[(usize, usize)], max_points: usize) -> Vec<(usize, usize)> {
    if points.len() <= max_points {
        return points.to_vec();
    }

    let stride = points.len().div_ceil(max_points);
    points.iter().step_by(stride).copied().collect()
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
