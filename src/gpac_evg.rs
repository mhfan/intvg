/****************************************************************
 * $ID: gpac_evg.rs  	Tue 24 Oct 2023 15:58:07+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

//pub mod gpac_evg {    // https://github.com/gpac/gpac/tree/master/src/evg/
#[allow(non_snake_case)] #[allow(non_camel_case_types)] //#[allow(non_upper_case_globals)]
#[allow(unused)] //#[allow(clippy::approx_constant)] #[allow(clippy::useless_transmute)]
mod evg_ffi { include!("../target/bindings/gpac_evg.rs"); }     use evg_ffi::*;
pub use evg_ffi::{GF_Point2D, GF_Rect, GF_Color, GF_Matrix2D, GF_PenSettings, GF_StencilType};

/*#[macro_export] */macro_rules! safe_dbg { //($v:expr$(,$g:expr)?) => { unsafe { $v } };
    ($v:expr,$g:expr) => { match unsafe { $v as i32 } {
        //eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($v), &res);
        res => { if res != $g as i32 { dbg!(res); } res } } };
    ($v:expr) => { safe_dbg!($v, 0) };
}

impl From<i32> for Fixed {  // 16.16 fixed-point, or 26.6?
    #[cfg(feature = "evg_fixed")] #[inline] fn from(v: i32) -> Self { Self(v << 16) }
    #[cfg(not(feature = "evg_fixed"))] #[inline] fn from(v: i32) -> Self { Self(v as _) }
}
#[cfg(feature = "evg_fixed")] impl From<Fixed> for i32 {
    #[inline] fn from(v: Fixed) -> Self { (v.0 + (1 << 15)) >> 16 }
}
impl From<f32> for Fixed { #[cfg(feature = "evg_fixed")]
    #[inline] fn from(v: f32) -> Self { Self((v * (1 << 16) as f32) as _) }
    #[cfg(not(feature = "evg_fixed"))] #[inline] fn from(v: f32) -> Self { Self(v) }
}
impl From<Fixed> for f32 { #[cfg(feature = "evg_fixed")]
    #[inline] fn from(v: Fixed) -> Self { v.0 as f32 / (1 << 16) as f32 }
    #[cfg(not(feature = "evg_fixed"))] #[inline] fn from(v: Fixed) -> Self { v.0 }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self { if value { Bool::GF_TRUE } else { Bool::GF_FALSE } }
}

impl From<(Fixed, Fixed)> for GF_Point2D {
    fn from((x, y): (Fixed, Fixed)) -> Self { Self { x, y } }
}

impl Copy  for Fixed {}
impl Copy  for GF_Point2D {}
impl Copy  for GF_PenSettings {}
impl Clone for Fixed { #[inline] fn clone(&self) -> Self { *self } }
impl Clone for GF_Point2D { #[inline] fn clone(&self) -> Self { *self } }
impl Clone for GF_PenSettings { #[inline] fn clone(&self) -> Self { *self } }

impl Default for GF_PenSettings {
    fn default() -> Self { Self {
        width: 0.into(), cap: 1, join: 1, align: 0, dash: 0,
        // GF_LINE_(CAP/JOIN)_ROUND, GF_PATH_LINE_CENTER, GF_DASH_STYLE_PLAIN
        dash_offset: 0.into(), dash_set: core::ptr::null_mut(),
        path_length: 0.into(), miterLimit: 4.into(),
    } }
}

pub struct VGPath(*mut GF_Path);
impl Drop for VGPath { #[inline] fn drop(&mut self) { unsafe { gf_path_del(self.0) } } }
impl VGPath { #[allow(clippy::new_without_default)] // to build path and stencil
    #[inline] pub fn new() -> Self { Self(unsafe { gf_path_new() }) }

    #[inline] pub fn move_to(&self, pt: GF_Point2D) {
        safe_dbg!(gf_path_add_move_to_vec(self.0, &pt as *const _ as _));
    }

    #[inline] pub fn line_to(&self, pt: GF_Point2D) {
        safe_dbg!(gf_path_add_line_to_vec(self.0, &pt as *const _ as _));
    }

