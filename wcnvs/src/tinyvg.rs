/****************************************************************
 * $ID: tinyvg.rs   	Sat 27 Apr 2024 07:47:47+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2024 M.H.Fan, All rights reserved.             *
 ****************************************************************/

use {std::io, intvg::tinyvg::*};
use web_sys::{CanvasRenderingContext2d as Contex2d, Path2d};

pub fn render<R: io::Read, W: io::Write>(tvg: &TinyVG<R, W>,
    scale: f32, ctx2d: &Contex2d) -> Result<(), &'static str> {
    ctx2d.set_line_join("round");   ctx2d.scale(scale as _, scale as _).unwrap();
    ctx2d.set_line_cap ("round");   ctx2d.set_miter_limit(4.0);
    //tracing::info!("scaling factor: {scale}");

    for cmd in &tvg.commands {
        let mut path = Path2d::new().unwrap();
        match cmd { Command::EndOfDocument => (),

            Command::FillPolyg(FillCMD { fill, coll }) => {
                let mut iter = coll.iter();
                if let Some(pt) = iter.next() { path.move_to(pt.x as _, pt.y as _) }
                iter.for_each(|pt|   path.line_to(pt.x as _, pt.y as _));
                ctx2d.set_fill_style(&convert_style(tvg, ctx2d, fill));     path.close_path();
                ctx2d.fill_with_path_2d(&path);
            }
            Command::FillRects(FillCMD { fill, coll }) => {
                ctx2d.set_fill_style(&convert_style(tvg, ctx2d, fill));
                coll.iter().for_each(|rect| {
                    path.rect(rect.x as _, rect.y as _, rect.w as _, rect.h as _);
                }); ctx2d.fill_with_path_2d(&path); //path = Path2d::new().unwrap();
            }
            Command::FillPath (FillCMD { fill, coll }) => {
                ctx2d.set_fill_style(&convert_style(tvg, ctx2d, fill));
                for seg in coll {   let _ = segment_to_path(seg, &path);
                    //if res { return Err("Got line width in fill path segment") }
                }   ctx2d.fill_with_path_2d(&path); //path = Path2d::new().unwrap();
            }
            Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                ctx2d.set_line_width(*lwidth as _);
                coll.iter().for_each(|line| {
                    path.move_to(line.start.x as _, line.start.y as _);
                    path.line_to(line.  end.x as _, line.  end.y as _);
                }); ctx2d.stroke_with_path(&path);
            }
            Command::DrawLoop (DrawCMD { line, lwidth, coll },
                strip) => {     let mut iter = coll.iter();
                if let Some(pt) = iter.next() { path.move_to(pt.x as _, pt.y as _) }
                iter.for_each(|pt| path.line_to(pt.x as _, pt.y as _));

                ctx2d.set_line_width(*lwidth as _);     if !*strip { path.close_path(); }
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                ctx2d.stroke_with_path(&path);
            }
            Command::DrawPath (DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_line_width(*lwidth as _);
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                for seg in coll { stroke_segment_path(seg, ctx2d); }
            }
            Command::OutlinePolyg(fill, DrawCMD {
                line, lwidth, coll }) => {
                let mut iter = coll.iter();
                if let Some(pt) = iter.next() { path.move_to(pt.x as _, pt.y as _) }
                iter.for_each(|pt| path.line_to(pt.x as _, pt.y as _));

                ctx2d.set_line_width(*lwidth as _);     path.close_path();
                ctx2d.set_fill_style  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                ctx2d.fill_with_path_2d(&path);
                ctx2d.stroke_with_path (&path);
            }
            Command::OutlineRects(fill, DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_fill_style  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                ctx2d.set_line_width(*lwidth as _);

                coll.iter().for_each(|rect| {
                    path.rect(rect.x as _, rect.y as _, rect.w as _, rect.h as _);
                }); ctx2d.fill_with_path_2d(&path);
                    ctx2d.stroke_with_path (&path); //path = Path2d::new().unwrap();
            }
            Command::OutlinePath (fill, DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_fill_style  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style(&convert_style(tvg, ctx2d, line));
                ctx2d.set_line_width(*lwidth as _);

                for seg in coll {   let res = segment_to_path(seg, &path);
                    ctx2d.fill_with_path_2d(&path);

                    if res { stroke_segment_path(seg, ctx2d);
                    } else { ctx2d.stroke_with_path(&path); }
                    path = Path2d::new().unwrap();
                }
            }
        }
    }   Ok(())
}

