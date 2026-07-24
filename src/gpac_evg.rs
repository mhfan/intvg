/****************************************************************
 * $ID: gpac_evg.rs  	Tue 24 Oct 2023 15:58:07+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

//pub mod gpac_evg {    // https://github.com/gpac/gpac/tree/master/src/evg/
use core::{marker::PhantomData, ptr::NonNull};

#[allow(unused, non_snake_case, non_camel_case_types)]
    //non_upper_case_globals, //clippy::approx_constant, clippy::useless_transmute,
mod evg_ffi { include!("../target/bindings/gpac_evg.rs"); }     use evg_ffi::*;
pub use evg_ffi::{GF_Point2D, GF_Rect, GF_Color, GF_Matrix2D, GF_PenSettings, GF_StencilType};

macro_rules! evg_result {
    ($v:expr) => {{ let result = unsafe { $v } as i32;
        if result == GF_Err::GF_OK as i32 { Ok(()) } else { Err(EvgError(result)) }
    }};
}

macro_rules! evg_debug {
    ($v:expr) => {{ let result = evg_result!($v); debug_assert!(result.is_ok()); }};
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct EvgError(i32);
impl EvgError {
    fn out_of_memory() -> Self { Self(GF_Err::GF_OUT_OF_MEM as i32) }
    fn bad_parameter() -> Self { Self(GF_Err::GF_BAD_PARAM as i32) }
    pub fn code(self) -> i32 { self.0 }
}
impl core::fmt::Display for EvgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GPAC EVG error {}", self.0)
    }
}
impl std::error::Error for EvgError {}

impl From<i32> for Fixed {  // 16.16 fixed-point, or 26.6?
    #[cfg(feature = "evg_fixed")] fn from(v: i32) -> Self { Self(v << 16) }
    #[cfg(not(feature = "evg_fixed"))] fn from(v: i32) -> Self { Self(v as _) }
}
#[cfg(feature = "evg_fixed")] impl From<Fixed> for i32 {
    fn from(v: Fixed) -> Self { (v.0 + (1 << 15)) >> 16 }
}
impl From<f32> for Fixed { #[cfg(feature = "evg_fixed")]
    fn from(v: f32) -> Self { Self((v * (1 << 16) as f32) as _) }
    #[cfg(not(feature = "evg_fixed"))] fn from(v: f32) -> Self { Self(v) }
}
impl From<Fixed> for f32 { #[cfg(feature = "evg_fixed")]
    fn from(v: Fixed) -> Self { v.0 as f32 / (1 << 16) as f32 }
    #[cfg(not(feature = "evg_fixed"))] fn from(v: Fixed) -> Self { v.0 }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self { if value { Bool::GF_TRUE } else { Bool::GF_FALSE } }
}

impl From<(Fixed, Fixed)> for GF_Point2D {
    fn from((x, y): (Fixed, Fixed)) -> Self { Self { x, y } }
}
impl From<(f32, f32)> for GF_Point2D {
    fn from((x, y): (f32, f32)) -> Self { Self { x: x.into(), y: y.into() } }
}

impl Copy  for Fixed {}
impl Copy  for GF_Point2D {}
impl Copy  for GF_PenSettings {}
impl Clone for Fixed { fn clone(&self) -> Self { *self } }
impl Clone for GF_Point2D { fn clone(&self) -> Self { *self } }
impl Clone for GF_PenSettings { fn clone(&self) -> Self { *self } }

impl Default for GF_PenSettings {
    fn default() -> Self { Self {
        width: 0.into(), cap: 1, join: 1, align: 0, dash: 0,
        // GF_LINE_(CAP/JOIN)_ROUND, GF_PATH_LINE_CENTER, GF_DASH_STYLE_PLAIN
        dash_offset: 0.into(), dash_set: core::ptr::null_mut(),
        path_length: 0.into(), miterLimit: 4.into(),
    } }
}

pub struct VGPath(NonNull<GF_Path>);
impl Drop for VGPath {
    // SAFETY: `self.0` is owned by this wrapper and released exactly once.
    fn drop(&mut self) { unsafe { gf_path_del(self.0.as_ptr()) } }
}
impl VGPath { // to build path and stencil
    pub fn new() -> Result<Self, EvgError> {
        // SAFETY: the returned allocation is uniquely owned by `VGPath`.
        NonNull::new(unsafe { gf_path_new() }).map(Self).ok_or_else(EvgError::out_of_memory)
    }

    pub fn move_to(&mut self, mut pt: GF_Point2D) {
        evg_debug!(gf_path_add_move_to_vec(self.0.as_ptr(), &mut pt));
    }

    pub fn line_to(&mut self, mut pt: GF_Point2D) {
        evg_debug!(gf_path_add_line_to_vec(self.0.as_ptr(), &mut pt));
    }

    pub fn cubic_to(&mut self, mut c1: GF_Point2D, mut c2: GF_Point2D, mut pt: GF_Point2D) {
        evg_debug!(gf_path_add_cubic_to_vec(
            self.0.as_ptr(), &mut c1, &mut c2, &mut pt));
    }

    pub fn quad_to(&mut self, mut cp: GF_Point2D, mut pt: GF_Point2D) {
        evg_debug!(gf_path_add_quadratic_to_vec(self.0.as_ptr(), &mut cp, &mut pt));
    }

    pub fn svg_arc_to(&mut self, radius: GF_Point2D, x_rot: Fixed, large: bool,
        sweep: bool, pt: GF_Point2D) -> Result<(), EvgError> {
        evg_result!(gf_path_add_svg_arc_to(self.0.as_ptr(), pt.x, pt.y,
            radius.x, radius.y, x_rot, large.into(), sweep.into()))
    }

    pub fn add_rect(&mut self, rect: GF_Rect) {
        evg_debug!(gf_path_add_rect(
            self.0.as_ptr(), rect.x, rect.y, rect.width, rect.height));
    }

    //gf_path_add_arc_to(path, end_x, end_y, fa_x, fa_y, fb_x, fb_y, cw);
    //gf_path_add_arc(path, radius, start_angle, end_angle, close_type);
    //gf_path_add_ellipse(path, cx, cy, a_axis, b_axis);
    //gf_path_add_bezier(path, pts, nb_pts);

    // SAFETY: `self.0` is a live, exclusively borrowed path.
    pub fn reset(&mut self) { unsafe { gf_path_reset(self.0.as_ptr()) }; }

    #[allow(clippy::len_without_is_empty)]
    // SAFETY: `self.0` stays valid for the shared borrow.
    pub fn len(&self) -> u32 { unsafe { self.0.as_ref().n_points } }

    pub fn last_point(&self) -> Option<GF_Point2D> {
        let cnt = self.len();
        if  cnt == 0 { return None; }
        // SAFETY: a path with `cnt > 0` owns at least `cnt` point entries.
        Some(unsafe { *self.0.as_ref().points.add(cnt as usize - 1) })
    }

    pub fn print_out(&self) {
        // SAFETY: point and tag arrays contain `n_points` entries.
        unsafe {    let path = self.0.as_ref();
            for n in 0..path.n_points {     let n = n as _;
                let pt = &*path.points.add(n);
                eprintln!("{}-({:?}, {:?})", *path.tags.add(n), //pt.x, pt.y);
                    <f32>::from(pt.x), <f32>::from(pt.y));
            }
        }
    }

    // XXX: fix and simplify difference judgement in path2d.c
    pub fn close(&mut self) { evg_debug!(gf_path_close(self.0.as_ptr())); }
}

pub struct Stencil(NonNull<GF_EVGStencil>);
impl Drop for Stencil {
    // SAFETY: `self.0` is owned by this wrapper and released exactly once.
    fn drop(&mut self) { unsafe { gf_evg_stencil_delete(self.0.as_ptr()) } }
}
impl Stencil {
    pub fn new(t: GF_StencilType) -> Result<Self, EvgError> {
        // SAFETY: the returned allocation is uniquely owned by `Stencil`.
        NonNull::new(unsafe { gf_evg_stencil_new(t) })
            .map(Self).ok_or_else(EvgError::out_of_memory)
    }

    pub fn set_color(&mut self, color: GF_Color) {
        evg_debug!(gf_evg_stencil_set_brush_color(self.0.as_ptr(), color));
    }

    pub fn set_linear(&mut self, start: GF_Point2D, end: GF_Point2D) {
        evg_debug!(gf_evg_stencil_set_linear_gradient(
            self.0.as_ptr(), start.x, start.y, end.x, end.y));
    }

    pub fn set_radial(&mut self, center: GF_Point2D, focal: GF_Point2D,
        radius: GF_Point2D) {
        evg_debug!(gf_evg_stencil_set_radial_gradient(self.0.as_ptr(),
            center.x, center.y, focal.x, focal.y, radius.x, radius.y));
    }

    pub fn push_interpolation(&mut self, pos: Fixed, col: GF_Color) {
        let result = evg_result!(gf_evg_stencil_push_gradient_interpolation(
            self.0.as_ptr(), pos, col));
        // GPAC's fixed-point implementation can report GF_OUT_OF_MEM here
        // despite retaining a usable stencil; preserve its established behavior.
        debug_assert!(result.is_ok() || cfg!(feature = "evg_fixed"));
    }

    /* pub fn set_interpolation(&mut self, pos: &[Fixed],
        col: &[GF_Color]) -> Result<(), EvgError> {
        if pos.len() != col.len() { return Err(EvgError::bad_parameter()); }
        evg_result!(gf_evg_stencil_set_gradient_interpolation(self.0.as_ptr(),
            pos.as_ptr().cast_mut(), col.as_ptr().cast_mut(), pos.len() as _))
    } */

    //evg_debug!(gf_evg_stencil_set_gradient_mode(sten, GF_GradientMode::GF_GRADIENT_MODE_PAD));
    //evg_debug!(gf_evg_stencil_set_alpha(sten, alpha));

    pub fn set_matrix(&mut self, mat: &GF_Matrix2D) {
        evg_debug!(gf_evg_stencil_set_matrix(
            self.0.as_ptr(), core::ptr::from_ref(mat).cast_mut()));
    }
}

