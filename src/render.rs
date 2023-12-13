
use crate::tinyvg::*;
use tiny_skia as skia;
use std::{io, result::Result};

pub trait Render { fn render(&self, scale: f32) -> Result<skia::Pixmap, &str>; }

impl<R: io::Read, W: io::Write> Render for TinyVG<R, W> {
    fn render(&self, scale: f32) -> Result<skia::Pixmap, &str> {
        let mut pixmap = skia::Pixmap::new(
            (self.header.width  as f32 * scale).ceil() as _,
            (self.header.height as f32 * scale).ceil() as _).ok_or("Fail to create pixmap")?;

        // XXX: rendering up-scale and then scale down for anti-aliasing?
        let ts = skia::Transform::from_scale(scale, scale);
        let mut stroke = skia::Stroke { line_join: skia::LineJoin::Round,
            line_cap: skia::LineCap::Round, ..Default::default() };
        let err_msg = "Fail to build path";

        impl From<Rect> for skia::Rect {
            fn from(r: Rect) -> Self { unsafe { std::mem::transmute(r) } }
            //fn from(r: Rect) -> Self { skia::Rect::from_ltrb(r.l, r.t, r.r, r.b).unwrap() }
            //fn from(r: Rect) -> Self { skia::Rect::from_xywh(r.x, r.y, r.w, r.h).unwrap() }
        }

        for cmd in &self.commands {
            let mut pb = skia::PathBuilder::new();
            match cmd {     Command::EndOfDocument => (),

                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));  pb.close();

                    pixmap.fill_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, fill, ts)?, skia::FillRule::Winding, ts, None);
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    let paint = style_to_paint(self, fill, ts)?;
                    coll.iter().for_each(|rect|
                        pixmap.fill_rect((*rect).into(), &paint, ts, None));
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    let paint = style_to_paint(self, fill, ts)?;
                    for seg in coll {   let res = segment_to_path(seg)?;
                        //if res.1 { return Err("Got line width in fill path segment") }
                        pixmap.fill_path(&res.0, &paint, skia::FillRule::Winding, ts, None);
                    }
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    coll.iter().for_each(|line| {
                        pb.move_to(line.start.x, line.start.y);
                        pb.line_to(line.  end.x, line.  end.y);
                    }); stroke.width = *lwidth;

                    pixmap.stroke_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, line, ts)?, &stroke, ts, None);
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));

                    if !*strip { pb.close(); }  stroke.width = *lwidth;
                    pixmap.stroke_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, line, ts)?, &stroke, ts, None);
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, line, ts)?;
                    stroke.width = *lwidth;

                    for seg in coll {
                        stroke_segment_path(seg, &mut pixmap, &paint, &mut stroke, ts)?;
                    }
                }
                Command::OutlinePolyg(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));  pb.close();

                    let path = pb.finish().ok_or(err_msg)?;
                    pixmap.fill_path(&path, &style_to_paint(self, fill, ts)?,
                        skia::FillRule::Winding, ts, None);
                    stroke.width = *lwidth;

                    pixmap.stroke_path(&path, &style_to_paint(self, line, ts)?, &stroke, ts, None);
                }
                Command::OutlineRects(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill, ts)?;
                    let pline = style_to_paint(self, line, ts)?;
                    stroke.width = *lwidth;

                    coll.iter().for_each(|rect| {
                        let path = skia::PathBuilder::from_rect((*rect).into());
                        pixmap.fill_path(&path, &paint, skia::FillRule::Winding, ts, None);
                        pixmap.stroke_path(&path, &pline, &stroke, ts, None);
                    });
                }
                Command::OutlinePath (fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill, ts)?;
                    let pline = style_to_paint(self, line, ts)?;
                    stroke.width = *lwidth;

                    for seg in coll {   let res = segment_to_path(seg)?;
                        pixmap.fill_path(&res.0, &paint, skia::FillRule::Winding, ts, None);
                        if res.1 {
                            stroke_segment_path(seg, &mut pixmap, &pline, &mut stroke, ts)?;
                        } else { pixmap.stroke_path(&res.0, &pline, &stroke, ts, None); }
                    }
                }
            }
        }   Ok(pixmap)
    }   // rasterize
}

fn stroke_segment_path(seg: &Segment, pixmap: &mut skia::Pixmap, paint: &skia::Paint,
    stroke: &mut skia::Stroke, ts: skia::Transform) -> Result<(), &'static str> {
    let mut pb = skia::PathBuilder::new();
    pb.move_to(seg.start.x, seg.start.y);

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if 1 < pb.len() {   let err_msg = "no start";
                let start = pb.last_point().ok_or(err_msg)?;
                pixmap.stroke_path(&pb.finish().ok_or(err_msg)?, paint, stroke, ts, None);
                pb = skia::PathBuilder::new();  pb.move_to(start.x, start.y);
            }   stroke.width = width;
        }   process_segcmd(&mut pb, &cmd.instr);
    }

    pixmap.stroke_path(&pb.finish().ok_or("Fail build path from segments")?,
        paint, stroke, ts, None);   Ok(())
}

fn segment_to_path(seg: &Segment) -> Result<(skia::Path, bool), &'static str> {
    let mut pb = skia::PathBuilder::new();
    pb.move_to(seg.start.x, seg.start.y);
    let mut change_lw = false;

    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(&mut pb, &cmd.instr);
    }   Ok((pb.finish().ok_or("Fail build path from segments")?, change_lw))
}

