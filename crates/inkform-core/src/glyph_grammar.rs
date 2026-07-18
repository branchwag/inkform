#[derive(Debug, Clone, Copy)]
pub struct GlyphStyle {
    pub slant: f32,
    pub width_scale: f32,
    pub stroke_width: f32,
    pub waviness: f32,
    pub baseline_lift: f32,
    pub body_height: f32,
    pub ascender_height: f32,
    pub descender_depth: f32,
    pub cursive_score: f32,
}

#[derive(Debug, Clone, Copy)]
struct StrokeRecipe {
    points: &'static [(f32, f32)],
    closed: bool,
    thickness_scale: f32,
}

#[derive(Debug, Clone, Copy)]
enum Accent {
    Acute,
    Grave,
    Circumflex,
    Tilde,
    Diaeresis,
    Ring,
    Cedilla,
    Slash,
}

pub fn build_glyph_from_grammar(
    character: char,
    style: GlyphStyle,
    seed: u64,
) -> Option<Vec<Vec<(i16, i16)>>> {
    let (base_character, accents) = decompose_character(character);
    let recipes = if style.cursive_score >= 0.68 {
        looped_recipes_for_character(base_character)
            .or_else(|| recipes_for_character(base_character))?
    } else {
        recipes_for_character(base_character)?
    };
    let mut contours = Vec::new();

    for recipe in recipes {
        let thickness = (style.stroke_width * recipe.thickness_scale).max(18.0);
        let mut rendered = render_recipe(*recipe, style, thickness, seed, 0.0, 0.0);
        contours.append(&mut rendered);
    }

    for accent in accents {
        let mut rendered = render_accent(*accent, style, base_character, seed);
        contours.append(&mut rendered);
    }

    if contours.is_empty() {
        return None;
    }

    Some(contours)
}

#[must_use]
pub fn build_cursive_join_stroke(
    style: GlyphStyle,
    advance_width: u16,
    terminal: (i16, i16),
) -> Option<Vec<(i16, i16)>> {
    if style.cursive_score < 0.68 || advance_width < 180 {
        return None;
    }

    let advance = f32::from(advance_width);
    let start_x = f32::from(terminal.0);
    let start_y = f32::from(terminal.1);
    let baseline = style.baseline_lift + 26.0;
    let points = [
        (start_x, start_y),
        ((start_x + advance) * 0.5, baseline + 16.0),
        (advance + 42.0, baseline + 12.0),
    ];
    // Keep the connector visually continuous with the handwritten stroke. The
    // trajectory stays short and attached, so this does not become an underline.
    let thickness = (style.stroke_width * 0.68).max(16.0);
    render_open_stroke(&points, thickness, style.cursive_score)
        .into_iter()
        .next()
}

const fn decompose_character(character: char) -> (char, &'static [Accent]) {
    match character {
        'Ä' => ('A', &[Accent::Diaeresis]),
        'Ö' => ('O', &[Accent::Diaeresis]),
        'Ü' => ('U', &[Accent::Diaeresis]),
        'ä' => ('a', &[Accent::Diaeresis]),
        'ö' => ('o', &[Accent::Diaeresis]),
        'ü' => ('u', &[Accent::Diaeresis]),
        'À' => ('A', &[Accent::Grave]),
        'Á' => ('A', &[Accent::Acute]),
        'Â' => ('A', &[Accent::Circumflex]),
        'Ã' => ('A', &[Accent::Tilde]),
        'Å' => ('A', &[Accent::Ring]),
        'Æ' => ('A', &[]),
        'Ç' => ('C', &[Accent::Cedilla]),
        'È' => ('E', &[Accent::Grave]),
        'É' => ('E', &[Accent::Acute]),
        'Ê' => ('E', &[Accent::Circumflex]),
        'Ë' => ('E', &[Accent::Diaeresis]),
        'Ì' => ('I', &[Accent::Grave]),
        'Í' => ('I', &[Accent::Acute]),
        'Î' => ('I', &[Accent::Circumflex]),
        'Ï' => ('I', &[Accent::Diaeresis]),
        'Ñ' => ('N', &[Accent::Tilde]),
        'Ò' => ('O', &[Accent::Grave]),
        'Ó' => ('O', &[Accent::Acute]),
        'Ô' => ('O', &[Accent::Circumflex]),
        'Õ' => ('O', &[Accent::Tilde]),
        'Ø' => ('O', &[Accent::Slash]),
        'Ù' => ('U', &[Accent::Grave]),
        'Ú' => ('U', &[Accent::Acute]),
        'Û' => ('U', &[Accent::Circumflex]),
        'Ý' => ('Y', &[Accent::Acute]),
        'à' => ('a', &[Accent::Grave]),
        'á' => ('a', &[Accent::Acute]),
        'â' => ('a', &[Accent::Circumflex]),
        'ã' => ('a', &[Accent::Tilde]),
        'å' => ('a', &[Accent::Ring]),
        'æ' => ('a', &[]),
        'ç' => ('c', &[Accent::Cedilla]),
        'è' => ('e', &[Accent::Grave]),
        'é' => ('e', &[Accent::Acute]),
        'ê' => ('e', &[Accent::Circumflex]),
        'ë' => ('e', &[Accent::Diaeresis]),
        'ì' => ('i', &[Accent::Grave]),
        'í' => ('i', &[Accent::Acute]),
        'î' => ('i', &[Accent::Circumflex]),
        'ï' => ('i', &[Accent::Diaeresis]),
        'ñ' => ('n', &[Accent::Tilde]),
        'ò' => ('o', &[Accent::Grave]),
        'ó' => ('o', &[Accent::Acute]),
        'ô' => ('o', &[Accent::Circumflex]),
        'õ' => ('o', &[Accent::Tilde]),
        'ø' => ('o', &[Accent::Slash]),
        'ù' => ('u', &[Accent::Grave]),
        'ú' => ('u', &[Accent::Acute]),
        'û' => ('u', &[Accent::Circumflex]),
        'ý' => ('y', &[Accent::Acute]),
        'ÿ' => ('y', &[Accent::Diaeresis]),
        _ => (character, &[]),
    }
}

