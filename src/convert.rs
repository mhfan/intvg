
use crate::tinyvg::*;
use std::{error::Error, io};
use usvg::tiny_skia_path as skia;

pub trait Convert { fn from_usvg(svgd: &[u8]) ->
    Result<Self, Box<dyn Error>> where Self: std::marker::Sized;
}

impl<R: io::Read, W: io::Write> Convert for TinyVG<R, W> {
    fn from_usvg(svgd: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut fontdb = usvg::fontdb::Database::new(); fontdb.load_system_fonts();
        let tree = usvg::Tree::from_data(svgd, //&std::fs::read(&path)?,
            &usvg::Options::default(), &fontdb)?;

        //Ok(Self::from_usvg(&tree)) }
        //fn from_usvg(tree: &usvg::Tree) -> Self {

        let mut tvg = Self::new();
        tvg.header.width  = tree.size().width() .round() as _;
        tvg.header.height = tree.size().height().round() as _;

        let coordinate_limit = u32::max(tvg.header.width, tvg.header.height);
        //let range_bits = match tvg.header.coordinate_range { CoordinateRange::Default => 16,
        //    CoordinateRange::Reduced => 8, CoordinateRange::Enhanced => 32,
        //} - 1; // coordinate_range/write_default isn't changed from default
        let (range_bits, mut scale_bits) = (16 - 1, 0);

        while scale_bits < range_bits && (coordinate_limit << (scale_bits + 1)) <
            (1 << range_bits) { scale_bits += 1; }  tvg.header.scale = scale_bits;
        // XXX: still need a traversely check CoordinateRange by a null writer?

        let trfm = tree.view_box().to_transform(tree.size())
            .pre_concat(tree.root().transform());
        convert_nodes(&mut tvg, tree.root(), &trfm);
        println!("{:?}, {} colors, {} cmds/paths", &tvg.header,
            tvg.color_table.len(), tvg.commands.len());     Ok(tvg)
    }
}

fn convert_nodes<R: io::Read, W: io::Write>(tvg: &mut TinyVG<R, W>,
    parent: &usvg::Group, trfm: &usvg::Transform) {
    for child in parent.children() { match child {
        usvg::Node::Group(group) =>     // XXX: trfm is needed on rendering only
            convert_nodes(tvg, group, &trfm.pre_concat(group.transform())),

        usvg::Node::Path(path) => {     let mut lwidth = 0.0;
            if path.visibility() != usvg::Visibility::Visible { continue }
            let coll = convert_path(path.data(), trfm);

            let fill = path  .fill().and_then(|fill|
                convert_paint(tvg, fill.paint(), fill.opacity(), trfm));
            let line = path.stroke().and_then(|line| {
                lwidth = line.width().get();    // XXX: need to apply transform?
                convert_paint(tvg, line.paint(), line.opacity(), trfm) });

            //match path.paint_order() {} // XXX:
            let cmd = match (fill, line) {
                (Some(fill), None) => Command::FillPath(FillCMD { fill, coll }),
                (None, Some(line)) => Command::DrawPath(DrawCMD { line, lwidth, coll }),
                (Some(fill), Some(line)) =>
                    Command::OutlinePath(fill, DrawCMD { line, lwidth, coll }),

                _ => { eprintln!("Neither fill nor line"); continue }
            };  tvg.commands.push(cmd);
        }

        usvg::Node::Image(_) => eprintln!("Not support image node"),
        usvg::Node::Text(text) => { let group = text.flattened();
            convert_nodes(tvg, group, &trfm.pre_concat(group.transform()));
        }
    } }
}

fn convert_path(path: &skia::Path, trfm: &usvg::Transform) -> Vec<Segment> {
    impl From<skia::Point> for Point {  //unsafe { std::mem::transmute(pt) }
        fn from(pt: skia::Point) -> Self { Self { x: pt.x, y: pt.y } }
    }

    let (mut coll, mut cmds) = (vec![], vec![]);
    let  mut start = Point { x: 0.0, y: 0.0 };
    let tpath = if trfm.is_identity() { None
    } else { path.clone().transform(*trfm) };

    for seg in tpath.as_ref().unwrap_or(path).segments() {
        let instr = match seg {
            skia::PathSegment::MoveTo(pt) => { //trfm.map_point(&mut pt);
                if !cmds.is_empty() { coll.push(Segment { start, cmds }); cmds = vec![]; }
                start = pt.into();  continue
            }
            skia::PathSegment::LineTo(pt) => { //trfm.map_point(&mut pt);
                SegInstr::Line { end: pt.into() }
            }
            skia::PathSegment::QuadTo(ctrl, end) => {
                //trfm.map_point(&mut ctrl);  trfm.map_point(&mut end);
                SegInstr::QuadBezier { ctrl: ctrl.into(), end: end.into() }
            }
            skia::PathSegment::CubicTo(ctrl0, ctrl1, end) => {
                //trfm.map_point(&mut ctrl0);
                //trfm.map_point(&mut ctrl1);   trfm.map_point(&mut end);
                SegInstr::CubicBezier { ctrl: (ctrl0.into(), ctrl1.into()), end: end.into() }
            }

            skia::PathSegment::Close => SegInstr::ClosePath,
        };  cmds.push(SegmentCommand { instr, lwidth: None });
    }   if !cmds.is_empty() { coll.push(Segment { start, cmds }); }     coll
}

fn convert_paint<R: io::Read, W: io::Write>(tvg: &mut TinyVG<R, W>,
    paint: &usvg::Paint, opacity: usvg::Opacity, _trfm: &usvg::Transform) -> Option<Style> {
    let get_color = |stop: &usvg::Stop| {
        let color = stop.color();
        RGBA8888 { r: color.red, g: color.green, b: color.blue,
                   a: (stop.opacity() * opacity).to_u8() }
    };

    impl From<(f32, f32)> for Point {   // unsafe { std::mem::transmute(pt) }
        fn from(pt: (f32, f32)) -> Self { Self { x: pt.0, y: pt.1 } }
    }

    match paint { usvg::Paint::Pattern(_) => {  // trfm should be applied here
            eprintln!("Not support pattern painting"); None },
        usvg::Paint::Color(color) => {
            Some(Style::FlatColor(tvg.push_color(RGBA8888 { r: color.red,
                g: color.green, b: color.blue, a: opacity.to_u8() })))
        }
        usvg::Paint::LinearGradient(grad) => {
            let p0 = (grad.x1(), grad.y1()).into();
            let p1 = (grad.x2(), grad.y2()).into();
            let c0 = tvg.push_color(get_color(&grad.stops()[0]));
            let c1 = tvg.push_color(get_color(&grad.stops()[1]));
            Some(Style::LinearGradient { points: (p0, p1), cindex: (c0, c1) })
        }
        usvg::Paint::RadialGradient(grad) => {
            let p0 = (grad.fx(), grad.fy()).into(); // focus/start, center/end
            let p1 = (grad.cx(), grad.cy() + grad.r().get()).into();
            let c0 = tvg.push_color(get_color(&grad.stops()[0]));
            let c1 = tvg.push_color(get_color(&grad.stops()[1]));
            Some(Style::RadialGradient { points: (p0, p1), cindex: (c0, c1) })
        }
    }
}

