//! Minimal text-on-path support for the Blend2D backend.
//!
//! This intentionally implements the common case only: horizontal shaped text
//! follows the first non-empty contour and each glyph remains rigid.

use super::*;
use kurbo::{CubicBez, Line, PathSeg, Point, QuadBez,
    ParamCurve, ParamCurveArclen, ParamCurveDeriv,
};

impl BLContext<'_> {
    /// Shapes and draws `text` along the first non-empty contour of `path`.
    ///
    /// Glyphs are positioned by advance width and rotated to the local tangent.
    /// Empty text and paths without drawable segments are successful no-ops.
    ///
    /// https://www.w3.org/TR/SVG2/text.html#TextLayoutPath
    pub fn draw_text_on_path(&mut self, path: &BLPath, font: &BLFont, text: &str,
        paint: TextPathPaint<'_>, options: TextPathOptions) -> Result<(), BLErr> {
        if text.is_empty() { return Ok(()); }
        let measured = MeasuredPath::new(path);
        if  measured.length == 0.0 { return Ok(()); }

        let (mut outlines, mut cursor) = (BLPath::new(), options.start_offset);
        let glyphs = BLGlyphBuffer::shape(font, text)?;
        let font_matrix = font_matrix(font);

        for (glyph_id, placement) in glyphs.items() {
            let advance = font_matrix.map(&placement.advance);
            let offset  = font_matrix.map(&placement.placement);
            let center_distance = cursor + offset.0 + advance.0 * 0.5;
            cursor += advance.0;

            let Some((point, tangent)) = measured.sample(center_distance) else { continue; };
            let normal = (-tangent.1, tangent.0);
            let baseline = options.baseline_offset + offset.1;
            let origin = (point.x - tangent.0 * advance.0 * 0.5 + normal.0 * baseline,
                          point.y - tangent.1 * advance.0 * 0.5 + normal.1 * baseline);
            let transform = BLMatrix2D::new([
                tangent.0, tangent.1, -tangent.1, tangent.0, origin.0, origin.1,
            ]);

            bl_result!(bl_font_get_glyph_outlines(&font.0, glyph_id,
                &transform, &mut outlines.0, None, null_mut()))?;
        }

        if outlines.get_size() != 0 {
            let origin = BLPoint::new();
            match paint {
                TextPathPaint::Fill(style) => bl_result!(bl_context_fill_path_d_ext(
                    &mut self.0, &origin, &outlines.0, style.as_ptr()))?,
                TextPathPaint::Stroke(style) => bl_result!(bl_context_stroke_path_d_ext(
                    &mut self.0, &origin, &outlines.0, style.as_ptr()))?,
            }
        }   Ok(())
    }
}

/// How glyph outlines are painted.
#[derive(Clone, Copy)] pub enum TextPathPaint<'a> {
    Fill(&'a dyn B2DStyle), Stroke(&'a dyn B2DStyle),
    // FillAndStroke { fill: &'a dyn B2DStyle, stroke: &'a dyn B2DStyle },
}

/// Placement offsets used by [`BLContext::draw_text_on_path`].
#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct TextPathOptions {
    /// Distance from the path start to the first glyph, in user units.
    pub start_offset: f64,
    /// Perpendicular distance from the path to the text baseline.
    pub baseline_offset: f64,
    // pub anchor: TextPathAnchor, // Start, middle, or end alignment.
    // pub side: TextPathSide,     // Normal or reversed path side.
    // pub spacing: TextPathSpacing, // Exact or automatically adjusted spacing.
    // pub method: TextPathMethod, // Rigid glyphs or stretched glyph outlines.
    // pub contour: usize,         // Select a contour instead of always using the first.
}

// Future shaping options (direction, script, language, and OpenType features)
// belong beside this RAII wrapper if BLFont's defaults are no longer sufficient.
struct BLGlyphBuffer(BLGlyphBufferCore); // XXX: better move to blend2d.rs?
impl Drop for BLGlyphBuffer {
    fn drop(&mut self) { bl_debug!(bl_glyph_buffer_destroy(&mut self.0)); }
}

impl   BLGlyphBuffer {
    fn shape(font: &BLFont, text: &str) -> Result<Self, BLErr> {
        let mut core = object_init();
        bl_debug!(bl_glyph_buffer_init(&mut core));
        let mut buffer = Self(core);

        bl_result!(bl_glyph_buffer_set_text(&mut buffer.0, text.as_ptr().cast(),
            text.len(), BLTextEncoding::BL_TEXT_ENCODING_UTF8))?;
        bl_result!(bl_font_shape(&font.0, &mut buffer.0))?;
        Ok(buffer)
    }

    fn items(&self) -> impl Iterator<Item = (BLGlyphId, &BLGlyphPlacement)> + '_ {
        // SAFETY: `self.0` is a live glyph buffer for the duration of the call.
        let len = unsafe { bl_glyph_buffer_get_size(&self.0) };
        let (ids, placements): (&[BLGlyphId], &[BLGlyphPlacement]) =
            if len == 0 { (&[], &[]) } else { unsafe {(
                // SAFETY: after successful shaping Blend2D exposes parallel glyph
                // and placement arrays of `len` elements owned by this buffer.
                core::slice::from_raw_parts(bl_glyph_buffer_get_content(&self.0), len),
                core::slice::from_raw_parts(
                    bl_glyph_buffer_get_placement_data(&self.0), len),
            )} };
        ids.iter().copied().zip(placements)
    }
}

