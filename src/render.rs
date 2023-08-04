
use crate::tinyvg::*;
use tiny_skia as skia;
use std::result::Result;

pub trait Render { fn render(&self) -> Result<skia::Pixmap, &str>; }

impl Render for TVGImage {
    fn render(&self) -> Result<skia::Pixmap, &str> {
        let mut pixmap = skia::Pixmap::new(self.header.width,
            self.header.height).ok_or("Fail to create pixmap")?;
        let (ts, err_str) = (skia::Transform::identity(), "Fail to build path");

        impl From<Rect> for skia::Rect {
            fn from(r: Rect) -> Self { unsafe { std::mem::transmute(r) }}
            //fn from(r: Rect) -> Self { skia::Rect::from_ltrb(r.l, r.t, r.r, r.b).unwrap() }
            //fn from(r: Rect) -> Self { skia::Rect::from_xywh(r.x, r.y, r.w, r.h).unwrap() }
        }

        for cmd in &self.commands {
            match cmd {     Command::EndOfDocument => (),
                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    let mut pb = skia::PathBuilder::new();
                    iter.next().map(|point| pb.move_to(point.x, point.y));
                    iter.for_each  (|point| pb.line_to(point.x, point.y));  pb.close();

                    pixmap.fill_path(&pb.finish().ok_or(err_str)?,
                        &style_to_paint(self, fill)?, skia::FillRule::Winding, ts, None);
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    let paint = style_to_paint(self, fill)?;
                    coll.iter().for_each(|rect| pixmap.fill_rect((*rect).into(),
                        &paint, ts, None));
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    let paint = style_to_paint(self, fill)?;
                    for seg in coll {   let res = segment_to_path(seg)?;
                        if res.1 { return Err("Got line width in fill path segment") }
                        pixmap.fill_path(&res.0, &paint, skia::FillRule::Winding, ts, None);
                    }
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    let mut pb = skia::PathBuilder::new();
                    coll.iter().for_each(|line| {
                        pb.move_to(line.start.x, line.start.y);
                        pb.line_to(line.  end.x, line.  end.y); });

                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;
                    pixmap.stroke_path(&pb.finish().ok_or(err_str)?,
                        &style_to_paint(self, line)?, &stroke, ts, None);
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    let mut pb = skia::PathBuilder::new();
                    iter.next().map(|point| pb.move_to(point.x, point.y));
                    iter.for_each  (|point| pb.line_to(point.x, point.y));
                    if !*strip { pb.close(); }

                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;
                    pixmap.stroke_path(&pb.finish().ok_or(err_str)?,
                        &style_to_paint(self, line)?, &stroke, ts, None);
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_paint(self, line)?;
                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;
                    for seg in coll {
                        stroke_segment_path(seg, &mut pixmap, &paint, &mut stroke)?;
                    }
                }
                Command::OutlinePolyg(OutlineCMD {
                    fill, line, lwidth, coll }) => {
                    let mut iter = coll.iter();
                    let mut pb = skia::PathBuilder::new();
                    iter.next().map(|point| pb.move_to(point.x, point.y));
                    iter.for_each  (|point| pb.line_to(point.x, point.y));  pb.close();

                    let path = pb.finish().ok_or(err_str)?;
                    pixmap.fill_path(&path, &style_to_paint(self, fill)?,
                        skia::FillRule::Winding, ts, None);

                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;
                    pixmap.stroke_path(&path, &style_to_paint(self, line)?, &stroke, ts, None);
                }
                Command::OutlineRects(OutlineCMD {
                    fill, line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill)?;
                    let pline = style_to_paint(self, line)?;
                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;

                    coll.iter().for_each(|rect| {
                        let path = skia::PathBuilder::from_rect((*rect).into());
                        pixmap.fill_path(&path, &paint, skia::FillRule::Winding, ts, None);
                        pixmap.stroke_path(&path, &pline, &stroke, ts, None);
                    });
                }
                Command::OutlinePath (OutlineCMD {
                    fill, line, lwidth, coll }) => {
                    let paint = style_to_paint(self, fill)?;
                    let pline = style_to_paint(self, line)?;
                    let mut stroke = skia::Stroke::default();
                    stroke.line_cap = skia::LineCap::Round;     stroke.width = *lwidth;

                    for seg in coll {   let res = segment_to_path(seg)?;
                        pixmap.fill_path(&res.0, &paint, skia::FillRule::Winding, ts, None);
                        if !res.1 { pixmap.stroke_path(&res.0, &pline, &stroke, ts, None);
                        } else { stroke_segment_path(seg, &mut pixmap, &pline, &mut stroke)?; }
                    }
                }
            }
        }   Ok(pixmap)
    }   // rasterize
}

