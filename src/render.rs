
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
        let trfm = skia::Transform::from_scale(scale, scale);
        let mut stroke = skia::Stroke { line_join: skia::LineJoin::Round,
            line_cap: skia::LineCap::Round, ..Default::default() };
        let err_msg = "Fail to build path";

        impl From<Rect> for skia::Rect {
            //fn from(r: Rect) -> Self { unsafe { std::mem::transmute(r) } }
            //fn from(r: Rect) -> Self { skia::Rect::from_ltrb(r.l, r.t, r.r, r.b).unwrap() }
            fn from(r: Rect) -> Self { skia::Rect::from_xywh(r.x, r.y, r.w, r.h).unwrap() }
        }

        let fillrule = skia::FillRule::Winding;
        for cmd in &self.commands {
            let mut pb = skia::PathBuilder::new();
            match cmd {     Command::EndOfDocument => (),

                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));  pb.close();

                    pixmap.fill_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, fill, trfm)?, fillrule, trfm, None);
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    coll.iter().for_each(|rect| pb.push_rect((*rect).into()));
                    pixmap.fill_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, fill, trfm)?, fillrule, trfm, None);
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    for seg in coll { let _ = segment_to_path(seg, &mut pb); }
                    pixmap.fill_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, fill, trfm)?, fillrule, trfm, None);
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    coll.iter().for_each(|line| {
                        pb.move_to(line.start.x, line.start.y);
                        pb.line_to(line.  end.x, line.  end.y);
                    }); stroke.width = *lwidth;

                    pixmap.stroke_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, line, trfm)?, &stroke, trfm, None);
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));

                    if !*strip { pb.close(); }  stroke.width = *lwidth;
                    pixmap.stroke_path(&pb.finish().ok_or(err_msg)?,
                        &style_to_paint(self, line, trfm)?, &stroke, trfm, None);
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, line, trfm)?;
                    stroke.width = *lwidth;

                    for seg in coll {
                        stroke_segment_path(seg, &mut pixmap, &paint, &mut stroke, trfm)?; }
                }
                Command::OutlinePolyg(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { pb.move_to(pt.x, pt.y) }
                    iter.for_each(|pt| pb.line_to(pt.x, pt.y));     pb.close();
                    let path = pb.finish().ok_or(err_msg)?;     stroke.width = *lwidth;

                    pixmap.  fill_path(&path,
                        &style_to_paint(self, fill, trfm)?, fillrule, trfm, None);
                    pixmap.stroke_path(&path,
                        &style_to_paint(self, line, trfm)?,  &stroke, trfm, None);
                }
                Command::OutlineRects(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill, trfm)?;
                    let pline = style_to_paint(self, line, trfm)?;
                    stroke.width = *lwidth;

                    coll.iter().for_each(|rect| pb.push_rect((*rect).into()));
                    let path = pb.finish().ok_or(err_msg)?;

                    pixmap.  fill_path(&path, &paint, fillrule, trfm, None);
                    pixmap.stroke_path(&path, &pline,  &stroke, trfm, None);
                }
                Command::OutlinePath (fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill, trfm)?;
                    let pline = style_to_paint(self, line, trfm)?;

                    stroke.width = *lwidth;     let mut res = false;
                    for seg in coll { res = segment_to_path(seg, &mut pb); }
                    let path = pb.finish().ok_or(err_msg)?;
                    pixmap.fill_path(&path, &paint, fillrule, trfm, None);

                    if res { for seg in coll {
                        stroke_segment_path(seg, &mut pixmap, &pline, &mut stroke, trfm)?;
                    } } else { pixmap.stroke_path(&path, &pline, &stroke, trfm, None); }
                }
            }
        }   Ok(pixmap)
    }   // rasterize
}

fn stroke_segment_path(seg: &Segment, pixmap: &mut skia::Pixmap, paint: &skia::Paint,
    stroke: &mut skia::Stroke, trfm: skia::Transform) -> Result<(), &'static str> {
    let mut pb = skia::PathBuilder::new();
    pb.move_to(seg.start.x, seg.start.y);

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if 1 < pb.len() {   let err_msg = "no start";
                let start = pb.last_point().ok_or(err_msg)?;
                pixmap.stroke_path(&pb.finish().ok_or(err_msg)?, paint, stroke, trfm, None);
                pb = skia::PathBuilder::new();  pb.move_to(start.x, start.y);
            }   stroke.width = width;
        }   process_segcmd(&mut pb, &cmd.instr);
    }

    pixmap.stroke_path(&pb.finish().ok_or("Fail build path from segments")?,
        paint, stroke, trfm, None);     Ok(())
}