fn stroke_segment_path(seg: &Segment, ctx2d: &Contex2d) {
    let mut path = Path2d::new().unwrap();
    path.move_to(seg.start.x as _, seg.start.y as _);

    let mut last_point = seg.start;
    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if true/* 1 < path.get_size()*/ { // XXX:
                ctx2d.stroke_with_path(&path);

                path = Path2d::new().unwrap();
                path.move_to(last_point.x as _, last_point.y as _);
            }   ctx2d.set_line_width(width as _);
        }   process_segcmd(&path, &cmd.instr, &mut last_point);
    }           ctx2d.stroke_with_path(&path);
}

fn segment_to_path(seg: &Segment, path: &Path2d) -> bool {
    path.move_to(seg.start.x as _, seg.start.y as _);
    let mut change_lw = false;

    let mut last_point = seg.start;
    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(path, &cmd.instr, &mut last_point);
    }   change_lw
}

fn process_segcmd(path: &Path2d, cmd: &SegInstr, last_point: &mut Point) {
    match cmd {     SegInstr::ClosePath => path.close_path(),
        SegInstr::Line  { end } => {
            path.line_to(end.x as _, end.y as _);      *last_point = *end;
        }
        SegInstr::HLine { x }     => {
            path.line_to(*x as _, last_point.y as _);   last_point.x = *x;
        }
        SegInstr::VLine { y }     => {
            path.line_to(last_point.x as _, *y as _);   last_point.y = *y;
        }

        SegInstr::CubicBezier { ctrl, end } => {
            path.bezier_curve_to(ctrl.0.x as _, ctrl.0.y as _, ctrl.1.x as _, ctrl.1.y as _,
                end.x as _, end.y as _);    *last_point = *end;
        }

        SegInstr::ArcCircle  { large, sweep, radius, target } => {
            wcns_arc_to(path, last_point, &(*radius, *radius), 0.0,
                *large, *sweep, target);    *last_point = *target;
        }

        SegInstr::ArcEllipse { large, sweep, radius,
            rotation, target } => {
            wcns_arc_to(path, last_point, radius, *rotation,
                *large, *sweep, target);    *last_point = *target;
        }

        SegInstr::QuadBezier { ctrl, end } => {
            path.quadratic_curve_to(ctrl.x as _, ctrl.y as _,
                end.x as _, end.y as _);    *last_point = *end;
        }
    }
}

fn wcns_arc_to(path: &Path2d, start: &Point, radius: &(f32, f32),
    rotation: f32, large: bool, sweep: bool, end: &Point) {
    let svg_arc = kurbo::SvgArc {   // lyon_geom: https://github.com/nical/lyon
           to: kurbo::Point::new(end.x as _, end.y as _),
         from: kurbo::Point::new(start.x as _, start.y as _),
        radii: kurbo::Vec2 ::new(radius.0 as _, radius.1 as _),
        x_rotation: (rotation as f64).to_radians(), large_arc: large, sweep,
    };

    match kurbo::Arc::from_svg_arc(&svg_arc) {  None => path.line_to(end.x as _, end.x as _),
        Some(arc) => arc.to_cubic_beziers(0.1, |p1, p2, p|
            path.bezier_curve_to(p1.x as _, p1.y as _, p2.x as _, p2.y as _, p.x as _, p.y as _)),
    }
}

fn convert_style<R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    ctx2d: &Contex2d, style: &Style) -> wasm_bindgen::JsValue {
    fn to_css_color<R: io::Read, W: io::Write>(img: &TinyVG<R, W>, idx: u32) -> String {
        let color = img.lookup_color(idx);
        let mut str = format!("#{:0<2x}{:0<2x}{:0<2x}", color.r, color.g, color.b);
        if color.a != 255 {    str.push_str(&format!("{:0<2x}", color.a)); }    str
    }

    match style {   Style::FlatColor(idx) => to_css_color(img, *idx).into(),

        Style::LinearGradient { points, cindex } => {
            let linear = ctx2d.create_linear_gradient(
                points.0.x as _, points.0.y as _, points.1.x as _, points.1.y as _);
            linear.add_color_stop(0.0, &to_css_color(img, cindex.0)).unwrap();
            linear.add_color_stop(1.0, &to_css_color(img, cindex.1)).unwrap();  linear.into()
        }   // don't need to scale, since created in context
        Style::RadialGradient { points, cindex } => {
            let (dx, dy) = (points.1.x - points.0.x, points.1.y - points.0.y);
            let radius = if dx.abs() < f32::EPSILON { dy.abs() }
                         else if dy.abs() < f32::EPSILON { dx.abs() }
                         else { (dx * dx + dy * dy).sqrt() };

            let radial = ctx2d.create_radial_gradient(
                points.0.x as _, points.0.y as _, 1.0,  // XXX:
                points.1.x as _, points.1.y as _, radius as _).unwrap();
            radial.add_color_stop(0.0, &to_css_color(img, cindex.0)).unwrap();
            radial.add_color_stop(1.0, &to_css_color(img, cindex.1)).unwrap();  radial.into()
        }
    }
}

