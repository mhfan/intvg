
use crate::tinyvg::*;
use crate::gpac_evg::*;
use std::io;

pub trait Render { fn render(&self, scale: f32) -> Result<Pixmap, EvgError>; }

impl<R: io::Read, W: io::Write> Render for TinyVG<R, W> {
    fn render(&self, scale: f32) -> Result<Pixmap, EvgError> {
        let width  = (self.header.width  as f32 * scale).ceil() as _;
        let height = (self.header.height as f32 * scale).ceil() as _;

        #[allow(non_local_definitions)] impl From<&Rect> for GF_Rect {
            fn from(rect: &Rect) -> Self {  // XXX: screen to world coordinates
                Self {  x: rect.x.into(), y: (rect.y + rect.h).into(),
                    width: rect.w.into(),      height: rect.h .into(), }
            }
        }

        // XXX: rendering up-scale and then scale down for anti-aliasing?
        let mut surf = Surface::new(width, height, GF_PixelFormat::GF_PIXEL_RGBA)?;
        let (mut path, mut pens) = (VGPath::new()?, GF_PenSettings::default());

        let trfm = GF_Matrix2D {
            m: [scale.into(), 0.into(), 0.into(), 0.into(), scale.into(), 0.into()] };
        surf.set_matrix(Some(&trfm));

        for cmd in &self.commands {
            match cmd { Command::EndOfDocument => (),
                Command::FillPolyg(FillCMD { fill, coll }) => {
                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.move_to((*pt).into()) }
                    iter.for_each(|pt| path.line_to((*pt).into()));  path.close();
                    surf.fill_path(&path, &style_to_stencil(self, fill)?)?;
                }
                Command::FillRects(FillCMD { fill, coll }) => {
                    let sten = style_to_stencil(self, fill)?;
                    coll.iter().for_each(|rect| path.add_rect(rect.into()));
                    surf.fill_path(&path, &sten)?;    //path.reset();
                }
                Command::FillPath (FillCMD { fill, coll }) => {
                    let sten = style_to_stencil(self, fill)?;
                    for seg in coll { let _ = segment_to_path(seg, &mut path)?; }
                    surf.fill_path(&path, &sten)?;    //path.reset();
                }
                Command::DrawLines(DrawCMD { line, lwidth, coll }) => {
                    coll.iter().for_each(|line| {
                        path.move_to(line.start.into()); path.line_to(line.  end.into());
                    }); pens.width =  (*lwidth).into();
                    surf.stroke_path(&path, &style_to_stencil(self, line)?, &pens)?;
                }
                Command::DrawLoop (DrawCMD { line, lwidth, coll },
                    strip) => {     let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.move_to((*pt).into()) }
                    iter.for_each(|pt| path.line_to((*pt).into()));

                    if !*strip { path.close(); }    pens.width = (*lwidth).into();
                    surf.stroke_path(&path, &style_to_stencil(self, line)?, &pens)?;
                }
                Command::DrawPath (DrawCMD {
                    line, lwidth, coll }) => {
                    let sten = style_to_stencil(self, line)?;
                    pens.width = (*lwidth).into();

                    for seg in coll {
                        stroke_segment_path(seg, &mut surf, &sten, &mut pens)?; }
                }
                Command::OutlinePolyg(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    pens.width = (*lwidth).into();

                    let mut iter = coll.iter();
                    if let Some(pt) = iter.next() { path.move_to((*pt).into()) }
                    iter.for_each(|pt| path.line_to((*pt).into()));  path.close();

                    surf.fill_path(&path, &style_to_stencil(self, fill)?)?;
                    surf.stroke_path(&path, &style_to_stencil(self, line)?, &pens)?;
                }
                Command::OutlineRects(fill, DrawCMD {
                    line, lwidth, coll }) => {
                    pens.width = (*lwidth).into();
                    let paint = style_to_stencil(self, fill)?;
                    let pline = style_to_stencil(self, line)?;

                    coll.iter().for_each(|rect| path.add_rect(rect.into()));
                    surf.fill_path(&path, &paint)?;
                    surf.stroke_path(&path, &pline, &pens)?;     //path.reset();
                }
                Command::OutlinePath (fill, DrawCMD {
                    line, lwidth, coll }) => {
                    let paint = style_to_stencil(self, fill)?;
                    let pline = style_to_stencil(self, line)?;

                    pens.width = (*lwidth).into();  let mut res = false;
                    for seg in coll { res = segment_to_path(seg, &mut path)?; }
                    surf.fill_path(&path, &paint)?;

                    if res { for seg in coll {
                        stroke_segment_path(seg, &mut surf, &pline, &mut pens)?;
                    } } else { surf.stroke_path(&path, &pline, &pens)?; }  //path.reset();
                }
            }   path.reset();
        }
        Ok(surf.end())
    }
}