fn segment_to_path(seg: &Segment, pb: &mut skia::PathBuilder) -> bool {
    pb.move_to(seg.start.x, seg.start.y);
    let mut change_lw = false;

    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(pb, &cmd.instr);
    }   change_lw
}

fn process_segcmd(pb: &mut skia::PathBuilder, cmd: &SegInstr) {
    match cmd {     SegInstr::ClosePath => pb.close(),
        SegInstr::Line  { end } => pb.line_to(end.x, end.y),
        SegInstr::HLine { x }     => pb.line_to(*x, pb.last_point().unwrap().y),
        SegInstr::VLine { y }     => pb.line_to(pb.last_point().unwrap().x, *y),

        SegInstr::CubicBezier { ctrl, end } =>
            pb.cubic_to(ctrl.0.x, ctrl.0.y, ctrl.1.x, ctrl.1.y, end.x, end.y),
        SegInstr::ArcCircle  { large, sweep, radius, target } =>
            pb.arc_to(&(*radius, *radius), 0.0, *large, *sweep, target),

        SegInstr::ArcEllipse { large, sweep, radius,
                rotation, target } => pb.arc_to(radius,
                *rotation, *large, *sweep, target),

        SegInstr::QuadBezier { ctrl, end } =>
            pb.quad_to(ctrl.x, ctrl.y, end.x, end.y),
    }
}

fn style_to_paint<'a, R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    style: &Style, trfm: skia::Transform) ->
    Result<skia::Paint<'a>, &'static str> {
    impl From<RGBA8888> for skia::Color {  // XXX: why not use ColorU8 defaultly in skia?
        fn from(c: RGBA8888) -> Self { Self::from_rgba8(c.r, c.g, c.b, c.a) }
    }

    impl From<Point> for skia::Point {
        //fn from(pt: Point) -> Self { (pt.x, pt.y).into() }
        fn from(pt: Point) -> Self { Self { x: pt.x, y: pt.y } }
        //fn from(pt: Point) -> Self { unsafe { std::mem::transmute(pt) } }
    }

    let mut paint = skia::Paint::default(); // default BlendMode::SourceOver
    match style {   // paint.anti_alias is default true
        Style::FlatColor(idx) => paint.set_color(img.lookup_color(*idx).into()),

        Style::LinearGradient { points, cindex } => {
            paint.shader = skia::LinearGradient::new(points.0.into(), points.1.into(),
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, trfm)
                .ok_or("Fail to create linear gradient shader")?;  //paint.anti_alias = false;
        }
        Style::RadialGradient { points, cindex } => {
            let (dx, dy) = (points.1.x - points.0.x, points.1.y - points.0.y);

            paint.shader = skia::RadialGradient::new(points.0.into(), points.0.into(),
                    (dx * dx + dy * dy).sqrt(),
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, trfm)
                .ok_or("Fail to create radial gradient shader")?;  //paint.anti_alias = false;
        }
    }   Ok(paint)
}

trait PathBuilderExt {
    fn arc_to(&mut self, radius: &(f32, f32), rotation: f32,
        large: bool, sweep: bool, target: &Point);
}   // https://github.com/RazrFalcon/resvg/blob/master/crates/usvg/src/parser/shapes.rs#L287

//  SVG arc to Canvas arc: https://github.com/nical/lyon/blob/main/crates/geom/src/arc.rs
impl PathBuilderExt for skia::PathBuilder {
    fn arc_to(&mut self, radius: &(f32, f32), rotation: f32,
        large: bool, sweep: bool, target: &Point) {
        let prev = self.last_point().unwrap();

        let svg_arc = kurbo::SvgArc {
             from: kurbo::Point::new(prev.x as _, prev.y as _),
               to: kurbo::Point::new(target.x as _, target.y as _),
            radii: kurbo::Vec2 ::new(radius.0 as _, radius.1 as _),
            x_rotation: (rotation as f64).to_radians(), large_arc: large, sweep,
        };

        match kurbo::Arc::from_svg_arc(&svg_arc) {  None => self.line_to(target.x, target.y),
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

