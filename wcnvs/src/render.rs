/****************************************************************
 * $ID: render.rs   	Sat 27 Apr 2024 07:47:47+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2024 M.H.Fan, All rights reserved.             *
 ****************************************************************/

use {std::io, intvg::tinyvg::*};
use web_sys::{CanvasRenderingContext2d as Context2d, Path2d};

pub fn render_svg(tree: &usvg::Tree, ctx2d: &Context2d, cw: u32, ch: u32) {
    let (tw, th) = (tree.size().width() as f64, tree.size().height() as f64);
    let scale = (cw as f64 / tw).min(ch as f64 / th);

    ctx2d.reset();  //ctx2d.clear_rect(0.0, 0.0, cw as _, ch as _);
    let _ = ctx2d.translate((cw as f64 - scale * tw) / 2., (ch as f64 - scale * th) / 2.);
    let _ = ctx2d.scale(scale, scale);  ctx2d.set_line_join("round");
    ctx2d.set_miter_limit(4.0);         ctx2d.set_line_cap ("round");

    render_nodes(ctx2d, tree.root(), &usvg::Transform::identity());
}

fn render_nodes(ctx2d: &Context2d, parent: &usvg::Group, trfm: &usvg::Transform) {
    for child in parent.children() { match child {
        usvg::Node::Group(group) =>     // trfm is needed on rendering only
            render_nodes(ctx2d, group, &trfm.pre_concat(group.transform())),
            // TODO: deal with group.clip_path()/mask()/filters()

        usvg::Node::Path(path) => if path.is_visible() {
            let tpath = if trfm.is_identity() { None
            } else { path.data().clone().transform(*trfm) };    // XXX:
            let fpath = Path2d::new().unwrap();

            for seg in tpath.as_ref().unwrap_or(path.data()).segments() {
                use usvg::tiny_skia_path::PathSegment;
                match seg {     PathSegment::Close => fpath.close_path(),
                    PathSegment::MoveTo(pt) => fpath.move_to(pt.x as _, pt.y as _),
                    PathSegment::LineTo(pt) => fpath.line_to(pt.x as _, pt.y as _),

                    PathSegment::QuadTo(ctrl, end) => fpath.quadratic_curve_to(
                        ctrl.x as _, ctrl.y as _, end.x as _, end.y as _),
                    PathSegment::CubicTo(ctrl0, ctrl1, end) =>
                        fpath.bezier_curve_to(ctrl0.x as _, ctrl0.y as _,
                            ctrl1.x as _, ctrl1.y as _, end.x as _, end.y as _),
                }
            }

            let fill = path.fill().map(|fill| {
                if let Some(style) = convert_paint(ctx2d, fill.paint(),
                      fill.opacity(), trfm) { ctx2d.set_fill_style_str(&style); }
                match fill.rule() {
                    usvg::FillRule::NonZero => web_sys::CanvasWindingRule::Nonzero,
                    usvg::FillRule::EvenOdd => web_sys::CanvasWindingRule::Evenodd,
                }
            });

            let stroke = path.stroke().map(|stroke| {
                if let Some(style) = convert_paint(ctx2d, stroke.paint(),
                    stroke.opacity(), trfm) { ctx2d.set_stroke_style_str(&style); }

                ctx2d.set_line_width (stroke.width().get() as _);
                ctx2d.set_miter_limit(stroke.miterlimit().get() as _);
                ctx2d.set_line_join(match stroke.linejoin() { usvg::LineJoin::MiterClip |
                    usvg::LineJoin::Miter => "miter",
                    usvg::LineJoin::Round => "round",
                    usvg::LineJoin::Bevel => "bevel",
                });
                ctx2d.set_line_cap (match stroke.linecap () {
                    usvg::LineCap::Butt   => "butt",
                    usvg::LineCap::Round  => "round",
                    usvg::LineCap::Square => "square",
                }); true
            });

            match path.paint_order() {
                usvg::PaintOrder::FillAndStroke => {
                    if let Some(fill) = fill {
                        ctx2d.fill_with_path_2d_and_winding(&fpath, fill); }
                    if stroke.is_some() { ctx2d.stroke_with_path(&fpath); }
                }       //ctx2d.fill_with_path_2d(&fpath);
                usvg::PaintOrder::StrokeAndFill => {
                    if stroke.is_some() { ctx2d.stroke_with_path(&fpath); }
                    if let Some(fill) = fill {
                        ctx2d.fill_with_path_2d_and_winding(&fpath, fill); }
                }
            }
        }

        // TODO: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Using_images
        usvg::Node::Image(img) => if img.is_visible() {
            match img.kind() {
                usvg::ImageKind::GIF(_) | usvg::ImageKind::WEBP(_) |
                usvg::ImageKind::PNG(_) | usvg::ImageKind::JPEG(_) => todo!(),
                // https://github.com/linebender/vello_svg/blob/main/src/lib.rs#L212
                usvg::ImageKind::SVG(svg) => render_nodes(ctx2d, svg.root(), trfm),
            }
        }

        usvg::Node::Text(text) => { let group = text.flattened();
            render_nodes(ctx2d, group, &trfm.pre_concat(group.transform()));
        }
    } }
}

