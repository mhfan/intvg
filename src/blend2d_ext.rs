//! Drawing extensions for the Blend2D backend.
//!
//! Includes minimal text-on-path support plus scoped path clipping and alpha
//! masks implemented with bounded offscreen layers.

use super::*;
use kurbo::{CubicBez, Line as KLine, Point as KPoint, QuadBez,
    PathSeg, ParamCurve, ParamCurveArclen, ParamCurveDeriv,
};

impl BLContext {
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
        let (glyphs, fmatrix) = (font.shape(text)?, font.get_matrix());

        for (glyph_id, place) in glyphs.items() {
            let advance = (place.advance.x   as f64, place.advance.y   as f64).into();
            let advance = fmatrix.map_point(advance);
            let offset  = (place.placement.x as f64, place.placement.y as f64).into();
            let offset  = fmatrix.map_point(offset);
            let center_distance = cursor + offset.x + advance.x * 0.5;
            cursor += advance.x;

            let Some((point, tangent)) = measured.sample(center_distance) else { continue; };
            let normal = (-tangent.1, tangent.0);
            let baseline = options.baseline_offset + offset.y;
            let origin = (point.x - tangent.0 * advance.x * 0.5 + normal.0 * baseline,
                          point.y - tangent.1 * advance.x * 0.5 + normal.1 * baseline);
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

    /// Renders `content` through an alpha mask produced by `mask`.
    ///
    /// `bounds` is the affected rectangle in target-device pixels. Both
    /// callbacks inherit this context's final coordinate mapping as their meta
    /// transform, but start with an identity user transform, fresh paint state,
    /// and transparent pixels. They can therefore freely reset or replace their
    /// user transform without disturbing the internal layer offset. The composed
    /// result is then drawn through this context's current clip, alpha, and
    /// composition operator.
    ///
    /// # Limitations
    ///
    /// This API **does not support luminance masks**; it supports alpha masks
    /// only. The mask is rendered into `A8`, so RGB color and luminance are
    /// discarded. For example, opaque black produces a fully opaque mask here,
    /// whereas SVG's default luminance-mask semantics make it transparent.
    ///
    /// A non-positive extent is a successful no-op. Keeping `bounds` tight
    /// limits temporary storage to one `PRGB32` and one `A8` image of `w * h`
    /// pixels.
    pub fn render_mask_in<M, C>(&mut self, bounds: BLRectI, mask: M, content: C) ->
        Result<(), BLErr> where
            M: FnOnce(&mut BLContext) -> Result<(), BLErr>,
            C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        let target = self.get_target_size();
        if bounds.w <= 0 || bounds.h <= 0 { return Ok(()); }
        let (max_x, max_y) = (target.width() as i32, target.height() as i32);
        let (x0, y0) = (bounds.x.clamp(0, max_x), bounds.y.clamp(0, max_y));
        let (x1, y1) = (bounds.x.saturating_add(bounds.w) .clamp(0, max_x),
                        bounds.y.saturating_add(bounds.h) .clamp(0, max_y));
        if x0 >= x1 || y0 >= y1 { return Ok(()); }
        let bounds = BLRectI { x: x0, y: y0, w: (x1 - x0), h: (y1 - y0) };

        fn render_layer(bounds: &BLRectI, transform: &BLMatrix2D, fmt: BLFormat,
            render: impl FnOnce(&mut BLContext) -> Result<(), BLErr>) ->
            Result<BLImage, BLErr> {
            let mut layer = BLContext::new(bounds.w as _, bounds.h as _, fmt)?;
            layer.clear_rect(None)?;
            layer.reset_transform(Some(transform));
            layer.user_to_meta();
            render(&mut layer)?;
            layer.end()
        }

        let mut trfm = self.final_transform();
        trfm.post_translate((-bounds.x as f64, -bounds.y as f64).into());
        let content = render_layer(&bounds, &trfm, BLFormat::BL_FORMAT_PRGB32, content)?;
        let mask = render_layer(&bounds, &trfm, BLFormat::BL_FORMAT_A8, mask)?;
        let area = BLRectI { x: 0, y: 0, w: bounds.w, h: bounds.h };

        let mut masked = BLContext::from_image(content)?;
        masked.reset_transform(None);
        masked.set_comp_op(BLCompOp::BL_COMP_OP_DST_IN);
        masked.blit_image((0, 0).into(), &mask, &area)?;
        let image = masked.end()?;

        let transform = self.user_transform();
        self.reset_transform(None);
        let result = self.blit_image((bounds.x, bounds.y).into(), &image, &area);
        self.reset_transform(Some(&transform));     result
    }