    #[inline] pub fn cubic_to(&self, c1: GF_Point2D, c2: GF_Point2D, pt: GF_Point2D) {
        safe_dbg!(gf_path_add_cubic_to_vec(self.0,
            &c1 as *const _ as _, &c2 as *const _ as _, &pt as *const _ as _));
    }

    #[inline] pub fn quad_to(&self, cp: GF_Point2D, pt: GF_Point2D) {
        safe_dbg!(gf_path_add_quadratic_to_vec(self.0,
            &cp as *const _ as _, &pt as *const _ as _));
    }

    #[inline] pub fn svg_arc_to(&self, radius: GF_Point2D,
        x_rot: Fixed, large: bool, sweep: bool, pt: GF_Point2D) {
        safe_dbg!(gf_path_add_svg_arc_to(self.0, pt.x, pt.y,
            radius.x, radius.y, x_rot, large.into(), sweep.into()));
    }

    #[inline] pub fn add_rect(&self, rect: GF_Rect) {
        safe_dbg!(gf_path_add_rect(self.0, rect.x, rect.y, rect.width, rect.height));
    }

    //gf_path_add_arc_to(path, end_x, end_y, fa_x, fa_y, fb_x, fb_y, cw);
    //gf_path_add_arc(path, radius, start_angle, end_angle, close_type);
    //gf_path_add_ellipse(path, cx, cy, a_axis, b_axis);
    //gf_path_add_bezier(path, pts, nb_pts);

    #[inline] pub fn reset(&self) { unsafe { gf_path_reset(self.0) }; }

    #[allow(clippy::len_without_is_empty)] #[inline] pub fn len(&self) -> u32 {
        if let Some(path) = unsafe { self.0.as_ref() } { path.n_points } else { 0 }
    }
    pub fn last_point(&self) -> Option<GF_Point2D> {
        let cnt = self.len();   if cnt < 1 { None } else {
            Some(unsafe { *(*self.0).points.offset(cnt as isize - 1) })
        }
    }

    pub fn print_out(&self) {
        unsafe {    let path = &*self.0;
            for n in 0..path.n_points {     let n = n as _;
                let pt = &*path.points.offset(n);
                eprintln!("{}-({:?}, {:?})", *path.tags.offset(n), //pt.x, pt.y);
                    <f32>::from(pt.x), <f32>::from(pt.y));
            }
        }
    }

    // XXX: fix and simplify difference judgement in path2d.c
    #[inline] pub fn close(&self) { safe_dbg!(gf_path_close(self.0)); }
}

pub struct Stencil(*mut GF_EVGStencil);
impl Drop for Stencil { #[inline] fn drop(&mut self) { unsafe { gf_evg_stencil_delete(self.0) } } }
impl Stencil {
    #[inline] pub fn new(t: GF_StencilType) -> Self  { unsafe { Self(gf_evg_stencil_new(t)) } }

    #[inline] pub fn set_color(&self, color: GF_Color) {
        safe_dbg!(gf_evg_stencil_set_brush_color(self.0, color));
    }

    #[inline] pub fn set_linear(&self, start: GF_Point2D, end: GF_Point2D) {
        safe_dbg!(gf_evg_stencil_set_linear_gradient(self.0, start.x, start.y, end.x, end.y));
    }

    #[inline] pub fn set_radial(&self,
        center: GF_Point2D, focal: GF_Point2D, radius: GF_Point2D) {
        safe_dbg!(gf_evg_stencil_set_radial_gradient(self.0,
            center.x, center.y, focal.x, focal.y, radius.x, radius.y));
    }

    #[inline] pub fn push_interpolation(&self, pos: Fixed, col: GF_Color) {
        safe_dbg!(gf_evg_stencil_push_gradient_interpolation(self.0, pos, col));
    }

    /* #[inline] pub fn set_interpolation(&self, pos: &[Fixed], col: &[GF_Color], cnt: u32) {
       safe_dbg!(gf_evg_stencil_set_gradient_interpolation(self.0,
            pos.as_mut_ptr(), col.as_mut_ptr(), cnt));
    } */

    //safe_dbg!(gf_evg_stencil_set_gradient_mode(sten, GF_GradientMode::GF_GRADIENT_MODE_PAD));
    //safe_dbg!(gf_evg_stencil_set_alpha(sten, alpha));

    #[inline] pub fn set_matrix(&self, mat: &GF_Matrix2D) {
        safe_dbg!(gf_evg_stencil_set_matrix(self.0, mat as *const _ as _));
    }
}

