/****************************************************************
 * $ID: render_b2d.rs  	Tue 31 Oct 2023 14:16:48+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

use crate::tinyvg::*;
use crate::blend2d::*;
use std::{io, result::Result};

pub trait Render { fn render(&self, scale: f32) -> Result<BLImage, &str>; }

impl<R: io::Read, W: io::Write> Render for TinyVG<R, W> {
    fn render(&self, scale: f32) -> Result<BLImage, &str> {
        let mut img = BLImage::new(
            (self.header.width  as f32 * scale).ceil() as _,
            (self.header.height as f32 * scale).ceil() as _, BLFormat::BL_FORMAT_PRGB32);

        impl From<&Rect> for BLRect {   // BLBox
            //fn from(rect: &Rect) -> Self { unsafe { std::mem::transmute(rect) } }
            fn from(r: &Rect) -> Self {
                //Self { x0: r.l as _, y0: r.t as _, x1: r.r as _, y1: r.b as _ }
                Self { x: r.x as _, y: r.y as _, w: r.w as _, h: r.h as _ }
            }
        }

        // XXX: rendering up-scale and then scale down for anti-aliasing?
        let (mut ctx, mut path) = (BLContext::new(&mut img), BLPath::new());
        ctx.setStrokeJoin(BLStrokeJoin::BL_STROKE_JOIN_ROUND);
        ctx.setStrokeCaps(BLStrokeCap ::BL_STROKE_CAP_ROUND);
        ctx.setStrokeMiterLimit(4.0);   ctx.scale(scale, scale);
        // XXX: does path needs to be transformed before fill/stroke?

        for cmd in &self.commands {
            match cmd { Command::EndOfDocument => (),
                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.moveTo(&(*pt).into()) }
                    iter.for_each(|pt| path.lineTo(&(*pt).into()));  path.close();
                    ctx.fillGeometryExt(&path, convert_style(self, fill).as_ref());
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    let style = convert_style(self, fill);
                    coll.iter().for_each(|rect| path.addRect(&rect.into()));
                    ctx.fillGeometryExt(&path, style.as_ref());  //path.reset();
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    let style = convert_style(self, fill);
                    for seg in coll { let _ = segment_to_path(seg, &mut path); }
                    ctx.fillGeometryExt(&path, style.as_ref());  //path.reset();
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    coll.iter().for_each(|line| {
                        path.moveTo(&line.start.into()); path.lineTo(&line.  end.into());
                    }); ctx.setStrokeWidth(*lwidth);
                    ctx.strokeGeometryExt(&path, convert_style(self, line).as_ref());
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.moveTo(&(*pt).into()) }
                    iter.for_each(|pt| path.lineTo(&(*pt).into()));

                    if !*strip { path.close(); }    ctx.setStrokeWidth(*lwidth);
                    ctx.strokeGeometryExt(&path, convert_style(self, line).as_ref());
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let style = convert_style(self, line);
                    ctx.setStrokeWidth(*lwidth);

                    for seg in coll {
                        stroke_segment_path(seg, &mut ctx, style.as_ref()); }
                }
                Command::OutlinePolyg(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.moveTo(&(*pt).into()) }
                    iter.for_each(|pt| path.lineTo(&(*pt).into()));  path.close();

                    ctx.setStrokeWidth(*lwidth);
                    ctx.  fillGeometryExt(&path, convert_style(self, fill).as_ref());
                    ctx.strokeGeometryExt(&path, convert_style(self, line).as_ref());
                }
                Command::OutlineRects(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = convert_style(self, fill);
                    let pline = convert_style(self, line);
                    ctx.setStrokeWidth(*lwidth);

                    coll.iter().for_each(|rect| path.addRect(&rect.into()));
                    ctx.  fillGeometryExt(&path, paint.as_ref());
                    ctx.strokeGeometryExt(&path, pline.as_ref());    //path.reset();
                }
                Command::OutlinePath (fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = convert_style(self, fill);
                    let pline = convert_style(self, line);

                    ctx.setStrokeWidth(*lwidth);    let mut res = false;
                    for seg in coll { res = segment_to_path(seg, &mut path); }
                    ctx.  fillGeometryExt(&path, paint.as_ref());

                    if res { for seg in coll {
                        stroke_segment_path(seg, &mut ctx, pline.as_ref());
                    } } else { ctx.strokeGeometryExt(&path, pline.as_ref()); }  //path.reset();
                }
            }   path.reset();
        }   Ok(img)
    }
}

fn stroke_segment_path(seg: &Segment, ctx: &mut BLContext, style: &dyn B2DStyle) {
    let mut path = BLPath::new();
    path.moveTo(&seg.start.into());

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if 1 < path.getSize() {
                let start = path.getLastVertex().unwrap();
                ctx.strokeGeometryExt(&path, style);
                path.reset(); path.moveTo(&start);
            }   ctx.setStrokeWidth(width);
        }   process_segcmd(&mut path, &cmd.instr);
    }   ctx.strokeGeometryExt(&path, style);
}

fn segment_to_path(seg: &Segment, path: &mut BLPath) -> bool {
    path.moveTo(&seg.start.into());
    let mut change_lw = false;

    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(path, &cmd.instr);
    }   change_lw
}

fn process_segcmd(path: &mut BLPath, cmd: &SegInstr) {
    match cmd {     SegInstr::ClosePath => path.close(),
        SegInstr::Line  { end } => path.lineTo(&(*end).into()),
        SegInstr::HLine { x }     =>
            path.lineTo(&BLPoint { x: *x as _, y: path.getLastVertex().unwrap().y }),
        SegInstr::VLine { y }     =>
            path.lineTo(&BLPoint { x: path.getLastVertex().unwrap().x, y: *y as _ }),

        SegInstr::CubicBezier { ctrl, end } =>
            path.cubicTo(&ctrl.0.into(), &ctrl.1.into(), &(*end).into()),
        SegInstr::ArcCircle  { large, sweep, radius, target } =>
            path.ellipticArcTo(&BLPoint { x: *radius as _, y: *radius as _ },
                0.0, *large, *sweep, &(*target).into()),

        SegInstr::ArcEllipse { large, sweep, radius,
            rotation, target } => path.ellipticArcTo(&(*radius).into(),
                *rotation, *large, *sweep, &(*target).into()),

        SegInstr::QuadBezier { ctrl, end } =>
            path.quadTo(&(*ctrl).into(), &(*end).into()),
    }
}

fn convert_style<R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    style: &Style) -> Box<dyn B2DStyle> {
    impl From<RGBA8888> for BLRgba32 {
        fn from(color: RGBA8888) -> Self { Self { value: // convert to 0xAARRGGBB
            (color.a as u32) << 24 | (color.r as u32) << 16 |
            (color.g as u32)  << 8 |  color.b as u32
        } }
    }
    impl From<RGBA8888> for BLRgba64 {
        fn from(color: RGBA8888) -> Self { Self { value:
            ((color.a as u64) << 48 | (color.r as u64) << 32 |
             (color.g as u64) << 16 |  color.b as u64) * 0x0101
        } }
    }

    match style {
        Style::FlatColor(idx) => Box::new(BLVar::initRgba32(img.lookup_color(*idx).into())),

        Style::LinearGradient { points, cindex } => {
            let mut linear = BLGradient::new(
                &BLLinearGradientValues::new(&points.0.into(), &points.1.into()));
            linear.addStop(0.0, img.lookup_color(cindex.0).into());
            linear.addStop(1.0, img.lookup_color(cindex.1).into());
            Box::new(linear)    //linear.scale(scale, scale);
        }
        Style::RadialGradient { points, cindex } => {
            let (dx, dy) = (points.1.x - points.0.x, points.1.y - points.0.y);
            let radius = if dx.abs() < f32::EPSILON { dy.abs() }
                         else if dy.abs() < f32::EPSILON { dx.abs() }
                         else { (dx * dx + dy * dy).sqrt() };

            let mut radial = BLGradient::new(
                &BLRadialGradientValues::new(&points.0.into(), &points.1.into(), radius));
            radial.addStop(0.0, img.lookup_color(cindex.0).into());
            radial.addStop(1.0, img.lookup_color(cindex.1).into());
            Box::new(radial)    //radial.scale(scale, scale);
        }
    }
}

impl From<Point> for BLPoint {
    fn from(pt: Point) -> Self { Self { x: pt.x as _, y: pt.y as _ } }
    //fn from(pt: Point) -> Self { unsafe { std::mem::transmute(pt) } }
}