    /// Renders `content` through the filled area of `path`, allocating only its
    /// device-space intersection with the target image.
    ///
    /// This is the path equivalent of a scoped clip: Blend2D has no persistent
    /// path-clip state, so callers provide the rendering operation as a
    /// closure. The context's current fill rule is used for the clip path.
    pub fn clip_to_path<C>(&mut self, path: &BLPath, content: C) ->
        Result<(), BLErr> where C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        let Some(bounds) = self.path_device_bounds(path)? else { return Ok(()); };
        self.clip_to_path_in(bounds, path, content)
    }

    /// Equivalent to [`BLContext::clip_to_path`], but uses caller-provided
    /// target-device bounds to avoid measuring and transforming `path`.
    pub fn clip_to_path_in<C>(&mut self, bounds: BLRectI, path: &BLPath, content: C) ->
        Result<(), BLErr> where C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        // SAFETY: `self.0` is a live context and the getter only reads its state.
        let fill_rule = unsafe { bl_context_get_fill_rule(&self.0) };
        self.render_mask_in(bounds, |mask| {
            mask.set_fill_rule(fill_rule);
            mask.fill_geometry_rgba32(path, BLRgba32::new(255, 255, 255, 255))
        }, content)
    }

    fn path_device_bounds(&self, path: &BLPath) -> Result<Option<BLRectI>, BLErr> {
        if path.get_size() == 0 { return Ok(None); }

        let mut device_path = BLPath::new();
        device_path.add_transformed_path(path, &self.final_transform())?;
        let bbox = device_path.get_bounding_box()?;
        let target = self.get_target_size();
        if ![bbox.x0, bbox.y0, bbox.x1, bbox.y1].iter().all(|v| v.is_finite()) {
            return Ok(None);
        }
        let (max_x, max_y) = (target.width() as f64, target.height() as f64);
        let (x0, y0) = (bbox.x0.floor().clamp(0.0, max_x),
                        bbox.y0.floor().clamp(0.0, max_y));
        let (x1, y1) = (bbox.x1.ceil() .clamp(0.0, max_x),
                        bbox.y1.ceil() .clamp(0.0, max_y));
        Ok((x0 < x1 && y0 < y1).then_some(BLRectI {
            x: x0 as _, y: y0 as _, w: (x1 - x0) as _, h: (y1 - y0) as _
        }))
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

struct MeasuredSegment {
    segment: PathSeg, start: f64, length: f64,
    // arclen_lut: ..., // Add only if profiling shows repeated inversion is costly.
}
struct MeasuredPath {
    segments: Vec<MeasuredSegment>,  length: f64,
    // contours: ..., // Enables explicit selection or continuation across contours.
    // closed: bool,  // Enables wrapping text around a closed contour.
}

const ACCURACY_TOLERANCE: f64 = 0.01;

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
                    PathSeg::Line(KLine::new(start, end))
                }),
                BLPathItem::QuadTo(control, end) => current.map(|start| {
                    current = Some(end);
                    PathSeg::Quad(QuadBez::new(start, control, end))
                }),
                BLPathItem::CubicTo(control1, control2, end) => current.map(|start| {
                    current = Some(end);
                    PathSeg::Cubic(CubicBez::new(start, control1, control2, end))
                }),
                BLPathItem::Close => match (current, contour_start) {
                    (Some(start), Some(end)) if start.x != end.x || start.y != end.y => {
                        current = Some(end);
                        Some(PathSeg::Line(KLine::new(start, end)))
                    }
                    _ => None,
                },
            };

            if let Some(segment) = segment {
                let segment_length = segment.arclen(ACCURACY_TOLERANCE);
                if  segment_length > 0.0 {
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

    fn sample(&self, distance: f64) -> Option<(KPoint, (f64, f64))> {
        if !(0.0..=self.length).contains(&distance) { return None; }
        let index = self.segments.partition_point(|item| item.start + item.length < distance);
        let measured = self.segments.get(index).or_else(|| self.segments.last())?;
        let local = (distance - measured.start).clamp(0.0, measured.length);
        let t = measured.segment.inv_arclen(local, ACCURACY_TOLERANCE);
        let point = measured.segment.eval(t);
        Some((point, segment_tangent(measured.segment, t)))
    }
}

impl From<BLPoint> for KPoint {
    fn from(value: BLPoint) -> Self { Self::new(value.x, value.y) }
}

fn segment_tangent(segment: PathSeg, t: f64) -> (f64, f64) {
    let vector = match segment {
        PathSeg::Line(line)   => line.p1 - line.p0,
        PathSeg::Quad(quad)   =>  quad.deriv().eval(t).to_vec2(),
        PathSeg::Cubic(cubic) => cubic.deriv().eval(t).to_vec2(),
    };
    let length = vector.hypot();
    if  length == 0.0 {     let delta = 1e-6;
        let chord = segment.eval((t + delta).min(1.0)) -
                    segment.eval((t - delta).max(0.0));
        let chord_length = chord.hypot();

        debug_assert!(chord_length > 0.0);
        return if chord_length > 0.0 {
            (chord.x / chord_length, chord.y / chord_length)
        } else { (1.0, 0.0) };
    }   debug_assert!(length > 0.0);
    (vector.x / length, vector.y / length)
}

#[cfg(test)] mod tests { use super::*;

    fn alpha(image: &BLImage, x: usize, y: usize) -> u8 {
        image.pixels().unwrap()[y * image.stride() as usize + x * 4 + 3]
    }

    #[test] fn measures_and_samples_a_line() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.move_to(( 10.0, 20.0).into());
        path.line_to((110.0, 20.0).into());

        let measured = MeasuredPath::new(&path);
        let (point, tangent) = measured.sample(25.0).unwrap();
        assert!((measured.length - 100.0).abs() < 1e-9);
        assert_eq!(point, KPoint::new(35.0, 20.0));
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

    #[test] fn clips_content_to_path() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.add_rect(&(2.0, 1.0, 3.0, 4.0).into(), None);

        let mut ctx = BLContext::new(8, 8, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_rect(None)?;
        ctx.clip_to_path(&path, |layer|
            layer.fill_all_rgba32(BLRgba32::new(255, 0, 0, 255)))?;
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 3, 2), 255);
        assert_eq!(alpha(&image, 0, 0), 0);
        assert_eq!(alpha(&image, 6, 6), 0);
        Ok(())
    }

    #[test] fn resetting_layer_transform_preserves_nonzero_clip_offset() ->
        Result<(), BLErr> {
        let rect: BLRect = (3.0, 2.0, 2.0, 3.0).into();
        let mut path = BLPath::new();
        path.add_rect(&rect, None);

        let mut ctx = BLContext::new(8, 8, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_rect(None)?;
        ctx.clip_to_path(&path, |layer| {
            layer.reset_transform(None);
            layer.fill_geometry_rgba32(&rect, BLRgba32::new(255, 0, 0, 255))
        })?;
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 3, 2), 255);
        assert_eq!(alpha(&image, 4, 4), 255);
        assert_eq!(alpha(&image, 2, 2), 0);
        assert_eq!(alpha(&image, 5, 4), 0);
        Ok(())
    }

    #[test] fn mask_and_content_share_the_parent_transform() -> Result<(), BLErr> {
        let mask_rect: BLRect = (0.0, 0.0, 2.0, 4.0).into();
        let mut ctx = BLContext::new(8, 4, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_rect(None)?;
        ctx.translate((2.0, 0.0).into());
        let transform = ctx.user_transform().get_values();
        ctx.render_mask_in((0, 0, 8, 4).into(),
            |mask| mask.fill_geometry_rgba32(
                &mask_rect, BLRgba32::new(255, 255, 255, 255)),
            |content| content.fill_all_rgba32(BLRgba32::new(0, 255, 0, 255)))?;
        assert_eq!(ctx.user_transform().get_values(), transform);
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 2, 1), 255);
        assert_eq!(alpha(&image, 0, 1), 0);
        assert_eq!(alpha(&image, 4, 1), 0);
        Ok(())
    }

    #[test] fn automatic_bounds_follow_the_final_transform() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.add_rect(&(0.0, 0.0, 2.0, 3.0).into(), None);

        let mut ctx = BLContext::new(10, 10, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_rect(None)?;
        ctx.translate((7.0, 2.0).into());
        ctx.rotate(core::f64::consts::FRAC_PI_2, None);
        ctx.clip_to_path(&path, |layer|
            layer.fill_all_rgba32(BLRgba32::new(0, 0, 255, 255)))?;
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 5, 3), 255);
        assert_eq!(alpha(&image, 7, 3), 0);
        assert_eq!(alpha(&image, 3, 3), 0);
        Ok(())
    }

    #[test] fn empty_and_offscreen_paths_are_noops() -> Result<(), BLErr> {
        let mut called = false;
        let mut ctx = BLContext::new(4, 4, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clip_to_path(&BLPath::new(), |_| { called = true; Ok(()) })?;

        let mut path = BLPath::new();
        path.add_rect(&(10.0, 10.0, 2.0, 2.0).into(), None);
        ctx.clip_to_path(&path, |_| { called = true; Ok(()) })?;

        assert!(!called);
        Ok(())
    }
}