#[allow(clippy::too_many_lines)]
const fn recipes_for_character(character: char) -> Option<&'static [StrokeRecipe]> {
    match character {
        'A' => Some(&[
            StrokeRecipe {
                points: &[(110.0, 0.0), (180.0, 350.0), (290.0, 760.0)],
                closed: false,
                thickness_scale: 0.95,
            },
            StrokeRecipe {
                points: &[(290.0, 760.0), (390.0, 360.0), (470.0, 0.0)],
                closed: false,
                thickness_scale: 0.95,
            },
            StrokeRecipe {
                points: &[(180.0, 320.0), (300.0, 340.0), (390.0, 320.0)],
                closed: false,
                thickness_scale: 0.72,
            },
        ]),
        'B' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 410.0), (120.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[
                    (150.0, 740.0),
                    (350.0, 720.0),
                    (390.0, 560.0),
                    (180.0, 420.0),
                ],
                closed: false,
                thickness_scale: 0.9,
            },
            StrokeRecipe {
                points: &[
                    (170.0, 410.0),
                    (360.0, 390.0),
                    (410.0, 180.0),
                    (150.0, 30.0),
                ],
                closed: false,
                thickness_scale: 0.9,
            },
        ]),
        'C' => Some(&[StrokeRecipe {
            points: &[
                (430.0, 650.0),
                (300.0, 750.0),
                (150.0, 620.0),
                (130.0, 230.0),
                (300.0, 60.0),
                (430.0, 140.0),
            ],
            closed: false,
            thickness_scale: 1.0,
        }]),
        'D' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (125.0, 400.0), (120.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[
                    (160.0, 740.0),
                    (390.0, 660.0),
                    (420.0, 120.0),
                    (160.0, 20.0),
                ],
                closed: false,
                thickness_scale: 0.92,
            },
        ]),
        'E' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 400.0), (120.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[(140.0, 740.0), (420.0, 720.0)],
                closed: false,
                thickness_scale: 0.78,
            },
            StrokeRecipe {
                points: &[(140.0, 390.0), (350.0, 400.0)],
                closed: false,
                thickness_scale: 0.72,
            },
            StrokeRecipe {
                points: &[(140.0, 30.0), (430.0, 40.0)],
                closed: false,
                thickness_scale: 0.78,
            },
        ]),
        'F' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 400.0), (120.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[(140.0, 740.0), (420.0, 720.0)],
                closed: false,
                thickness_scale: 0.78,
            },
            StrokeRecipe {
                points: &[(140.0, 410.0), (340.0, 420.0)],
                closed: false,
                thickness_scale: 0.72,
            },
        ]),
        'G' => Some(&[StrokeRecipe {
            points: &[
                (430.0, 650.0),
                (300.0, 750.0),
                (150.0, 620.0),
                (130.0, 230.0),
                (300.0, 60.0),
                (460.0, 160.0),
                (420.0, 320.0),
                (300.0, 330.0),
            ],
            closed: false,
            thickness_scale: 1.0,
        }]),
        'H' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[(430.0, 0.0), (420.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[(150.0, 380.0), (280.0, 390.0), (410.0, 380.0)],
                closed: false,
                thickness_scale: 0.72,
            },
        ]),
        'I' => Some(&[StrokeRecipe {
            points: &[(250.0, 0.0), (260.0, 760.0)],
            closed: false,
            thickness_scale: 1.0,
        }]),
        'J' => Some(&[StrokeRecipe {
            points: &[
                (420.0, 740.0),
                (300.0, 750.0),
                (290.0, 120.0),
                (200.0, 20.0),
                (90.0, 80.0),
            ],
            closed: false,
            thickness_scale: 0.96,
        }]),
        'K' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[(410.0, 740.0), (250.0, 420.0), (120.0, 320.0)],
                closed: false,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[(220.0, 390.0), (340.0, 210.0), (430.0, 20.0)],
                closed: false,
                thickness_scale: 0.82,
            },
        ]),
        'L' => Some(&[StrokeRecipe {
            points: &[(120.0, 760.0), (130.0, 40.0), (430.0, 30.0)],
            closed: false,
            thickness_scale: 1.0,
        }]),
        'M' => Some(&[StrokeRecipe {
            points: &[
                (100.0, 0.0),
                (120.0, 760.0),
                (280.0, 430.0),
                (430.0, 760.0),
                (460.0, 0.0),
            ],
            closed: false,
            thickness_scale: 0.98,
        }]),
        'N' => Some(&[StrokeRecipe {
            points: &[(110.0, 0.0), (120.0, 760.0), (420.0, 0.0), (430.0, 760.0)],
            closed: false,
            thickness_scale: 0.98,
        }]),
        'O' => Some(&[StrokeRecipe {
            points: &[
                (300.0, 760.0),
                (160.0, 620.0),
                (130.0, 250.0),
                (290.0, 40.0),
                (430.0, 200.0),
                (420.0, 590.0),
                (300.0, 760.0),
            ],
            closed: true,
            thickness_scale: 0.92,
        }]),
        'P' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[
                    (150.0, 720.0),
                    (360.0, 690.0),
                    (390.0, 500.0),
                    (150.0, 400.0),
                ],
                closed: false,
                thickness_scale: 0.88,
            },
        ]),
        'Q' => Some(&[
            StrokeRecipe {
                points: &[
                    (300.0, 760.0),
                    (160.0, 620.0),
                    (130.0, 250.0),
                    (290.0, 40.0),
                    (430.0, 200.0),
                    (420.0, 590.0),
                    (300.0, 760.0),
                ],
                closed: true,
                thickness_scale: 0.92,
            },
            StrokeRecipe {
                points: &[(320.0, 180.0), (430.0, -30.0)],
                closed: false,
                thickness_scale: 0.6,
            },
        ]),
        'R' => Some(&[
            StrokeRecipe {
                points: &[(120.0, 0.0), (130.0, 760.0)],
                closed: false,
                thickness_scale: 1.0,
            },
            StrokeRecipe {
                points: &[
                    (150.0, 720.0),
                    (360.0, 690.0),
                    (390.0, 500.0),
                    (150.0, 400.0),
                ],
                closed: false,
                thickness_scale: 0.88,
            },
            StrokeRecipe {
                points: &[(200.0, 390.0), (320.0, 210.0), (430.0, 10.0)],
                closed: false,
                thickness_scale: 0.78,
            },
        ]),
        'S' => Some(&[StrokeRecipe {
            points: &[
                (420.0, 650.0),
                (280.0, 740.0),
                (140.0, 620.0),
                (320.0, 410.0),
                (420.0, 250.0),
                (270.0, 40.0),
                (110.0, 130.0),
            ],
            closed: false,
            thickness_scale: 0.96,
        }]),
        'T' => Some(&[
            StrokeRecipe {
                points: &[(90.0, 740.0), (450.0, 740.0)],
                closed: false,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[(270.0, 740.0), (260.0, 0.0)],
                closed: false,
                thickness_scale: 0.96,
            },
        ]),
        'U' => Some(&[StrokeRecipe {
            points: &[
                (110.0, 740.0),
                (130.0, 180.0),
                (270.0, 40.0),
                (410.0, 200.0),
                (430.0, 740.0),
            ],
            closed: false,
            thickness_scale: 0.96,
        }]),
        'V' => Some(&[StrokeRecipe {
            points: &[(110.0, 740.0), (270.0, 20.0), (430.0, 740.0)],
            closed: false,
            thickness_scale: 0.96,
        }]),
        'W' => Some(&[StrokeRecipe {
            points: &[
                (80.0, 740.0),
                (170.0, 30.0),
                (270.0, 430.0),
                (360.0, 20.0),
                (450.0, 740.0),
            ],
            closed: false,
            thickness_scale: 0.92,
        }]),
        'X' => Some(&[
            StrokeRecipe {
                points: &[(110.0, 740.0), (430.0, 0.0)],
                closed: false,
                thickness_scale: 0.84,
            },
            StrokeRecipe {
                points: &[(430.0, 740.0), (110.0, 0.0)],
                closed: false,
                thickness_scale: 0.84,
            },
        ]),
        'Y' => Some(&[
            StrokeRecipe {
                points: &[(110.0, 740.0), (260.0, 420.0), (420.0, 740.0)],
                closed: false,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[(260.0, 420.0), (250.0, 0.0)],
                closed: false,
                thickness_scale: 0.92,
            },
        ]),
        'Z' => Some(&[StrokeRecipe {
            points: &[(110.0, 730.0), (440.0, 730.0), (130.0, 20.0), (450.0, 20.0)],
            closed: false,
            thickness_scale: 0.82,
        }]),
        'a' => Some(&[
            StrokeRecipe {
                points: &[
                    (275.0, 410.0),
                    (175.0, 335.0),
                    (165.0, 175.0),
                    (270.0, 85.0),
                    (375.0, 165.0),
                    (370.0, 330.0),
                ],
                closed: true,
                thickness_scale: 0.76,
            },
            StrokeRecipe {
                points: &[(375.0, 335.0), (400.0, 45.0), (450.0, 30.0)],
                closed: false,
                thickness_scale: 0.72,
            },
        ]),
        'b' => Some(&[
            StrokeRecipe {
                points: &[(130.0, 0.0), (135.0, 760.0)],
                closed: false,
                thickness_scale: 0.92,
            },
            StrokeRecipe {
                points: &[
                    (170.0, 420.0),
                    (330.0, 420.0),
                    (390.0, 260.0),
                    (300.0, 80.0),
                    (160.0, 170.0),
                ],
                closed: false,
                thickness_scale: 0.8,
            },
        ]),
        'c' => Some(&[StrokeRecipe {
            points: &[
                (380.0, 380.0),
                (270.0, 450.0),
                (150.0, 330.0),
                (150.0, 150.0),
                (300.0, 70.0),
                (390.0, 130.0),
            ],
            closed: false,
            thickness_scale: 0.82,
        }]),
        'd' => Some(&[
            StrokeRecipe {
                points: &[(390.0, 0.0), (390.0, 760.0)],
                closed: false,
                thickness_scale: 0.92,
            },
            StrokeRecipe {
                points: &[
                    (340.0, 410.0),
                    (180.0, 360.0),
                    (150.0, 170.0),
                    (300.0, 70.0),
                    (390.0, 180.0),
                    (330.0, 360.0),
                ],
                closed: false,
                thickness_scale: 0.8,
            },
        ]),
        'e' => Some(&[
            // The sample's `e` is a compact, low loop instead of a full-height
            // circular bowl.
            StrokeRecipe {
                points: &[
                    (350.0, 175.0),
                    (260.0, 105.0),
                    (180.0, 135.0),
                    (150.0, 205.0),
                    (185.0, 275.0),
                    (280.0, 290.0),
                    (355.0, 230.0),
                    (365.0, 190.0),
                    (405.0, 230.0),
                ],
                closed: false,
                thickness_scale: 0.76,
            },
            StrokeRecipe {
                points: &[(150.0, 205.0), (300.0, 205.0)],
                closed: false,
                thickness_scale: 0.66,
            },
        ]),
        'f' => Some(&[
            StrokeRecipe {
                points: &[
                    (350.0, 700.0),
                    (250.0, 760.0),
                    (170.0, 620.0),
                    (220.0, 440.0),
                    (250.0, 220.0),
                    (220.0, -180.0),
                ],
                closed: false,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[(110.0, 420.0), (330.0, 420.0)],
                closed: false,
                thickness_scale: 0.58,
            },
        ]),
        'g' => Some(&[StrokeRecipe {
            points: &[
                (330.0, 420.0),
                (180.0, 350.0),
                (150.0, 170.0),
                (290.0, 80.0),
                (390.0, 180.0),
                (340.0, 360.0),
                (310.0, 80.0),
                (280.0, -140.0),
                (150.0, -180.0),
            ],
            closed: false,
            thickness_scale: 0.8,
        }]),
        'h' => Some(&[
            StrokeRecipe {
                points: &[(130.0, 0.0), (135.0, 760.0)],
                closed: false,
                thickness_scale: 0.92,
            },
            StrokeRecipe {
                points: &[
                    (150.0, 280.0),
                    (230.0, 410.0),
                    (370.0, 330.0),
                    (390.0, 30.0),
                ],
                closed: false,
                thickness_scale: 0.76,
            },
        ]),
        'i' => Some(&[
            StrokeRecipe {
                points: &[(230.0, 20.0), (240.0, 420.0)],
                closed: false,
                thickness_scale: 0.74,
            },
            StrokeRecipe {
                points: &[
                    (230.0, 610.0),
                    (250.0, 650.0),
                    (270.0, 610.0),
                    (250.0, 570.0),
                    (230.0, 610.0),
                ],
                closed: true,
                thickness_scale: 0.42,
            },
        ]),
        'j' => Some(&[
            StrokeRecipe {
                points: &[
                    (320.0, 420.0),
                    (300.0, 20.0),
                    (250.0, -170.0),
                    (130.0, -150.0),
                ],
                closed: false,
                thickness_scale: 0.74,
            },
            StrokeRecipe {
                points: &[
                    (300.0, 610.0),
                    (320.0, 650.0),
                    (340.0, 610.0),
                    (320.0, 570.0),
                    (300.0, 610.0),
                ],
                closed: true,
                thickness_scale: 0.42,
            },
        ]),
        'k' => Some(&[
            StrokeRecipe {
                points: &[(130.0, 0.0), (135.0, 760.0)],
                closed: false,
                thickness_scale: 0.92,
            },
            StrokeRecipe {
                points: &[(360.0, 430.0), (220.0, 260.0), (370.0, 20.0)],
                closed: false,
                thickness_scale: 0.74,
            },
        ]),
        'l' => Some(&[StrokeRecipe {
            points: &[(250.0, 0.0), (260.0, 760.0)],
            closed: false,
            thickness_scale: 0.9,
        }]),
        'm' => Some(&[StrokeRecipe {
            points: &[
                (120.0, 20.0),
                (130.0, 420.0),
                (220.0, 310.0),
                (290.0, 420.0),
                (360.0, 310.0),
                (430.0, 420.0),
                (450.0, 20.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        'n' => Some(&[
            StrokeRecipe {
                points: &[(140.0, 20.0), (145.0, 420.0)],
                closed: false,
                thickness_scale: 0.76,
            },
            StrokeRecipe {
                points: &[
                    (150.0, 420.0),
                    (250.0, 400.0),
                    (365.0, 295.0),
                    (375.0, 20.0),
                ],
                closed: false,
                thickness_scale: 0.76,
            },
        ]),
        'o' => Some(&[StrokeRecipe {
            points: &[
                (280.0, 430.0),
                (170.0, 350.0),
                (150.0, 170.0),
                (280.0, 70.0),
                (400.0, 180.0),
                (390.0, 340.0),
                (280.0, 430.0),
            ],
            closed: true,
            thickness_scale: 0.76,
        }]),
        'p' => Some(&[
            StrokeRecipe {
                points: &[(140.0, -190.0), (145.0, 420.0)],
                closed: false,
                thickness_scale: 0.84,
            },
            StrokeRecipe {
                points: &[
                    (180.0, 390.0),
                    (340.0, 380.0),
                    (390.0, 230.0),
                    (310.0, 60.0),
                    (160.0, 120.0),
                ],
                closed: false,
                thickness_scale: 0.76,
            },
        ]),
        'q' => Some(&[
            StrokeRecipe {
                points: &[(380.0, -190.0), (375.0, 420.0)],
                closed: false,
                thickness_scale: 0.84,
            },
            StrokeRecipe {
                points: &[
                    (330.0, 390.0),
                    (170.0, 350.0),
                    (150.0, 170.0),
                    (290.0, 70.0),
                    (390.0, 170.0),
                    (340.0, 350.0),
                ],
                closed: false,
                thickness_scale: 0.76,
            },
        ]),
        'r' => Some(&[StrokeRecipe {
            points: &[
                (150.0, 20.0),
                (160.0, 420.0),
                (260.0, 320.0),
                (340.0, 360.0),
            ],
            closed: false,
            thickness_scale: 0.72,
        }]),
        's' => Some(&[StrokeRecipe {
            points: &[
                (380.0, 360.0),
                (260.0, 430.0),
                (150.0, 340.0),
                (320.0, 240.0),
                (390.0, 130.0),
                (260.0, 60.0),
                (140.0, 120.0),
            ],
            closed: false,
            thickness_scale: 0.72,
        }]),
        't' => Some(&[
            StrokeRecipe {
                points: &[(280.0, 620.0), (240.0, 420.0), (250.0, 20.0)],
                closed: false,
                thickness_scale: 0.76,
            },
            StrokeRecipe {
                points: &[(140.0, 360.0), (340.0, 370.0)],
                closed: false,
                thickness_scale: 0.56,
            },
        ]),
        'u' => Some(&[StrokeRecipe {
            points: &[
                (140.0, 410.0),
                (150.0, 120.0),
                (260.0, 60.0),
                (370.0, 140.0),
                (390.0, 420.0),
            ],
            closed: false,
            thickness_scale: 0.74,
        }]),
        'v' => Some(&[StrokeRecipe {
            points: &[(140.0, 420.0), (260.0, 30.0), (390.0, 420.0)],
            closed: false,
            thickness_scale: 0.74,
        }]),
        'w' => Some(&[StrokeRecipe {
            points: &[
                (100.0, 420.0),
                (180.0, 30.0),
                (270.0, 260.0),
                (350.0, 30.0),
                (430.0, 420.0),
            ],
            closed: false,
            thickness_scale: 0.72,
        }]),
        'x' => Some(&[
            StrokeRecipe {
                points: &[(150.0, 420.0), (380.0, 20.0)],
                closed: false,
                thickness_scale: 0.7,
            },
            StrokeRecipe {
                points: &[(380.0, 420.0), (160.0, 20.0)],
                closed: false,
                thickness_scale: 0.7,
            },
        ]),
        'y' => Some(&[StrokeRecipe {
            points: &[
                (150.0, 420.0),
                (270.0, 120.0),
                (350.0, 420.0),
                (300.0, 20.0),
                (240.0, -170.0),
                (120.0, -180.0),
            ],
            closed: false,
            thickness_scale: 0.74,
        }]),
        'z' => Some(&[StrokeRecipe {
            points: &[(150.0, 390.0), (390.0, 390.0), (150.0, 40.0), (400.0, 40.0)],
            closed: false,
            thickness_scale: 0.7,
        }]),
        '0' => Some(&[
            StrokeRecipe {
                points: &[
                    (290.0, 700.0),
                    (170.0, 560.0),
                    (150.0, 180.0),
                    (290.0, 30.0),
                    (420.0, 180.0),
                    (400.0, 560.0),
                    (290.0, 700.0),
                ],
                closed: true,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[(190.0, 120.0), (390.0, 590.0)],
                closed: false,
                thickness_scale: 0.44,
            },
        ]),
        '1' => Some(&[
            StrokeRecipe {
                points: &[(190.0, 540.0), (280.0, 700.0), (270.0, 20.0)],
                closed: false,
                thickness_scale: 0.78,
            },
            StrokeRecipe {
                points: &[(170.0, 30.0), (390.0, 30.0)],
                closed: false,
                thickness_scale: 0.58,
            },
        ]),
        '2' => Some(&[StrokeRecipe {
            points: &[
                (160.0, 540.0),
                (280.0, 700.0),
                (420.0, 560.0),
                (170.0, 20.0),
                (430.0, 30.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        '3' => Some(&[StrokeRecipe {
            points: &[
                (150.0, 620.0),
                (290.0, 710.0),
                (400.0, 560.0),
                (270.0, 390.0),
                (410.0, 210.0),
                (290.0, 20.0),
                (150.0, 90.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        '4' => Some(&[
            StrokeRecipe {
                points: &[(390.0, 20.0), (380.0, 700.0)],
                closed: false,
                thickness_scale: 0.74,
            },
            StrokeRecipe {
                points: &[(120.0, 250.0), (420.0, 260.0)],
                closed: false,
                thickness_scale: 0.58,
            },
            StrokeRecipe {
                points: &[(130.0, 250.0), (330.0, 700.0)],
                closed: false,
                thickness_scale: 0.68,
            },
        ]),
        '5' => Some(&[StrokeRecipe {
            points: &[
                (410.0, 700.0),
                (180.0, 700.0),
                (160.0, 390.0),
                (310.0, 430.0),
                (400.0, 260.0),
                (310.0, 40.0),
                (150.0, 100.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        '6' => Some(&[StrokeRecipe {
            points: &[
                (390.0, 620.0),
                (260.0, 710.0),
                (160.0, 460.0),
                (170.0, 160.0),
                (290.0, 50.0),
                (400.0, 180.0),
                (350.0, 350.0),
                (200.0, 300.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        '7' => Some(&[StrokeRecipe {
            points: &[(140.0, 700.0), (430.0, 700.0), (220.0, 20.0)],
            closed: false,
            thickness_scale: 0.74,
        }]),
        '8' => Some(&[
            StrokeRecipe {
                points: &[
                    (280.0, 700.0),
                    (180.0, 560.0),
                    (270.0, 410.0),
                    (380.0, 560.0),
                    (280.0, 700.0),
                ],
                closed: true,
                thickness_scale: 0.66,
            },
            StrokeRecipe {
                points: &[
                    (280.0, 400.0),
                    (170.0, 240.0),
                    (290.0, 40.0),
                    (400.0, 220.0),
                    (280.0, 400.0),
                ],
                closed: true,
                thickness_scale: 0.74,
            },
        ]),
        '9' => Some(&[StrokeRecipe {
            points: &[
                (370.0, 450.0),
                (300.0, 680.0),
                (170.0, 550.0),
                (190.0, 360.0),
                (350.0, 350.0),
                (380.0, 180.0),
                (270.0, 20.0),
                (150.0, 80.0),
            ],
            closed: false,
            thickness_scale: 0.76,
        }]),
        '.' => Some(&[StrokeRecipe {
            points: &[
                (250.0, 40.0),
                (280.0, 80.0),
                (250.0, 120.0),
                (220.0, 80.0),
                (250.0, 40.0),
            ],
            closed: true,
            thickness_scale: 0.46,
        }]),
        ',' => Some(&[StrokeRecipe {
            points: &[(270.0, 90.0), (230.0, -70.0)],
            closed: false,
            thickness_scale: 0.42,
        }]),
        ';' => Some(&[
            StrokeRecipe {
                points: &[
                    (250.0, 260.0),
                    (280.0, 300.0),
                    (250.0, 340.0),
                    (220.0, 300.0),
                    (250.0, 260.0),
                ],
                closed: true,
                thickness_scale: 0.38,
            },
            StrokeRecipe {
                points: &[(270.0, 80.0), (230.0, -70.0)],
                closed: false,
                thickness_scale: 0.4,
            },
        ]),
        ':' => Some(&[
            StrokeRecipe {
                points: &[
                    (250.0, 260.0),
                    (280.0, 300.0),
                    (250.0, 340.0),
                    (220.0, 300.0),
                    (250.0, 260.0),
                ],
                closed: true,
                thickness_scale: 0.38,
            },
            StrokeRecipe {
                points: &[
                    (250.0, 40.0),
                    (280.0, 80.0),
                    (250.0, 120.0),
                    (220.0, 80.0),
                    (250.0, 40.0),
                ],
                closed: true,
                thickness_scale: 0.38,
            },
        ]),
        '!' => Some(&[
            StrokeRecipe {
                points: &[(250.0, 120.0), (260.0, 600.0)],
                closed: false,
                thickness_scale: 0.5,
            },
            StrokeRecipe {
                points: &[
                    (250.0, 10.0),
                    (280.0, 50.0),
                    (250.0, 90.0),
                    (220.0, 50.0),
                    (250.0, 10.0),
                ],
                closed: true,
                thickness_scale: 0.34,
            },
        ]),
        '?' => Some(&[
            StrokeRecipe {
                points: &[
                    (150.0, 520.0),
                    (250.0, 680.0),
                    (390.0, 580.0),
                    (300.0, 430.0),
                    (250.0, 330.0),
                    (250.0, 220.0),
                ],
                closed: false,
                thickness_scale: 0.6,
            },
            StrokeRecipe {
                points: &[
                    (250.0, 20.0),
                    (280.0, 60.0),
                    (250.0, 100.0),
                    (220.0, 60.0),
                    (250.0, 20.0),
                ],
                closed: true,
                thickness_scale: 0.34,
            },
        ]),
        '-' | '_' => Some(&[StrokeRecipe {
            points: &[(140.0, 120.0), (390.0, 120.0)],
            closed: false,
            thickness_scale: 0.44,
        }]),
        '\'' => Some(&[StrokeRecipe {
            points: &[(200.0, 690.0), (180.0, 540.0)],
            closed: false,
            thickness_scale: 0.3,
        }]),
        '"' => Some(&[
            StrokeRecipe {
                points: &[(200.0, 690.0), (180.0, 540.0)],
                closed: false,
                thickness_scale: 0.3,
            },
            StrokeRecipe {
                points: &[(300.0, 700.0), (280.0, 550.0)],
                closed: false,
                thickness_scale: 0.3,
            },
        ]),
        '(' => Some(&[StrokeRecipe {
            points: &[
                (330.0, 760.0),
                (220.0, 560.0),
                (190.0, 360.0),
                (220.0, 160.0),
                (330.0, -40.0),
            ],
            closed: false,
            thickness_scale: 0.52,
        }]),
        ')' => Some(&[StrokeRecipe {
            points: &[
                (190.0, 760.0),
                (300.0, 560.0),
                (330.0, 360.0),
                (300.0, 160.0),
                (190.0, -40.0),
            ],
            closed: false,
            thickness_scale: 0.52,
        }]),
        '[' | '{' => Some(&[StrokeRecipe {
            points: &[
                (320.0, 760.0),
                (220.0, 760.0),
                (220.0, -40.0),
                (320.0, -40.0),
            ],
            closed: false,
            thickness_scale: 0.48,
        }]),
        ']' | '}' => Some(&[StrokeRecipe {
            points: &[
                (200.0, 760.0),
                (300.0, 760.0),
                (300.0, -40.0),
                (200.0, -40.0),
            ],
            closed: false,
            thickness_scale: 0.48,
        }]),
        '@' => Some(&[StrokeRecipe {
            points: &[
                (320.0, 650.0),
                (160.0, 560.0),
                (130.0, 220.0),
                (270.0, 70.0),
                (420.0, 190.0),
                (360.0, 420.0),
                (250.0, 340.0),
                (290.0, 220.0),
                (360.0, 260.0),
            ],
            closed: false,
            thickness_scale: 0.62,
        }]),
        '/' | '#' | '&' | '%' | '+' | '*' | '=' | '<' | '>' => Some(&[
            StrokeRecipe {
                points: &[(140.0, 40.0), (390.0, 720.0)],
                closed: false,
                thickness_scale: 0.46,
            },
            StrokeRecipe {
                points: &[(160.0, 320.0), (420.0, 320.0)],
                closed: false,
                thickness_scale: 0.36,
            },
        ]),
        ' ' => Some(&[]),
        'ß' => Some(&[StrokeRecipe {
            points: &[
                (160.0, 0.0),
                (170.0, 730.0),
                (310.0, 730.0),
                (350.0, 590.0),
                (250.0, 420.0),
                (360.0, 290.0),
                (310.0, 80.0),
                (160.0, 60.0),
            ],
            closed: false,
            thickness_scale: 0.78,
        }]),
        _ => None,
    }
}

// These variants are only selected after the source image strongly signals a
// cursive hand. They preserve familiar descender loops without forcing them on
// print-like samples.
#[allow(clippy::too_many_lines)]
const fn looped_recipes_for_character(character: char) -> Option<&'static [StrokeRecipe]> {
    match character {
        'f' => Some(&[
            StrokeRecipe {
                points: &[
                    (235.0, -170.0),
                    (245.0, 390.0),
                    (140.0, 610.0),
                    (225.0, 760.0),
                    (345.0, 680.0),
                    (330.0, 520.0),
                    (245.0, 400.0),
                ],
                closed: false,
                thickness_scale: 0.8,
            },
            StrokeRecipe {
                points: &[(105.0, 405.0), (350.0, 405.0)],
                closed: false,
                thickness_scale: 0.56,
            },
        ]),
        'g' => Some(&[
            StrokeRecipe {
                points: &[
                    (335.0, 390.0),
                    (190.0, 365.0),
                    (145.0, 205.0),
                    (250.0, 85.0),
                    (385.0, 150.0),
                    (360.0, 315.0),
                    (270.0, 390.0),
                ],
                closed: true,
                thickness_scale: 0.76,
            },
            StrokeRecipe {
                points: &[
                    (350.0, 200.0),
                    (390.0, -40.0),
                    (360.0, -230.0),
                    (265.0, -300.0),
                    (175.0, -205.0),
                    (195.0, -65.0),
                    (295.0, -90.0),
                    (345.0, -220.0),
                ],
                closed: false,
                thickness_scale: 0.7,
            },
        ]),
        'j' => Some(&[
            StrokeRecipe {
                points: &[
                    (320.0, 420.0),
                    (305.0, 20.0),
                    (275.0, -180.0),
                    (180.0, -270.0),
                    (100.0, -180.0),
                    (135.0, -55.0),
                    (235.0, -90.0),
                    (285.0, -210.0),
                ],
                closed: false,
                thickness_scale: 0.74,
            },
            StrokeRecipe {
                points: &[
                    (300.0, 610.0),
                    (320.0, 650.0),
                    (340.0, 610.0),
                    (320.0, 570.0),
                ],
                closed: true,
                thickness_scale: 0.42,
            },
        ]),
        'p' => Some(&[
            StrokeRecipe {
                points: &[
                    (145.0, 420.0),
                    (145.0, 10.0),
                    (180.0, -175.0),
                    (265.0, -270.0),
                    (345.0, -190.0),
                    (305.0, -65.0),
                ],
                closed: false,
                thickness_scale: 0.82,
            },
            StrokeRecipe {
                points: &[
                    (175.0, 385.0),
                    (325.0, 400.0),
                    (400.0, 265.0),
                    (320.0, 90.0),
                    (165.0, 125.0),
                ],
                closed: true,
                thickness_scale: 0.74,
            },
        ]),
        'q' => Some(&[
            StrokeRecipe {
                points: &[
                    (330.0, 390.0),
                    (185.0, 360.0),
                    (145.0, 200.0),
                    (255.0, 85.0),
                    (390.0, 155.0),
                    (350.0, 325.0),
                ],
                closed: true,
                thickness_scale: 0.74,
            },
            StrokeRecipe {
                points: &[
                    (370.0, 170.0),
                    (390.0, -15.0),
                    (365.0, -220.0),
                    (285.0, -305.0),
                    (205.0, -235.0),
                    (220.0, -115.0),
                    (295.0, -80.0),
                ],
                closed: false,
                thickness_scale: 0.78,
            },
        ]),
        'y' => Some(&[StrokeRecipe {
            points: &[
                (145.0, 420.0),
                (255.0, 105.0),
                (350.0, 420.0),
                (355.0, 15.0),
                (315.0, -220.0),
                (220.0, -295.0),
                (125.0, -205.0),
                (150.0, -65.0),
                (255.0, -95.0),
                (310.0, -225.0),
            ],
            closed: false,
            thickness_scale: 0.72,
        }]),
        _ => None,
    }
}

fn render_accent(
    accent: Accent,
    style: GlyphStyle,
    base_character: char,
    seed: u64,
) -> Vec<Vec<(i16, i16)>> {
    let top = if base_character.is_ascii_uppercase() {
        style.ascender_height + 60.0
    } else {
        style.body_height + 170.0
    };
    let thickness = (style.stroke_width * 0.34).max(14.0);
    let recipe = match accent {
        Accent::Acute => StrokeRecipe {
            points: &[(270.0, 0.0), (350.0, 90.0)],
            closed: false,
            thickness_scale: 1.0,
        },
        Accent::Grave => StrokeRecipe {
            points: &[(330.0, 0.0), (250.0, 90.0)],
            closed: false,
            thickness_scale: 1.0,
        },
        Accent::Circumflex => StrokeRecipe {
            points: &[(220.0, 20.0), (290.0, 100.0), (360.0, 20.0)],
            closed: false,
            thickness_scale: 0.9,
        },
        Accent::Tilde => StrokeRecipe {
            points: &[(220.0, 45.0), (270.0, 90.0), (330.0, 20.0), (380.0, 70.0)],
            closed: false,
            thickness_scale: 0.8,
        },
        Accent::Diaeresis => StrokeRecipe {
            points: &[
                (220.0, 45.0),
                (250.0, 85.0),
                (220.0, 125.0),
                (190.0, 85.0),
                (220.0, 45.0),
            ],
            closed: true,
            thickness_scale: 0.7,
        },
        Accent::Ring => StrokeRecipe {
            points: &[
                (290.0, 20.0),
                (240.0, 70.0),
                (290.0, 120.0),
                (340.0, 70.0),
                (290.0, 20.0),
            ],
            closed: true,
            thickness_scale: 0.65,
        },
        Accent::Cedilla => StrokeRecipe {
            points: &[(280.0, -40.0), (240.0, -140.0), (320.0, -190.0)],
            closed: false,
            thickness_scale: 0.7,
        },
        Accent::Slash => StrokeRecipe {
            points: &[(160.0, 50.0), (410.0, 620.0)],
            closed: false,
            thickness_scale: 0.7,
        },
    };

    if matches!(accent, Accent::Diaeresis) {
        let mut contours = render_recipe(recipe, style, thickness, seed ^ 0x44, -55.0, top);
        contours.extend(render_recipe(
            recipe,
            style,
            thickness,
            seed ^ 0x55,
            75.0,
            top,
        ));
        return contours;
    }

    render_recipe(recipe, style, thickness, seed ^ 0x77, 0.0, top)
}

#[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
fn render_recipe(
    recipe: StrokeRecipe,
    style: GlyphStyle,
    thickness: f32,
    seed: u64,
    offset_x: f32,
    offset_y: f32,
) -> Vec<Vec<(i16, i16)>> {
    let styled_points = recipe
        .points
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let channel = u32::try_from(index).unwrap_or(0);
            let jitter_x = noise(seed, channel).mul_add(style.waviness * 0.18, 0.0);
            let jitter_y = noise(seed.rotate_left(7), channel).mul_add(style.waviness * 0.2, 0.0);
            let shifted_y = *y + offset_y;
            let height_shift = if shifted_y > style.body_height {
                style.ascender_height - 760.0
            } else if shifted_y < 0.0 {
                -style.descender_depth - shifted_y.abs()
            } else {
                style.baseline_lift
            };
            let shifted_x = *x + offset_x;
            // Cursive samples need a stronger shared shear than the default
            // handwritten grammar. A small baseline rise keeps the strokes
            // moving forward without inventing disconnected connectors.
            let cursive_slant = style.cursive_score * 136.0;
            let baseline_flow = (shifted_x - 280.0) * style.cursive_score * 0.045;
            let scaled_x = (shifted_x - 280.0).mul_add(
                style.width_scale,
                (style.slant + cursive_slant).mul_add(shifted_y / 700.0, 280.0),
            ) + jitter_x;
            let scaled_y = shifted_y + height_shift + baseline_flow + jitter_y;
            (scaled_x, scaled_y)
        })
        .collect::<Vec<_>>();

    let smoothed_points = round_stroke_points(&styled_points, recipe.closed, style.cursive_score);
    if recipe.closed {
        return render_closed_stroke(&smoothed_points, thickness);
    }

    render_open_stroke(&smoothed_points, thickness, style.cursive_score)
}

#[allow(clippy::cast_precision_loss)]
fn round_stroke_points(points: &[(f32, f32)], closed: bool, cursive_score: f32) -> Vec<(f32, f32)> {
    let passes = if cursive_score >= 0.72 {
        2
    } else {
        usize::from(cursive_score >= 0.32)
    };
    let mut rounded = points.to_vec();

    for _ in 0..passes {
        if rounded.len() < 2 {
            break;
        }

        let mut next = Vec::with_capacity(rounded.len().saturating_mul(2));
        if !closed && let Some(first) = rounded.first().copied() {
            next.push(first);
        }

        let segment_count = if closed {
            rounded.len()
        } else {
            rounded.len().saturating_sub(1)
        };
        for index in 0..segment_count {
            let left = rounded[index];
            let right = rounded[(index + 1) % rounded.len()];
            next.push((
                left.0.mul_add(0.75, right.0 * 0.25),
                left.1.mul_add(0.75, right.1 * 0.25),
            ));
            next.push((
                left.0.mul_add(0.25, right.0 * 0.75),
                left.1.mul_add(0.25, right.1 * 0.75),
            ));
        }

        if !closed && let Some(last) = rounded.last().copied() {
            next.push(last);
        }
        rounded = next;
    }

    rounded
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::suboptimal_flops
)]
fn render_open_stroke(
    points: &[(f32, f32)],
    thickness: f32,
    cursive_score: f32,
) -> Vec<Vec<(i16, i16)>> {
    if points.len() < 2 {
        return Vec::new();
    }

    let mut left_edge = Vec::with_capacity(points.len());
    let mut right_edge = Vec::with_capacity(points.len());

    for (index, point) in points.iter().enumerate() {
        let previous = if index == 0 {
            *point
        } else {
            points[index - 1]
        };
        let next = if index + 1 >= points.len() {
            *point
        } else {
            points[index + 1]
        };
        let tangent_x = next.0 - previous.0;
        let tangent_y = next.1 - previous.1;
        let length = tangent_x.hypot(tangent_y).max(1.0);
        let normal_x = -tangent_y / length;
        let normal_y = tangent_x / length;
        let edge_ratio = (index as f32) / (points.len().saturating_sub(1).max(1) as f32);
        let from_terminal = edge_ratio.min(1.0 - edge_ratio);
        let terminal_taper = 1.0 - (cursive_score * 0.48);
        let taper = terminal_taper + ((1.0 - terminal_taper) * (from_terminal * 3.0).min(1.0));
        let edge_thickness = thickness * taper;
        left_edge.push((
            round_to_i16(point.0 + normal_x * edge_thickness),
            round_to_i16(point.1 + normal_y * edge_thickness),
        ));
        right_edge.push((
            round_to_i16((normal_x * edge_thickness).mul_add(-0.86, point.0)),
            round_to_i16((normal_y * edge_thickness).mul_add(-0.86, point.1)),
        ));
    }

    right_edge.reverse();
    left_edge.extend(right_edge);
    vec![left_edge]
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::suboptimal_flops)]
fn render_closed_stroke(points: &[(f32, f32)], thickness: f32) -> Vec<Vec<(i16, i16)>> {
    if points.len() < 4 {
        return Vec::new();
    }

    let mut outer = Vec::with_capacity(points.len());
    let mut inner = Vec::with_capacity(points.len());

    for index in 0..points.len() {
        let previous = points[(index + points.len() - 1) % points.len()];
        let current = points[index];
        let next = points[(index + 1) % points.len()];
        let tangent_x = next.0 - previous.0;
        let tangent_y = next.1 - previous.1;
        let length = tangent_x.hypot(tangent_y).max(1.0);
        let normal_x = -tangent_y / length;
        let normal_y = tangent_x / length;

        outer.push((
            round_to_i16(current.0 + normal_x * thickness),
            round_to_i16(current.1 + normal_y * thickness),
        ));
        inner.push((
            round_to_i16((normal_x * thickness).mul_add(-0.8, current.0)),
            round_to_i16((normal_y * thickness).mul_add(-0.8, current.1)),
        ));
    }

    inner.reverse();
    vec![outer, inner]
}

fn noise(seed: u64, channel: u32) -> f32 {
    let shifted = seed.rotate_left(channel % 63);
    let narrowed = u16::try_from((shifted ^ u64::from(channel).wrapping_mul(0x9E37_79B9)) & 0xFFFF)
        .unwrap_or(0);
    f32::from(narrowed).mul_add(2.0 / 65_535.0, -1.0)
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