// Keep this linear-only: glyph outlines already receive Blend2D's font matrix.
// A future writing-mode implementation may expose both inline and block vectors.
#[derive(Clone, Copy)] struct FontMatrix([f64; 4]);

impl FontMatrix {
    fn map(self, value: &BLPointI) -> (f64, f64) {
        let [m00, m01, m10, m11] = self.0;
        (value.x as f64 * m00 + value.y as f64 * m10,
         value.x as f64 * m01 + value.y as f64 * m11)
    }
}

fn font_matrix(font: &BLFont) -> FontMatrix {
    let mut matrix = object_init();
    bl_debug!(bl_font_get_matrix(&font.0, &mut matrix));
    // SAFETY: `bl_font_get_matrix` initialized the matrix member on success.
    FontMatrix(unsafe { *matrix.__bindgen_anon_1.m })
}

struct MeasuredSegment {
    segment: PathSeg, start: f64, length: f64,
    // arclen_lut: ..., // Add only if profiling shows repeated inversion is costly.
}
struct MeasuredPath {
    segments: Vec<MeasuredSegment>,  length: f64,
    // contours: ..., // Enables explicit selection or continuation across contours.
    // closed: bool,  // Enables wrapping text around a closed contour.
}

const ARC_LENGTH_ACCURACY: f64 = 0.01;

impl   MeasuredPath {
    fn new(path: &BLPath) -> Self {
        let (mut contour_start, mut current) = (None, None);
        let (mut segments, mut length, mut started) = (Vec::new(), 0.0, false);

        for item in path.iter() {
            let segment = match item {
                BLPathItem::MoveTo(point) => {
                    if started { break; }
                    contour_start = Some(point);
                    current = Some(point);
                    continue;
                }
                BLPathItem::LineTo(end) => current.map(|start| {
                    current = Some(end);
                    PathSeg::Line(Line::new(point(start), point(end)))
                }),
                BLPathItem::QuadTo(control, end) => current.map(|start| {
                    current = Some(end);
                    PathSeg::Quad(QuadBez::new(point(start), point(control), point(end)))
                }),
                BLPathItem::CubicTo(control1, control2, end) => current.map(|start| {
                    current = Some(end);
                    PathSeg::Cubic(CubicBez::new(point(start),
                        point(control1), point(control2), point(end)))
                }),
                BLPathItem::Close => match (current, contour_start) {
                    (Some(start), Some(end)) if start.x != end.x || start.y != end.y => {
                        current = Some(end);
                        Some(PathSeg::Line(Line::new(point(start), point(end))))
                    }
                    _ => None,
                },
            };

            if let Some(segment) = segment {
                let segment_length = segment.arclen(ARC_LENGTH_ACCURACY);
                if segment_length > 0.0 {
                    segments.push(MeasuredSegment {
                        segment, start: length, length: segment_length,
                    });
                    length += segment_length;
                    started = true;
                }
            }
        }
        Self { segments, length }
    }

    fn sample(&self, distance: f64) -> Option<(Point, (f64, f64))> {
        if !(0.0..=self.length).contains(&distance) { return None; }
        let index = self.segments.partition_point(|item| item.start + item.length < distance);
        let measured = self.segments.get(index).or_else(|| self.segments.last())?;
        let local = (distance - measured.start).clamp(0.0, measured.length);
        let t = measured.segment.inv_arclen(local, ARC_LENGTH_ACCURACY);
        let point = measured.segment.eval(t);
        Some((point, segment_tangent(measured.segment, t)))
    }
}

fn point(value: BLPoint) -> Point { Point::new(value.x, value.y) }

fn segment_tangent(segment: PathSeg, t: f64) -> (f64, f64) {
    let vector = match segment {
        PathSeg::Line(line) => line.p1 - line.p0,
        PathSeg::Quad(quad) => quad.deriv().eval(t).to_vec2(),
        PathSeg::Cubic(cubic) => cubic.deriv().eval(t).to_vec2(),
    };
    let length = vector.hypot();
    if  length == 0.0 {
        let delta = 1e-6;
        let chord = segment.eval((t + delta).min(1.0)) - segment.eval((t - delta).max(0.0));
        let chord_length = chord.hypot();
        debug_assert!(chord_length > 0.0);
        return if chord_length > 0.0 {
            (chord.x / chord_length, chord.y / chord_length)
        } else { (1.0, 0.0) };
    }
    debug_assert!(length > 0.0);
    (vector.x / length, vector.y / length)
}

#[cfg(test)] mod tests { use super::*;

    #[test] fn measures_and_samples_a_line() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.move_to(( 10.0, 20.0).into());
        path.line_to((110.0, 20.0).into());

        let measured = MeasuredPath::new(&path);
        let (point, tangent) = measured.sample(25.0).unwrap();
        assert!((measured.length - 100.0).abs() < 1e-9);
        assert_eq!(point, Point::new(35.0, 20.0));
        assert_eq!(tangent, (1.0, 0.0));
        Ok(())
    }

    #[test] fn ignores_later_contours() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.move_to((  0.0, 0.0).into());
        path.line_to(( 10.0, 0.0).into());
        path.move_to((100.0, 0.0).into());
        path.line_to((200.0, 0.0).into());

        assert!((MeasuredPath::new(&path).length - 10.0).abs() < 1e-9);
        Ok(())
    }
}
