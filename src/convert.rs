
use crate::tinyvg::*;

pub trait Convert { fn from_usvg(tree: &usvg::Tree) -> Self; }

impl Convert for TVGImage {
    fn from_usvg(tree: &usvg::Tree) -> Self { // traversely check CoordinateRange?
        let mut img = Self::new();  img.header.scale  = 1;
        // XXX: handle scale and coordinate range? https://github.com/TinyVG/sdk/pull/5
        img.header.width  = tree.size.width() .round() as u32;
        img.header.height = tree.size.height().round() as u32;

        let trans = usvg::utils::view_box_to_transform(
            tree.view_box.rect, tree.view_box.aspect, tree.size);
        convert_children(&mut img, tree, &tree.root, trans);
        eprintln!("{:?}", &img.header);
        img
    }
}

fn convert_children(img: &mut TVGImage, tree: &usvg::Tree,
    parent: &usvg::Node, trans: usvg::Transform) {
    for child in parent.children() {
        match *child.borrow() {
            usvg::NodeKind::Group(ref g) =>
                convert_children(img, tree, &child, trans.pre_concat(g.transform)),

            usvg::NodeKind::Path(ref path) => { // XXX: how to avoid clone path.data?
                let new_path = (*path.data).clone().transform(trans).unwrap();
                let (bbox, mut lwidth) = (new_path.bounds(), 0.0);

                let fill = path.  fill.as_ref().and_then(|fill|
                    convert_paint(img, &fill.paint, fill.opacity, bbox));
                let line = path.stroke.as_ref().and_then(|line| {
                    lwidth = line.width.get() as f32;
                    convert_paint(img, &line.paint, line.opacity, bbox) });

                let cmd = match (fill, line) {  // XXX: detect simple shape (line/rect)?
                    (Some(fill), None) => Command::FillPath(FillCMD { fill,
                        coll: convert_path_segment(&new_path) }),
                    (None, Some(line)) => Command::DrawPath(DrawCMD { line, lwidth,
                        coll: convert_path_segment(&new_path) }),
                    (Some(fill), Some(line)) => Command::OutlinePath(fill,
                        DrawCMD { line, lwidth, coll: convert_path_segment(&new_path) }),
                    _ => continue
                };  img.commands.push(cmd);
            }

            usvg::NodeKind::Image(ref img) => eprintln!("Unsupported {:?}", img.kind),
            usvg::NodeKind::Text(_)  => eprintln!("Text Should be converted to path in usvg"),
        }
    }
}

fn convert_path_segment(path: &usvg::tiny_skia_path::Path) -> Vec<Segment> {
    let (mut coll, mut cmds) = (vec![], vec![]);
    let  mut start = Point { x: 0.0, y: 0.0 };
    for seg in path.segments() {
        let instr = match seg {
            usvg::tiny_skia_path::PathSegment::MoveTo(pt) => { start = pt.into();
                if !cmds.is_empty() {   coll.push(Segment { start, cmds });
                    cmds = vec![]; }    continue
            }
            usvg::tiny_skia_path::PathSegment::LineTo(pt) =>
                SegInstr::Line { end: pt.into() },
            usvg::tiny_skia_path::PathSegment::QuadTo(ctrl, end) =>
                SegInstr::QuadBezier { ctrl: ctrl.into(), end: end.into() },
            usvg::tiny_skia_path::PathSegment::CubicTo(ctrl1, ctrl2, end) =>
                SegInstr::CubicBezier { ctrl: (ctrl1.into(), ctrl2.into()), end: end.into() },
            usvg::tiny_skia_path::PathSegment::Close => SegInstr::ClosePath,
        };  cmds.push(SegmentCommand { instr, lwidth: None });
    }; if !cmds.is_empty() { coll.push(Segment { start, cmds }); }     coll
}

impl Into<Point> for usvg::tiny_skia_path::Point {
    //fn into(self) -> Point { Point { x: self.x, y: self.y } }
    fn into(self) -> Point { unsafe { std::mem::transmute(self) } }
}

fn convert_paint(img: &mut TVGImage, paint: &usvg::Paint,
    opacity: usvg::Opacity, bbox: usvg::Rect) -> Option<Style> {
    fn gradient_transform(grad: &usvg::BaseGradient,
        bbox: usvg::Rect) -> Option<usvg::Transform> {
        if grad.units == usvg::Units::ObjectBoundingBox {
            //let bbox = match bbox.to_rect() {
            //    Some(bbox) => bbox, None => return None };
            let  ts = usvg::Transform::from_row(bbox.width(),
                0.0, 0.0, bbox.height(), bbox.x(), bbox.y());
            Some(ts.pre_concat(grad.transform))
        } else { Some(grad.transform) }
    }

    fn convert_stop_color(img: &mut TVGImage, stop: &usvg::Stop,
        opacity: usvg::Opacity) -> u32 {    let color = stop.color;
        let color = RGBA8888 { r: color.red, g: color.green,
            b: color.blue, a: (stop.opacity * opacity).to_u8() };
        img.push_color(color)
    }

    match paint { usvg::Paint::Pattern(_) => None,
        usvg::Paint::Color(color) => {
            let color = RGBA8888 { r: color.red, g: color.green,
                b: color.blue, a: opacity.to_u8() };
            Some(Style::FlatColor(img.push_color(color)))
        }
        usvg::Paint::LinearGradient(grad) => {
            let ts = gradient_transform(grad, bbox)?;
            let mut p1 = (grad.x1, grad.y1).into();
            let mut p2 = (grad.x2, grad.y2).into();
            ts.map_point(&mut p1);  ts.map_point(&mut p2);

            let c1 = convert_stop_color(img, &grad.stops[0], opacity);
            let c2 = convert_stop_color(img, &grad.stops[1], opacity);
            Some(Style::LinearGradient { points: (p1.into(), p2.into()), cindex: (c1, c2) })
        }
        usvg::Paint::RadialGradient(grad) => {
            let ts = gradient_transform(grad, bbox)?;
            let mut p1 = (grad.cx, grad.cy).into();
            //let mut p2 = (grad.fx, grad.fy).into();   // XXX:
            let mut p2 = (grad.cx, grad.cy + grad.r.get()).into();
            ts.map_point(&mut p1);  ts.map_point(&mut p2);

            let c1 = convert_stop_color(img, &grad.stops[0], opacity);
            let c2 = convert_stop_color(img, &grad.stops[1], opacity);
            Some(Style::RadialGradient { points: (p1.into(), p2.into()), cindex: (c1, c2) })
        }
    }
}