fn process_segcmd(pb: &mut skia::PathBuilder, cmd: &SegInstr) {
    match cmd {     SegInstr::ClosePath => pb.close(),
        SegInstr::Line  { end } => pb.line_to(end.x, end.y),
        SegInstr::HLine { x }     => pb.line_to(*x, pb.last_point().unwrap().y),
        SegInstr::VLine { y }     => pb.line_to(pb.last_point().unwrap().x, *y),

        SegInstr::CubicBezier { ctrl, end } =>
            pb.cubic_to(ctrl.0.x, ctrl.0.y, ctrl.1.x, ctrl.1.y, end.x, end.y),
        SegInstr::ArcCircle  { large, sweep, radius, target } =>
            pb.arc_to(*radius, *radius, 0.0, *large, *sweep, target.x, target.y),

        SegInstr::ArcEllipse { large, sweep, radius,
                rotation, target } => pb.arc_to(radius.0, radius.1,
                *rotation, *large, *sweep, target.x, target.y),

        SegInstr::QuadBezier { ctrl, end } =>
            pb.quad_to(ctrl.x, ctrl.y, end.x, end.y),
    }
}

fn style_to_paint<'a, R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    style: &Style, ts: skia::Transform) ->
    Result<skia::Paint<'a>, &'static str> {
    impl From<RGBA8888> for skia::Color {  // XXX: why not use ColorU8 defaultly in skia?
        fn from(c: RGBA8888) -> Self { Self::from_rgba8(c.r, c.g, c.b, c.a) }
    }

    impl From<Point> for skia::Point {
        //fn from(pt: Point) -> Self { (pt.x, pt.y).into() }
        //fn from(pt: Point) -> Self { Self { x: pt.x, y: pt.y } }
        fn from(pt: Point) -> Self { unsafe { std::mem::transmute(pt) } }
    }

    let mut paint = skia::Paint::default(); // default BlendMode::SourceOver
    match style {   // paint.anti_alias is default true
        Style::FlatColor(idx) => paint.set_color(img.lookup_color(*idx).into()),

        Style::LinearGradient { points, cindex } => {
            paint.shader = skia::LinearGradient::new(points.0.into(), points.1.into(),
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, ts)
                .ok_or("Fail to create linear gradient shader")?;  //paint.anti_alias = false;
        }
        Style::RadialGradient { points, cindex } => {
            let (dx, dy) = (points.1.x - points.0.x, points.1.y - points.0.y);
            let radius = if dx.abs() < f32::EPSILON { dy.abs() }
                         else if dy.abs() < f32::EPSILON { dx.abs() }
                         else { (dx.powi(2) + dy.powi(2)).sqrt() };

            paint.shader = skia::RadialGradient::new(points.0.into(), points.1.into(), radius,
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, ts)
                .ok_or("Fail to create radial gradient shader")?;  //paint.anti_alias = false;
        }
    }   Ok(paint)
}

trait PathBuilderExt {  #[allow(clippy::too_many_arguments)]
    fn arc_to(&mut self, rx: f32, ry: f32, rotation: f32,
        large: bool, sweep: bool, x: f32, y: f32);
}

impl PathBuilderExt for skia::PathBuilder {
    fn arc_to(&mut self, rx: f32, ry: f32, rotation: f32,
        large: bool, sweep: bool, x: f32, y: f32) {
        let prev = match self.last_point() { Some(v) => v, None => return };

        let svg_arc = kurbo::SvgArc {   // lyon_geom: https://github.com/nical/lyon
            from: kurbo::Point::new(prev.x as _, prev.y as _),
              to: kurbo::Point::new( x as _,  y as _),
            radii: kurbo::Vec2::new(rx as _, ry as _),
            x_rotation: (rotation as f64).to_radians(), large_arc: large, sweep,
        };

        match kurbo::Arc::from_svg_arc(&svg_arc) {  None => self.line_to(x, y),
            Some(arc) => arc.to_cubic_beziers(0.1, |p1, p2, p|
                self.cubic_to(p1.x as _, p1.y as _, p2.x as _, p2.y as _, p.x as _, p.y as _)),
        }
    }
}

/* fn lerp_sRGB(c0: RGBA8888, c1: RGBA8888, f_unchecked: f32) -> RGBA8888 { // blend
    //  Color interpolation is needed in gradients and must performed in linear
    //  color space. This means that the value from the color table needs to be
    //  converted to linear color space, then each color component is
    //  interpolated linearly and the final color is then determined by
    //  converting the color back to the specified color space.
    let f = f32::clamp(f_unchecked, 0.0, 1.0);  RGBA8888 {
        r: gamma2linear(lerp(linear2gamma(c0.r), linear2gamma(c1.r), f)),
        g: gamma2linear(lerp(linear2gamma(c0.g), linear2gamma(c1.g), f)),
        b: gamma2linear(lerp(linear2gamma(c0.b), linear2gamma(c1.b), f)),
        a: (lerp(c0.a as f32 / 255.0, c1.a as f32 / 255.0, f) * 255.0) as _,
    }
}

fn lerp_value(src: f32, dst: f32, src_alpha: f32, dst_alpha: f32, fin_alpha: f32) -> f32 {
    mapToGamma((1.0 / fin_alpha) * (src_alpha * mapToLinear(src) +
               (1.0 - src_alpha) *  dst_alpha * mapToLinear(dst)))  // Alpha Blending
}

const sRGB_gamma: f32 = 2.2;
#[inline] fn  mapToLinear(v: f32) -> f32 { f32::powf(v, sRGB_gamma) }
#[inline] fn  mapToGamma (v: f32) -> f32 { f32::powf(v, 1.0 / sRGB_gamma) }
#[inline] fn gamma2linear(v: f32) ->  u8 { (255.0 * mapToGamma(v)) as _ }
#[inline] fn linear2gamma(v:  u8) -> f32 { mapToLinear(v as f32 / 255.0) }
#[inline] fn lerp(a: f32, b: f32, f: f32) -> f32 { a + (b - a) * f } */