fn stroke_segment_path(seg: &Segment, surf: &mut Surface,
    sten: &Stencil, pens: &mut GF_PenSettings) -> Result<(), EvgError> {
    let mut path = VGPath::new()?;
    path.move_to(seg.start.into());

    for cmd in &seg.cmds {
        if let Some(width) = cmd.lwidth {
            if 1 < path.len() {
                let start = path.last_point().unwrap();
                surf.stroke_path(&path, sten, pens)?;
                path.reset(); path.move_to(start);
            }   pens.width = width.into();
        }   process_segcmd(&mut path, &cmd.instr)?;
    }   surf.stroke_path(&path, sten, pens)
}

fn segment_to_path(seg: &Segment, path: &mut VGPath) -> Result<bool, EvgError> {
    path.move_to(seg.start.into());
    let mut change_lw = false;

    for cmd in &seg.cmds {
        if cmd.lwidth.is_some() { change_lw = true }
        process_segcmd(path, &cmd.instr)?;
    }   Ok(change_lw)
}

fn process_segcmd(path: &mut VGPath, cmd: &SegInstr) -> Result<(), EvgError> {
    match cmd {     SegInstr::ClosePath => path.close(),
        SegInstr::Line  { end } => path.line_to((*end).into()),
        SegInstr::HLine { x }     =>
            path.line_to(((*x).into(), path.last_point().unwrap().y).into()),
        SegInstr::VLine { y }     =>
            path.line_to((path.last_point().unwrap().x, (*y).into()).into()),

        SegInstr::CubicBezier { ctrl, end } =>
            path.cubic_to(ctrl.0.into(), ctrl.1.into(), (*end).into()),
        SegInstr::ArcCircle  { large, sweep, radius, end } =>
            return path.svg_arc_to(
                (*radius, *radius).into(), 0.into(), *large, *sweep, (*end).into()),

        SegInstr::ArcEllipse { large, sweep, radii,
            rotation, end } => return path.svg_arc_to((*radii).into(),
                (*rotation).into(), *large, *sweep, (*end).into()),

        SegInstr::QuadBezier { ctrl, end } =>
            path.quad_to((*ctrl).into(), (*end).into()),
    }   Ok(())
}

fn style_to_stencil<R: io::Read, W: io::Write>(img: &TinyVG<R, W>,
    style: &Style) -> Result<Stencil, EvgError> {
    #[allow(non_local_definitions)] impl From<RGBA8888> for GF_Color {
        fn from(color: RGBA8888) -> Self { // convert to 0xAARRGGBB
            (color.a as u32) << 24 | (color.r as u32) << 16 |
            (color.g as u32) <<  8 |  color.b as u32
        }
    }   use crate::gpac_evg::GF_StencilType::*;

    match style {
        Style::FlatColor(idx) => {
            let mut sten = Stencil::new(GF_STENCIL_SOLID)?;
            sten.set_color(img.lookup_color(*idx).into());  Ok(sten)
        }
        Style::LinearGradient { points, cindex } => {
            let mut sten = Stencil::new(GF_STENCIL_LINEAR_GRADIENT)?;
            sten.push_interpolation(0.into(), img.lookup_color(cindex.0).into());
            sten.push_interpolation(1.into(), img.lookup_color(cindex.1).into());
            sten.set_linear(points.0.into(), points.1.into());
            Ok(sten)    //sten.set_matrix(trfm);
        }
        Style::RadialGradient { points, cindex } => {
            let mut sten = Stencil::new(GF_STENCIL_RADIAL_GRADIENT)?;
            sten.push_interpolation(0.into(), img.lookup_color(cindex.0).into());
            sten.push_interpolation(1.into(), img.lookup_color(cindex.1).into());
            let radius = (points.1.x - points.0.x).hypot(points.1.y - points.0.y);
            sten.set_radial(points.0.into(), points.1.into(), (radius, radius).into());
            Ok(sten)    //sten.set_matrix(trfm);
        }
    }
}

impl From<Point> for GF_Point2D { fn from(pt: Point) -> Self { (pt.x, pt.y).into() } }

/* impl From<tiny_skia::Transform> for GF_Matrix2D {
    fn from(mv: tiny_skia::Transform) -> Self {
        Self { m: [mv.sx.into(), mv.kx.into(), mv.tx.into(),
                   mv.sy.into(), mv.ky.into(), mv.ty.into()] }
        // sx = m[0], kx = m[1], tx = m[2], sy = m[3], ky = m[4], ty = m[5]
    }
} */