pub struct Surface(*mut GF_EVGSurface);
impl Drop for Surface { #[inline] fn drop(&mut self) { unsafe { gf_evg_surface_delete(self.0) } } }
impl Surface {
    #[inline] pub fn new(pixm: &mut Pixmap) -> Self {
        let surf = Self(unsafe { gf_evg_surface_new(Bool::GF_FALSE) });
        safe_dbg!(gf_evg_surface_attach_to_buffer(surf.0,
            pixm.data.as_mut_ptr(), pixm.width, pixm.height, 4, (pixm.width << 2) as _,
            GF_PixelFormat::GF_PIXEL_RGBA));
        //safe_dbg!(gf_evg_surface_clear(surf, &mut bbox, 0xFF000000));
        surf
    }

    #[inline] pub fn fill_path(&self, path: &VGPath, sten: &Stencil) {
        safe_dbg!(gf_evg_surface_set_path(self.0, path.0));
        safe_dbg!(gf_evg_surface_fill(self.0, sten.0));
    }

    #[inline] pub fn stroke_path(&self, path: &VGPath, sten: &Stencil,  pens: &GF_PenSettings) {
        let path = VGPath(unsafe { gf_path_get_outline(path.0, *pens) });
        self.fill_path(&path, sten);
    }

    #[inline] pub fn set_matrix(&self, mat: &GF_Matrix2D) {
        safe_dbg!(gf_evg_surface_set_matrix(self.0, mat as *const _ as _));
    }
}

pub struct Pixmap { pub data: Vec<u8>, pub width: u32, pub height: u32, }

impl Pixmap {
    #[inline] pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, data: vec![0; (width * height * 4) as _] }
    }

    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(
            std::fs::File::create(path)?), self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        //    png::ScaledFloat::from_scaled(45455)  // 1.0 / 2.2 scaled by 100000
        //let source_chromaticities = png::SourceChromaticities::new( // unscaled instant
        //    (0.3127, 0.3290), (0.6400, 0.3300), (0.3000, 0.6000), (0.1500, 0.0600));
        //encoder.set_source_chromaticities(source_chromaticities);
        encoder.write_header()?.write_image_data(&self.data)?;  Ok(())
    }
}

//}

#[cfg(test)] mod tests { use super::*;
    #[test] fn fill_stroke() {
        let mut pixm = Pixmap::new(1024, 512);
        let mut pens = GF_PenSettings::default();
        let sten = Stencil::new(GF_StencilType::GF_STENCIL_SOLID);
        let (surf, path) = (Surface::new(&mut pixm), VGPath::new());

        path.add_rect(GF_Rect {   x: (pixm.width  as i32 >> 2) .into(),
            y: (pixm.height as i32 - (pixm.height as i32 >> 2)).into(),
            width: (pixm.width as i32 >> 1).into(), height: (pixm.height as i32 >> 1).into() });
        // RUSTDOCFLAGS="-Z unstable-options --nocapture" cargo +nightly test #--doc

        /* path.move_to(GF_Point2D { x: rect.x, y: rect.y });
        path.line_to(GF_Point2D { x: Fixed(rect.x.0 + rect.width.0), y: rect.y });
        path.line_to(GF_Point2D { x: Fixed(rect.x.0 + rect.width.0),
            y: Fixed(rect.y.0 - rect.height.0) });
        path.line_to(GF_Point2D { x: rect.x, y: Fixed(rect.y.0 - rect.height.0) });
        path.line_to(GF_Point2D { x: rect.x, y: rect.y });  path.print_out();
        path.close(); */

        sten.set_color(0xFF0000FF); surf.fill_path(&path, &sten);
        sten.set_color(0xAA00FF00); pens.width = 10.into();
        surf.stroke_path(&path, &sten, &pens);

        pixm.save_png("target/demo_evg.png").unwrap();
    }
}