fn convert_paint(ctx2d: &Context2d, paint: &usvg::Paint,
    opacity: usvg::Opacity, _trfm: &usvg::Transform) -> Option<String> {
    fn to_css_color(color: usvg::Color, opacity: usvg::Opacity) -> String {
        let mut str = format!("#{:0<2x}{:0<2x}{:0<2x}",
            color.red, color.green, color.blue);
        if opacity != 1.0 { str.push_str(&format!("{:0<2x}",
            (opacity.get() * 255.) as u8)); }   str
    }

    match paint { usvg::Paint::Pattern(_) => { // trfm should be applied here
            // TODO: https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvas
            // refer to render_pattern_pixmap@resvg/crates/resvg/src/path.rs
            eprintln!("Not support pattern painting"); None }
        usvg::Paint::Color(color) => Some(to_css_color(*color, opacity)),

        usvg::Paint::LinearGradient(grad) => {
            let linear = ctx2d.create_linear_gradient(
                grad.x1() as _, grad.y1() as _, grad.x2() as _, grad.y2() as _);

            grad.stops().iter().for_each(|stop| { let _ = linear.add_color_stop(0.0,
                &to_css_color(stop.color(), stop.opacity() * opacity));
            }); linear.as_string()
        }
        usvg::Paint::RadialGradient(grad) => {
            let radial = ctx2d.create_radial_gradient(
                grad.fx() as _, grad.fy() as _,     // XXX: 1./0.
               (grad.cx() - grad.fx()).hypot(grad.cy() - grad.fy()) as _,
                grad.cx() as _, grad.cy() as _, grad.r().get() as _).unwrap();

            grad.stops().iter().for_each(|stop| { let _ = radial.add_color_stop(0.0,
                &to_css_color(stop.color(), stop.opacity() * opacity));
            }); radial.as_string()
        }
    }
}

