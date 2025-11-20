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

        #[allow(non_local_definitions)] impl From<&Rect> for BLRect {   // BLBox
            //fn from(rect: &Rect) -> Self { unsafe { std::mem::transmute(rect) } }
            fn from(r: &Rect) -> Self {
                //Self { x0: r.l as _, y0: r.t as _, x1: r.r as _, y1: r.b as _ }
                Self { x: r.x as _, y: r.y as _, w: r.w as _, h: r.h as _ }
            }
        }

        // XXX: rendering up-scale and then scale down for anti-aliasing?
        let (mut ctx, mut path) = (BLContext::new(&mut img), BLPath::new());
        ctx.set_stroke_join(BLStrokeJoin::BL_STROKE_JOIN_ROUND);
        ctx.set_stroke_caps(BLStrokeCap::BL_STROKE_CAP_ROUND);
        ctx.set_stroke_miter_limit(4.0);
        ctx.scale((scale as _, scale as _));
        // XXX: does path needs to be transformed before fill/stroke?

        for cmd in &self.commands {
            match cmd { Command::EndOfDocument => (),
                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(&pt) = iter.next() { path.move_to(pt.into()) }
                    iter.for_each(|&pt| path.line_to(pt.into()));  path.close();
                    ctx.fill_geometry_ext(&path, convert_style(self, fill).as_ref());
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    let style = convert_style(self, fill);
                    coll.iter().for_each(|rect| path.add_rect(&rect.into()));
                    ctx.fill_geometry_ext(&path, style.as_ref());  //path.reset();
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    let style = convert_style(self, fill);
                    for seg in coll { let _ = segment_to_path(seg, &mut path); }
                    ctx.fill_geometry_ext(&path, style.as_ref());  //path.reset();
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    coll.iter().for_each(|line| {
                        path.move_to(line.start.into()); path.line_to(line.end.into());
                    }); ctx.set_stroke_width(*lwidth as _);
                    ctx.stroke_geometry_ext(&path, convert_style(self, line).as_ref());
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    if let Some(&pt) = iter.next() { path.move_to(pt.into()) }
                    iter.for_each(|&pt| path.line_to(pt.into()));

                    if !*strip { path.close(); }    ctx.set_stroke_width(*lwidth as _);
                    ctx.stroke_geometry_ext(&path, convert_style(self, line).as_ref());
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let style = convert_style(self, line);
                    ctx.set_stroke_width(*lwidth as _);

                    for seg in coll {
                        stroke_segment_path(seg, &mut ctx, style.as_ref()); }
                }
                Command::OutlinePolyg(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(&pt) = iter.next() { path.move_to(pt.into()) }
                    iter.for_each(|&pt| path.line_to(pt.into()));  path.close();

                    ctx.set_stroke_width(*lwidth as _);
                    ctx.  fill_geometry_ext(&path, convert_style(self, fill).as_ref());
                    ctx.stroke_geometry_ext(&path, convert_style(self, line).as_ref());
                }
                Command::OutlineRects(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = convert_style(self, fill);
                    let pline = convert_style(self, line);
                    ctx.set_stroke_width(*lwidth as _);

                    coll.iter().for_each(|rect| path.add_rect(&rect.into()));
                    ctx.  fill_geometry_ext(&path, paint.as_ref());
                    ctx.stroke_geometry_ext(&path, pline.as_ref());    //path.reset();
                }
                Command::OutlinePath (fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = convert_style(self, fill);
                    let pline = convert_style(self, line);

                    ctx.set_stroke_width(*lwidth as _);     let mut res = false;
                    for seg in coll { res = segment_to_path(seg, &mut path); }
                    ctx.  fill_geometry_ext(&path, paint.as_ref());

                    if res { for seg in coll {
                        stroke_segment_path(seg,  &mut ctx, pline.as_ref());
                    } } else { ctx.stroke_geometry_ext(&path, pline.as_ref()); }  //path.reset();
                }
            }   path.reset();
        }   Ok(img)
    }
}

fn stroke_segment_path(seg: &Segment, ctx: &mut BLContext, style: &dyn B2DStyle) {
    let mut path = BLPath::new();
    path.move_to(seg.start.into());

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if 1 < path.get_size() {
                let start = path.get_last_vertex().unwrap();
                ctx.stroke_geometry_ext(&path, style);
                path.reset(); path.move_to(start);
            }   ctx.set_stroke_width(width as _);
        }   process_segcmd(&mut path, &cmd.instr);
    }   ctx.stroke_geometry_ext(&path, style);
}

fn segment_to_path(seg: &Segment, path: &mut BLPath) -> bool {
    path.move_to(seg.start.into());
    let mut change_lw = false;

    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(path, &cmd.instr);
    }   change_lw
}

fn process_segcmd(path: &mut BLPath, cmd: &SegInstr) {
    match cmd {     SegInstr::ClosePath => path.close(),
        SegInstr::Line  { end } => path.line_to((*end).into()),
        SegInstr::HLine { x }     =>
            path.line_to((*x as _, path.get_last_vertex().unwrap().y).into()),
        SegInstr::VLine { y }     =>
            path.line_to((path.get_last_vertex().unwrap().x, *y as _).into()),

        SegInstr::CubicBezier { ctrl, end } =>
            path.cubic_to(ctrl.0.into(), ctrl.1.into(), (*end).into()),
        SegInstr::ArcCircle  { large, sweep, radius, end } =>
            path.elliptic_arc_to((*radius as _, *radius as _),
                0.0, *large, *sweep, (*end).into()),

        SegInstr::ArcEllipse { large, sweep, radii,
            rotation, end } => path.elliptic_arc_to((radii.0 as _, radii.1 as _),
                *rotation as _, *large, *sweep, (*end).into()),

        SegInstr::QuadBezier { ctrl, end } =>
            path.quad_to((*ctrl).into(), (*end).into()),
    }
}

fn convert_style<R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    style: &Style) -> Box<dyn B2DStyle> {
    #[allow(non_local_definitions)] impl From<RGBA8888> for BLRgba32 {
        fn from(color: RGBA8888) -> Self { Self { value: // convert to 0xAARRGGBB
            (color.a as u32) << 24 | (color.r as u32) << 16 |
            (color.g as u32)  << 8 |  color.b as u32
        } }
    }

    match style {
        Style::FlatColor(idx) =>
            Box::new(BLSolidColor::init_rgba32(img.lookup_color(*idx).into())),

        Style::LinearGradient { points, cindex } => {
            let mut linear = BLGradient::new(
                &BLLinearGradientValues::new(points.0.into(), points.1.into()));
            linear.add_stop(0.0, img.lookup_color(cindex.0).into());
            linear.add_stop(1.0, img.lookup_color(cindex.1).into());
            Box::new(linear)    //linear.scale(scale, scale);
        }
        Style::RadialGradient { points, cindex } => {
            let radius = (points.1.x - points.0.x).hypot(points.1.y - points.0.y);
            let mut radial = BLGradient::new(&BLRadialGradientValues::new(
                points.0.into(), points.1.into(), (0., radius as _)));
            radial.add_stop(0.0, img.lookup_color(cindex.0).into());
            radial.add_stop(1.0, img.lookup_color(cindex.1).into());
            Box::new(radial)    //radial.scale(scale, scale);
        }
    }
}

impl From<Point> for BLPoint {
    fn from(pt: Point) -> Self { Self { x: pt.x as _, y: pt.y as _ } }
    //fn from(pt: Point) -> Self { unsafe { std::mem::transmute(pt) } }
}