pub struct Surface<'a>(NonNull<GF_EVGSurface>, PhantomData<&'a mut Pixmap>);
impl Drop for Surface<'_> {
    // SAFETY: `self.0` is owned by this wrapper and released exactly once.
    fn drop(&mut self) { unsafe { gf_evg_surface_delete(self.0.as_ptr()) } }
}
impl<'a> Surface<'a> {
    pub fn new(pixm: &'a mut Pixmap) -> Result<Self, EvgError> {
        let row_bytes = pixm.width.checked_mul(4).ok_or_else(EvgError::bad_parameter)?;
        let required = row_bytes.checked_mul(pixm.height)
            .ok_or_else(EvgError::bad_parameter)? as usize;
        let stride = i32::try_from(row_bytes).map_err(|_| EvgError::bad_parameter())?;
        if pixm.data.len() < required { return Err(EvgError::bad_parameter()); }

        // SAFETY: the returned allocation is uniquely owned by `Surface`.
        let ptr = NonNull::new(unsafe { gf_evg_surface_new(Bool::GF_FALSE) })
            .ok_or_else(EvgError::out_of_memory)?;
        let surf = Self(ptr, PhantomData);
        evg_result!(gf_evg_surface_attach_to_buffer(surf.0.as_ptr(),
            pixm.data.as_mut_ptr(), pixm.width, pixm.height, 4, stride,
            GF_PixelFormat::GF_PIXEL_RGBA))?;
        //evg_debug!(gf_evg_surface_clear(surf, &mut bbox, 0xFF000000));
        Ok(surf)
    }