pub fn render_tvg<R: io::Read, W: io::Write>(tvg: &TinyVG<R, W>,
    ctx2d: &Context2d, cw: u32, ch: u32) {
    let (tw, th) = (tvg.header.width as f64, tvg.header.height as f64);
    let scale = (cw as f64 / tw).min(ch as f64 / th);

    ctx2d.reset();  //ctx2d.clear_rect(0.0, 0.0, cw as _, ch as _);
    let _ = ctx2d.translate((cw as f64 - scale * tw) / 2., (ch as f64 - scale * th) / 2.);
    let _ = ctx2d.scale(scale, scale);  ctx2d.set_line_join("round");
    ctx2d.set_miter_limit(4.0);         ctx2d.set_line_cap ("round");

    for cmd in &tvg.commands {
        let mut path = Path2d::new().unwrap();
        match cmd { Command::EndOfDocument => (),

            Command::FillPolyg(FillCMD { fill, coll }) => {
                let mut iter = coll.iter();
                if let Some(pt) = iter.next() { path.move_to(pt.x as _, pt.y as _) }
                iter.for_each(|pt|   path.line_to(pt.x as _, pt.y as _));
                ctx2d.set_fill_style_str(&convert_style(tvg, ctx2d, fill)); path.close_path();
                ctx2d.fill_with_path_2d(&path);
            }
            Command::FillRects(FillCMD { fill, coll }) => {
                ctx2d.set_fill_style_str(&convert_style(tvg, ctx2d, fill));
                coll.iter().for_each(|rect| {
                    path.rect(rect.x as _, rect.y as _, rect.w as _, rect.h as _);
                }); ctx2d.fill_with_path_2d(&path);
            }
            Command::FillPath (FillCMD { fill, coll }) => {
                ctx2d.set_fill_style_str(&convert_style(tvg, ctx2d, fill));
                for seg in coll {   let _ = segment_to_path(seg, &path);
                    //if res { return Err("Got line width in fill path segment") }
                }   ctx2d.fill_with_path_2d(&path);
            }
            Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
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
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
                ctx2d.stroke_with_path(&path);
            }
            Command::DrawPath (DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_line_width(*lwidth as _);
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
                for seg in coll { stroke_segment_path(seg, ctx2d); }
            }
            Command::OutlinePolyg(fill, DrawCMD {
                line, lwidth, coll }) => {
                let mut iter = coll.iter();
                if let Some(pt) = iter.next() { path.move_to(pt.x as _, pt.y as _) }
                iter.for_each(|pt| path.line_to(pt.x as _, pt.y as _));

                ctx2d.set_line_width(*lwidth as _);     path.close_path();
                ctx2d.set_fill_style_str  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
                ctx2d.fill_with_path_2d(&path);     ctx2d.stroke_with_path (&path);
            }
            Command::OutlineRects(fill, DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_fill_style_str  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
                ctx2d.set_line_width(*lwidth as _);

                coll.iter().for_each(|rect| {
                    path.rect(rect.x as _, rect.y as _, rect.w as _, rect.h as _);
                }); ctx2d.fill_with_path_2d(&path);
                    ctx2d.stroke_with_path (&path);
            }
            Command::OutlinePath (fill, DrawCMD {
                line, lwidth, coll }) => {
                ctx2d.set_fill_style_str  (&convert_style(tvg, ctx2d, fill));
                ctx2d.set_stroke_style_str(&convert_style(tvg, ctx2d, line));
                ctx2d.set_line_width(*lwidth as _);

                for seg in coll {   let res = segment_to_path(seg, &path);
                    ctx2d.fill_with_path_2d(&path);

                    if res { stroke_segment_path(seg, ctx2d);
                    } else { ctx2d.stroke_with_path(&path); }
                    path = Path2d::new().unwrap();
                }
            }
        }
    }
}

fn stroke_segment_path(seg: &Segment, ctx2d: &Context2d) {
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
    let svg_arc = kurbo::SvgArc {
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
    ctx2d: &Context2d, style: &Style) -> String {
    fn to_css_color<R: io::Read, W: io::Write>(img: &TinyVG<R, W>, idx: u32) -> String {
        let color = img.lookup_color(idx);
        let mut str = format!("#{:0<2x}{:0<2x}{:0<2x}", color.r, color.g, color.b);
        if color.a != 255 {    str.push_str(&format!("{:0<2x}", color.a)); }    str
    }

    match style {   Style::FlatColor(idx) => to_css_color(img, *idx),

        Style::LinearGradient { points, cindex } => {
            let linear = ctx2d.create_linear_gradient(
                points.0.x as _, points.0.y as _, points.1.x as _, points.1.y as _);
            let _ = linear.add_color_stop(0.0, &to_css_color(img, cindex.0));
            let _ = linear.add_color_stop(1.0, &to_css_color(img, cindex.1));
            linear.as_string().unwrap_or("".to_owned())
        }   // don't need to scale, since created in context
        Style::RadialGradient { points, cindex } => {
            let radial = ctx2d.create_radial_gradient(  // XXX: 0.
                points.0.x as _, points.0.y as _, 1., points.0.x as _, points.0.y as _,
               (points.1.x - points.0.x).hypot(points.1.y - points.0.y) as _).unwrap();
            let _ = radial.add_color_stop(0.0, &to_css_color(img, cindex.0));
            let _ = radial.add_color_stop(1.0, &to_css_color(img, cindex.1));
            radial.as_string().unwrap_or("".to_owned())
        }
    }
}