fn stroke_segment_path(seg: &Segment, pixmap: &mut skia::Pixmap,
    paint: &skia::Paint, stroke: &mut skia::Stroke) -> Result<(), &'static str> {
    let mut start: skia::Point = (seg.start.x, seg.start.y).into();

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth { stroke.width = width }
        let mut pb = skia::PathBuilder::new();
        pb.move_to(start.x, start.y);

        match &cmd.instr {
            SegInstr::Line  { end } => pb.line_to(end.x, end.y),
            SegInstr::HLine { x }     => pb.line_to(*x, start.y),
            SegInstr::VLine { y }     => pb.line_to(start.x, *y),

            SegInstr::CubicBezier { ctrl, end } =>
                pb.cubic_to(ctrl.0.x, ctrl.0.y, ctrl.1.x, ctrl.1.y, end.x, end.y),
            SegInstr::ArcCircle  { large, sweep, radius, target } =>
                pb.arc_to(*radius, *radius, 0.0, *large, *sweep, target.x, target.y),

            SegInstr::ArcEllipse { large, sweep, radius,
                    rotation, target } => pb.arc_to(radius.0, radius.1,
                   *rotation, *large, *sweep, target.x, target.y),

            SegInstr::ClosePath => pb.close(),
            SegInstr::QuadBezier { ctrl, end } =>
                pb.quad_to(ctrl.x, ctrl.y, end.x, end.y),
        }   start = pb.last_point().ok_or("no last point")?;

        pixmap.stroke_path(&pb.finish().ok_or("Fail to get path from a segment")?,
            &paint, &stroke, skia::Transform::identity(), None);
    }   Ok(())
}

fn segment_to_path(seg: &Segment) -> Result<(skia::Path, bool), &'static str> {
    let mut pb = skia::PathBuilder::new();
    pb.move_to(seg.start.x, seg.start.y);

    let mut change_lw = false;
    for cmd in &seg.cmds {
        if let Some(_) = cmd.lwidth { change_lw = true }

        match &cmd.instr {
            SegInstr::Line  { end } => pb.line_to(end.x, end.y),
            SegInstr::HLine { x }     => pb.line_to(*x, pb.last_point().ok_or("")?.y),
            SegInstr::VLine { y }     => pb.line_to(pb.last_point().ok_or("")?.x, *y),

            SegInstr::CubicBezier { ctrl, end } =>
                pb.cubic_to(ctrl.0.x, ctrl.0.y, ctrl.1.x, ctrl.1.y, end.x, end.y),
            SegInstr::ArcCircle  { large, sweep, radius, target } =>
                pb.arc_to(*radius, *radius, 0.0, *large, *sweep, target.x, target.y),

            SegInstr::ArcEllipse { large, sweep, radius,
                    rotation, target } => pb.arc_to(radius.0, radius.1,
                   *rotation, *large, *sweep, target.x, target.y),

            SegInstr::ClosePath => pb.close(),
            SegInstr::QuadBezier { ctrl, end } =>
                pb.quad_to(ctrl.x, ctrl.y, end.x, end.y),
        }
    }   Ok((pb.finish().ok_or("Fail to build path from segments")?, change_lw))
}

fn style_to_paint<'a>(img: &TVGImage, style: &Style) -> Result<skia::Paint<'a>, &'static str> {
    impl From<RGBA8888> for skia::Color {  // XXX: why not use ColorU8 defaultly in skia?
        fn from(c: RGBA8888) -> Self { Self::from_rgba8(c.r, c.g, c.b, c.a) }
    }

    impl From<Point> for skia::Point {
        //fn from(pt: Point) -> Self { (pt.x, pt.y).into() }
        //fn from(pt: Point) -> Self { Self { x: pt.x, y: pt.y } }
        fn from(pt: Point) -> Self { unsafe { std::mem::transmute(pt) } }
    }

    let mut paint = skia::Paint::default();
    match style {   // paint.anti_alias is default true
        Style::FlatColor(idx) => paint.set_color(img.lookup_color(*idx).into()),

        Style::LinearGradient { points, cindex } => {
            paint.shader = skia::LinearGradient::new(points.0.into(), points.1.into(),
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, skia::Transform::identity(),
            ).ok_or("Fail to create linear gradient shader")?;  paint.anti_alias = false;
        }
        Style::RadialGradient { points, cindex } => {
            let (dx, dy) = (points.1.x - points.0.x, points.1.y - points.0.y);
            let radius = if dx.abs() < f32::EPSILON { dy.abs() }
                         else if dy.abs() < f32::EPSILON { dx.abs() }
                         else { (dx.powi(2) + dy.powi(2)).sqrt() };

            paint.shader = skia::RadialGradient::new(points.0.into(), points.1.into(), radius,
                vec![ skia::GradientStop::new(0.0, img.lookup_color(cindex.0).into()),
                      skia::GradientStop::new(1.0, img.lookup_color(cindex.1).into()),
                ],    skia::SpreadMode::Pad, skia::Transform::identity(),
            ).ok_or("Fail to create radial gradient shader")?;  paint.anti_alias = false;
        }
    }   Ok(paint)
}

trait PathBuilderExt {
    fn arc_to(&mut self, rx: f32, ry: f32, rotation: f32,
        large: bool, sweep: bool, x: f32, y: f32);
}

impl PathBuilderExt for skia::PathBuilder {
    fn arc_to(&mut self, rx: f32, ry: f32, rotation: f32,
        large: bool, sweep: bool, x: f32, y: f32) {
        let prev = match self.last_point() { Some(v) => v, None => return };

        let svg_arc = kurbo::SvgArc {
            from: kurbo::Point::new(prev.x as f64, prev.y as f64),
              to: kurbo::Point::new( x as f64,  y as f64),
            radii: kurbo::Vec2::new(rx as f64, ry as f64),
            x_rotation: (rotation as f64).to_radians(), large_arc: large, sweep,
        };

        match kurbo::Arc::from_svg_arc(&svg_arc) {  None => self.line_to(x, y),
            Some(arc) => arc.to_cubic_beziers(0.1, |p1, p2, p|
                    self.cubic_to(p1.x as f32, p1.y as f32,
                                  p2.x as f32, p2.y as f32, p.x as f32, p.y as f32)),
        }
    }
}