    pub fn fill_path(&mut self, path: &VGPath, sten: &Stencil) -> Result<(), EvgError> {
        evg_result!(gf_evg_surface_set_path(self.0.as_ptr(), path.0.as_ptr()))?;
        evg_result!(gf_evg_surface_fill(self.0.as_ptr(), sten.0.as_ptr()))
    }

    pub fn stroke_path(&mut self, path: &VGPath, sten: &Stencil,
        pens: &GF_PenSettings) -> Result<(), EvgError> {
        // SAFETY: GPAC returns a newly allocated path owned by the caller.
        let path = NonNull::new(unsafe { gf_path_get_outline(path.0.as_ptr(), *pens) })
            .map(VGPath).ok_or_else(EvgError::out_of_memory)?;
        self.fill_path(&path, sten)
    }

    pub fn set_matrix(&mut self, mat: &GF_Matrix2D) {
        evg_debug!(gf_evg_surface_set_matrix(
            self.0.as_ptr(), core::ptr::from_ref(mat).cast_mut()));
    }
}

pub struct Pixmap { pub data: Vec<u8>, pub width: u32, pub height: u32, }

impl Pixmap {
    pub fn new(width: u32, height: u32) -> Result<Self, EvgError> {
        let len = width.checked_mul(height).and_then(|v| v.checked_mul(4))
            .ok_or_else(EvgError::bad_parameter)?;
        Ok(Self { width, height, data: vec![0; len as _] })
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
    #[test] fn rejects_short_surface_buffer() {
        let mut pixm = Pixmap { data: vec![0; 15], width: 2, height: 2 };
        assert!(Surface::new(&mut pixm).is_err());
    }

    #[test] fn fill_stroke() -> Result<(), Box<dyn std::error::Error>> {
        let mut pixm = Pixmap::new(1024, 512)?;
        let (width, height) = (pixm.width, pixm.height);
        let mut pens = GF_PenSettings::default();
        let mut sten = Stencil::new(GF_StencilType::GF_STENCIL_SOLID)?;
        let (mut surf, mut path) = (Surface::new(&mut pixm)?, VGPath::new()?);

        path.add_rect(GF_Rect { x: (width as i32 >> 2).into(),
            y:   (height as i32 - (height as i32 >> 2)).into(),
            width: (width as i32 >> 1).into(), height: (height as i32 >> 1).into() });
        // RUSTDOCFLAGS="-Z unstable-options --nocapture" cargo +nightly test #--doc

        /* path.move_to(GF_Point2D { x: rect.x, y: rect.y });
        path.line_to(GF_Point2D { x: Fixed(rect.x.0 + rect.width.0), y: rect.y });
        path.line_to(GF_Point2D { x: Fixed(rect.x.0 + rect.width.0),
            y: Fixed(rect.y.0 - rect.height.0) });
        path.line_to(GF_Point2D { x: rect.x, y: Fixed(rect.y.0 - rect.height.0) });
        path.line_to(GF_Point2D { x: rect.x, y: rect.y });  path.print_out();
        path.close(); */

        sten.set_color(0xFF0000FF); surf.fill_path(&path, &sten)?;
        sten.set_color(0xAA00FF00); pens.width = 10.into();
        surf.stroke_path(&path, &sten, &pens)?;

        drop(surf);
        pixm.save_png("target/demo_evg.png")?;  Ok(())
    }
}
