
use crate::tinyvg::*;
use usvg::tiny_skia_path as skia;
use std::{path::Path, fs, error::Error};

pub trait Convert { fn from_svgf<P: AsRef<Path>>(file: P) ->
    Result<Self, Box<dyn Error>> where Self: std::marker::Sized;
}

impl Convert for TVGImage {
    fn from_svgf<P: AsRef<Path>>(file: P) -> Result<Self, Box<dyn Error>> {
        use usvg::{TreeParsing, TreeTextToPath};
        let mut fontdb = usvg::fontdb::Database::new();     fontdb.load_system_fonts();
        let mut tree = usvg::Tree::from_data(&fs::read(&file)?,
            &usvg::Options::default())?;    tree.convert_text(&fontdb);

        //Ok(Self::from_usvg(&tree)) }
        //fn from_usvg(tree: &usvg::Tree) -> Self {

        let mut tvg = Self::new();
        tvg.header.width  = tree.size.width() .round() as u32;
        tvg.header.height = tree.size.height().round() as u32;

        let coordinate_limit = u32::max(tvg.header.width, tvg.header.height);
        //let range_bits = match tvg.header.coordinate_range { CoordinateRange::Default => 16,
        //    CoordinateRange::Reduced => 8, CoordinateRange::Enhanced => 32,
        //} - 1; // coordinate_range/write_default isn't changed from default
        let (range_bits, mut scale_bits) = (16 - 1, 0);

        while scale_bits < range_bits && (coordinate_limit << (scale_bits + 1)) <
            (1 << range_bits) { scale_bits += 1; }  tvg.header.scale = scale_bits;
        // XXX: still need a traversely check CoordinateRange by a null writer?

        let trans = usvg::utils::view_box_to_transform(
            tree.view_box.rect, tree.view_box.aspect, tree.size);
        convert_children(&mut tvg, &tree.root, trans);
        eprintln!("{:?}", &tvg.header);     Ok(tvg)
    }
}

fn convert_children(tvg: &mut TVGImage, parent: &usvg::Node, trans: usvg::Transform) {
    for child in parent.children() {
        match *child.borrow() {
            usvg::NodeKind::Group(ref g) =>
                convert_children(tvg, &child, trans.pre_concat(g.transform)),

            usvg::NodeKind::Path(ref path) => { // XXX: how to avoid clone path.data?
                let new_path = (*path.data).clone().transform(trans).unwrap();
                let (bbox, mut lwidth) = (new_path.bounds(), 0.0);

                let fill = path.  fill.as_ref().and_then(|fill|
                    convert_paint(tvg, &fill.paint, fill.opacity, bbox));
                let line = path.stroke.as_ref().and_then(|line| {
                    lwidth = line.width.get();
                    convert_paint(tvg, &line.paint, line.opacity, bbox) });

                let cmd = match (fill, line) {  // XXX: detect simple shape (line/rect)?
                    (Some(fill), None) => Command::FillPath(FillCMD { fill,
                        coll: convert_path_segment(&new_path) }),
                    (None, Some(line)) => Command::DrawPath(DrawCMD { line, lwidth,
                        coll: convert_path_segment(&new_path) }),
                    (Some(fill), Some(line)) => Command::OutlinePath(fill,
                        DrawCMD { line, lwidth, coll: convert_path_segment(&new_path) }),
                    _ => continue
                };  tvg.commands.push(cmd);
            }

            usvg::NodeKind::Image(ref tvg) => eprintln!("Unsupported {:?}", tvg.kind),
            usvg::NodeKind::Text(_)  => eprintln!("Text Should be converted to path in usvg"),
        }
    }
}

fn convert_path_segment(path: &skia::Path) -> Vec<Segment> {
    let (mut coll, mut cmds) = (vec![], vec![]);
    let  mut start = Point { x: 0.0, y: 0.0 };
    for seg in path.segments() {
        let instr = match seg {
            skia::PathSegment::MoveTo(pt) => { start = pt.into();
                if !cmds.is_empty() {   coll.push(Segment { start, cmds });
                    cmds = vec![]; }    continue
            }
            skia::PathSegment::LineTo(pt) =>
                SegInstr::Line { end: pt.into() },
            skia::PathSegment::QuadTo(ctrl, end) =>
                SegInstr::QuadBezier { ctrl: ctrl.into(), end: end.into() },
            skia::PathSegment::CubicTo(ctrl1, ctrl2, end) =>
                SegInstr::CubicBezier { ctrl: (ctrl1.into(), ctrl2.into()), end: end.into() },
            skia::PathSegment::Close => SegInstr::ClosePath,
        };  cmds.push(SegmentCommand { instr, lwidth: None });
    }   if !cmds.is_empty() { coll.push(Segment { start, cmds }); }     coll
}

#[allow(clippy::from_over_into)] impl Into<Point> for skia::Point {
    fn into(self) -> Point { unsafe { std::mem::transmute(self) } }
    //fn into(self) -> Point { Point { x: self.x, y: self.y } }
}

fn convert_paint(tvg: &mut TVGImage, paint: &usvg::Paint,
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

    fn convert_stop_color(tvg: &mut TVGImage, stop: &usvg::Stop,
        opacity: usvg::Opacity) -> u32 {    let color = stop.color;
        let color = RGBA8888 { r: color.red, g: color.green,
            b: color.blue, a: (stop.opacity * opacity).to_u8() };
        tvg.push_color(color)
    }

    match paint { usvg::Paint::Pattern(_) => {
            eprintln!("pattern painting is not supported"); None },
        usvg::Paint::Color(color) => {
            let color = RGBA8888 { r: color.red, g: color.green,
                b: color.blue, a: opacity.to_u8() };
            Some(Style::FlatColor(tvg.push_color(color)))
        }
        usvg::Paint::LinearGradient(grad) => {
            let ts = gradient_transform(grad, bbox)?;
            let mut p1 = (grad.x1, grad.y1).into();
            let mut p2 = (grad.x2, grad.y2).into();
            ts.map_point(&mut p1);  ts.map_point(&mut p2);

            let c1 = convert_stop_color(tvg, &grad.stops[0], opacity);
            let c2 = convert_stop_color(tvg, &grad.stops[1], opacity);
            Some(Style::LinearGradient { points: (p1.into(), p2.into()), cindex: (c1, c2) })
        }
        usvg::Paint::RadialGradient(grad) => {
            let ts = gradient_transform(grad, bbox)?;
            let mut p1 = (grad.cx, grad.cy).into();
            //let mut p2 = (grad.fx, grad.fy).into();   // XXX:
            let mut p2 = (grad.cx, grad.cy + grad.r.get()).into();
            ts.map_point(&mut p1);  ts.map_point(&mut p2);

            let c1 = convert_stop_color(tvg, &grad.stops[0], opacity);
            let c2 = convert_stop_color(tvg, &grad.stops[1], opacity);
            Some(Style::RadialGradient { points: (p1.into(), p2.into()), cindex: (c1, c2) })
        }
    }
}

