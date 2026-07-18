use crate::domain::{GlyphCandidate, SampleImage, ScriptPack};

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
struct TableRecord {
    tag: [u8; 4],
    data: Vec<u8>,
}

pub fn build_ttf(
    family_name: &str,
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
) -> Vec<u8> {
    let base_seed = hash_bytes(&sample_image.bytes);
    let glyph_definitions = build_glyph_definitions(base_seed, script_pack, glyphs);

    let metrics = FontMetrics::from_glyphs(&glyph_definitions);
    let head = build_head_table(metrics);
    let hhea = build_hhea_table(metrics, glyph_definitions.len());
    let maxp = build_maxp_table(&glyph_definitions);
    let hmtx = build_hmtx_table(&glyph_definitions);
    let (glyf, loca) = build_glyf_and_loca_tables(&glyph_definitions);
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

fn build_glyph_definitions(
    base_seed: u64,
    script_pack: &ScriptPack,
    glyphs: &[GlyphCandidate],
) -> Vec<GlyphDefinition> {
    let mut definitions = Vec::with_capacity(glyphs.len() + 1);
    definitions.push(notdef_glyph());

    for (glyph_index, character) in script_pack.glyphs.iter().enumerate() {
        let candidate = glyphs
            .get(glyph_index)
            .map_or(*character, |glyph| glyph.character);
        definitions.push(build_algorithmic_glyph(
            candidate,
            mix_seed(base_seed, *character),
        ));
    }

    definitions
}

fn notdef_glyph() -> GlyphDefinition {
    let points = [(80_i16, 0_i16), (80, 700), (520, 700), (520, 0)];
    build_simple_polygon_glyph(600, 40, &points)
}

fn build_algorithmic_glyph(character: char, seed: u64) -> GlyphDefinition {
    if character == ' ' {
        return GlyphDefinition {
            advance_width: 320,
            left_side_bearing: 0,
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            data: empty_glyph_data(),
        };
    }

    let advance_width = match character {
        'A'..='Z' => 720,
        'a'..='z' | 'Ä' | 'Ö' | 'Ü' | 'ä' | 'ö' | 'ü' | 'ß' => 620,
        '0'..='9' => 600,
        _ => 460,
    };

    let width_span = i16::try_from(advance_width).unwrap_or(720) - 120;
    let x0 = 40 + seeded_range(seed, 0, 40);
    let x1 = 24 + seeded_range(seed, 1, 60);
    let x2 = 100 + seeded_range(seed, 2, 80);
    let x3 = (width_span / 2) + seeded_range(seed, 3, 100);
    let x4 = (width_span - 100) + seeded_range(seed, 4, 60);
    let x5 = (width_span - 40) + seeded_range(seed, 5, 30);

    let points = [
        (x0, 0),
        (x1, 120 + seeded_range(seed, 6, 120)),
        (x2, 560 + seeded_range(seed, 7, 90)),
        (x3, 690 - seeded_range(seed, 8, 60)),
        (x4, 520 + seeded_range(seed, 9, 100)),
        (x5, 0),
    ];

    build_simple_polygon_glyph(advance_width, 32, &points)
}

fn build_simple_polygon_glyph(
    advance_width: u16,
    left_side_bearing: i16,
    points: &[(i16, i16)],
) -> GlyphDefinition {
    let (x_min, y_min, x_max, y_max) = bounds(points);

    let mut data = Vec::new();
    push_i16(&mut data, 1);
    push_i16(&mut data, x_min);
    push_i16(&mut data, y_min);
    push_i16(&mut data, x_max);
    push_i16(&mut data, y_max);
    let end_point = u16::try_from(points.len().saturating_sub(1)).unwrap_or(0);
    push_u16(&mut data, end_point);
    push_u16(&mut data, 0);

    data.extend(std::iter::repeat_n(0x01, points.len()));

    let mut previous_x = 0_i16;
    for (x, _) in points {
        push_i16(&mut data, *x - previous_x);
        previous_x = *x;
    }

    let mut previous_y = 0_i16;
    for (_, y) in points {
        push_i16(&mut data, *y - previous_y);
        previous_y = *y;
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

fn bounds(points: &[(i16, i16)]) -> (i16, i16, i16, i16) {
    let x_min = points.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let y_min = points.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let x_max = points.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let y_max = points.iter().map(|(_, y)| *y).max().unwrap_or(0);
    (x_min, y_min, x_max, y_max)
}

fn build_head_table(metrics: FontMetrics) -> Vec<u8> {
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
    push_i16(&mut table, 0);
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
    let number_of_h_metrics = u16::try_from(glyph_count).unwrap_or(u16::MAX);
    push_u16(&mut table, number_of_h_metrics);
    table
}

fn build_maxp_table(glyphs: &[GlyphDefinition]) -> Vec<u8> {
    let max_points = glyphs.iter().map(max_point_count).max().unwrap_or(0);
    let max_contours = glyphs
        .iter()
        .map(|glyph| u16::from(glyph.data.len() > 10))
        .max()
        .unwrap_or(0);

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

fn build_hmtx_table(glyphs: &[GlyphDefinition]) -> Vec<u8> {
    let mut table = Vec::new();
    for glyph in glyphs {
        push_u16(&mut table, glyph.advance_width);
        push_i16(&mut table, glyph.left_side_bearing);
    }
    table
}

fn build_glyf_and_loca_tables(glyphs: &[GlyphDefinition]) -> (Vec<u8>, Vec<u8>) {
    let mut glyf = Vec::new();
    let mut offsets = Vec::with_capacity(glyphs.len() + 1);

    for glyph in glyphs {
        offsets.push(u16::try_from(glyf.len() / 2).unwrap_or(u16::MAX));
        glyf.extend_from_slice(&glyph.data);
        if glyf.len() % 2 != 0 {
            glyf.push(0);
        }
    }
    offsets.push(u16::try_from(glyf.len() / 2).unwrap_or(u16::MAX));

    let mut loca = Vec::new();
    for offset in offsets {
        push_u16(&mut loca, offset);
    }

    (glyf, loca)
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

const fn seeded_range(seed: u64, lane: u32, modulo: i16) -> i16 {
    let shift = lane % 8 * 8;
    let narrowed = ((seed >> shift) & 0xFF) as i16;
    if modulo == 0 { 0 } else { narrowed % modulo }
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
