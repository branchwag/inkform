use ttf_parser::{Face, OutlineBuilder};

const BASE_FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/HomemadeApple-Regular.ttf");

pub fn base_font_glyph(character: char) -> Option<Vec<Vec<(i16, i16)>>> {
    let face = Face::parse(BASE_FONT_BYTES, 0).ok()?;
    let glyph_id = face.glyph_index(character)?;
    let units_per_em = f32::from(face.units_per_em());
    let mut builder = ContourCollector::default();
    face.outline_glyph(glyph_id, &mut builder)?;

    let contours = builder.finish();
    normalize_contours(&contours, units_per_em)
}

#[derive(Default)]
struct ContourCollector {
    contours: Vec<Vec<(f32, f32)>>,
    current: Vec<(f32, f32)>,
    pen: Option<(f32, f32)>,
}

impl ContourCollector {
    fn finish(mut self) -> Vec<Vec<(f32, f32)>> {
        if !self.current.is_empty() {
            self.contours.push(self.current);
        }
        self.contours
    }

    fn push_point(&mut self, point: (f32, f32)) {
        self.current.push(point);
        self.pen = Some(point);
    }

    fn flush_contour(&mut self) {
        if !self.current.is_empty() {
            let next = std::mem::take(&mut self.current);
            self.contours.push(next);
        }
    }
}

impl OutlineBuilder for ContourCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush_contour();
        self.push_point((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push_point((x, y));
    }

    #[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let start = self.pen.unwrap_or((x1, y1));
        for step in 1..=8 {
            let t = (step as f32) / 8.0;
            let one_minus_t = 1.0 - t;
            let point = (
                one_minus_t.powi(2) * start.0 + 2.0 * one_minus_t * t * x1 + t.powi(2) * x,
                one_minus_t.powi(2) * start.1 + 2.0 * one_minus_t * t * y1 + t.powi(2) * y,
            );
            self.push_point(point);
        }
    }

    #[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let start = self.pen.unwrap_or((x1, y1));
        for step in 1..=10 {
            let t = (step as f32) / 10.0;
            let one_minus_t = 1.0 - t;
            let point = (
                one_minus_t.powi(3) * start.0
                    + 3.0 * one_minus_t.powi(2) * t * x1
                    + 3.0 * one_minus_t * t.powi(2) * x2
                    + t.powi(3) * x,
                one_minus_t.powi(3) * start.1
                    + 3.0 * one_minus_t.powi(2) * t * y1
                    + 3.0 * one_minus_t * t.powi(2) * y2
                    + t.powi(3) * y,
            );
            self.push_point(point);
        }
    }

    fn close(&mut self) {
        self.flush_contour();
        self.pen = None;
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::suboptimal_flops
)]
fn normalize_contours(
    contours: &[Vec<(f32, f32)>],
    units_per_em: f32,
) -> Option<Vec<Vec<(i16, i16)>>> {
    if contours.is_empty() || units_per_em <= 0.0 {
        return None;
    }

    let min_x = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(x, _)| *x))
        .reduce(f32::min)?;
    let max_x = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(x, _)| *x))
        .reduce(f32::max)?;
    let min_y = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(_, y)| *y))
        .reduce(f32::min)?;
    let max_y = contours
        .iter()
        .flat_map(|contour| contour.iter().map(|(_, y)| *y))
        .reduce(f32::max)?;

    if max_x <= min_x || max_y <= min_y {
        return None;
    }

    let width = max_x - min_x;
    let scale = (700.0 / units_per_em).min(520.0 / width.max(1.0));
    let scaled_width = width * scale;
    let horizontal_padding = ((520.0 - scaled_width) * 0.5).max(0.0);

    let normalized = contours
        .iter()
        .filter(|contour| !contour.is_empty())
        .map(|contour| {
            contour
                .iter()
                .map(|(x, y)| {
                    (
                        round_to_i16((x - min_x).mul_add(scale, horizontal_padding + 50.0)),
                        round_to_i16((y - min_y) * scale),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if normalized.is_empty() {
        return None;
    }

    Some(normalized)
}

#[allow(clippy::cast_possible_truncation)]
fn round_to_i16(value: f32) -> i16 {
    i16::try_from(value.round() as i32).unwrap_or_else(|_| {
        if value.is_sign_negative() {
            i16::MIN
        } else {
            i16::MAX
        }
    })
}

#[cfg(test)]
mod tests {
    use super::base_font_glyph;

    #[test]
    fn extracts_base_font_contours() {
        let contours = base_font_glyph('a');
        assert!(contours.is_some(), "expected base font contour for 'a'");
        let contours = contours.unwrap_or_default();
        assert!(!contours.is_empty(), "expected at least one contour");
        assert!(contours.iter().all(|contour| contour.len() >= 3));
    }
}
