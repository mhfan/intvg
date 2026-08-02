/****************************************************************
 * $ID: gpac_evg.rs  	Tue 24 Oct 2023 15:58:07+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

//pub mod gpac_evg {    // https://github.com/gpac/gpac/tree/master/src/evg/
use core::ptr::{null_mut, NonNull};

#[allow(unused, non_snake_case, non_camel_case_types)]
    //non_upper_case_globals, //clippy::approx_constant, clippy::useless_transmute,
mod evg_ffi { include!("../target/bindings/gpac_evg.rs"); }     use evg_ffi::*;
pub use evg_ffi::{GF_Point2D, GF_Rect, GF_Color, GF_Matrix2D, GF_PenSettings, GF_StencilType,
    GF_PixelFormat, GF_EVGCompositeMode, GF_RasterQuality, GF_IRect,
};

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
    fn bad_parameter() -> Self { Self(GF_Err::GF_BAD_PARAM  as i32) }
    pub fn code(self) -> i32 { self.0 }
}
impl core::fmt::Display for EvgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GPAC EVG error {}", self.0)
    }
}
impl std::error::Error for EvgError {}

impl From<i32> for Fixed {  // 16.16 fixed-point, or 24.8?
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
impl From<(i32, i32)> for GF_Point2D {
    fn from((x, y): (i32, i32)) -> Self { Self { x: x.into(), y: y.into() } }
}

impl Copy  for Fixed {}
impl Copy  for GF_Point2D {}
impl Copy  for GF_PenSettings {}
impl Copy  for GF_PixelFormat {}
impl Clone for Fixed { fn clone(&self) -> Self { *self } }
impl Clone for GF_Point2D { fn clone(&self) -> Self { *self } }
impl Clone for GF_PenSettings { fn clone(&self) -> Self { *self } }

impl Default for GF_PenSettings {
    fn default() -> Self { Self {
        width: 0.into(), cap: 1, join: 1, align: 0, dash: 0,
        // GF_LINE_(CAP/JOIN)_ROUND, GF_PATH_LINE_CENTER, GF_DASH_STYLE_PLAIN
        dash_offset: 0.into(), dash_set: null_mut(),
        path_length: 0.into(), miterLimit: 4.into(),
    } }
}

impl GF_PenSettings {
    pub fn set_dash_pattern(&mut self, style: u8, pattern: &[Fixed], offset: Fixed) {
        self.dash_set = pattern.as_ptr() as _; // XXX:
        self.dash_offset = offset;
        self.dash = style as _;
    }
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
        if unsafe { gf_path_is_empty(self.0.as_ptr()) } == Bool::GF_TRUE {
            self.move_to(pt);   return
        }
        evg_debug!(gf_path_add_line_to_vec(self.0.as_ptr(), &mut pt));
    }

    pub fn cubic_to(&mut self, mut c1: GF_Point2D, mut c2: GF_Point2D, mut pt: GF_Point2D) {
        evg_debug!(gf_path_add_cubic_to_vec(self.0.as_ptr(), &mut c1, &mut c2, &mut pt));
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
        evg_debug!(gf_path_add_rect(self.0.as_ptr(),
            rect.x, rect.y, rect.width, rect.height));
    }

    //gf_path_add_arc_to(path, end_x, end_y, fa_x, fa_y, fb_x, fb_y, cw);
    //gf_path_add_arc(path, radius, start_angle, end_angle, close_type);
    //gf_path_add_ellipse(path, cx, cy, a_axis, b_axis);
    //gf_path_add_subpath(path, subpath, trfm);
    //gf_path_add_bezier(path, pts, nb_pts);

    //gf_path_point_over(path, x, y);
    //gf_path_get_bounds(path);

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

    pub fn print_out(&self) { unsafe {
        // SAFETY: point and tag arrays contain `n_points` entries.
            let path = self.0.as_ref();
            for n in 0..path.n_points {     let n = n as _;
                let pt = &*path.points.add(n);
                eprintln!("{}-({:?}, {:?})", *path.tags.add(n), //pt.x, pt.y);
                    f32::from(pt.x), f32::from(pt.y));
            }
    } }

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

    pub fn set_alpha(&mut self, alpha: u8) {
        evg_debug!(gf_evg_stencil_set_alpha(self.0.as_ptr(), alpha));
    }

    pub fn set_matrix(&mut self, mat: Option<&GF_Matrix2D>) {
        evg_debug!(gf_evg_stencil_set_matrix(self.0.as_ptr(),
            mat.map_or(null_mut(), |mat| mat as *const _ as _)));
    }
}

pub struct Surface(NonNull<GF_EVGSurface>, Option<Pixmap>);
impl Drop for Surface {
    // SAFETY: `self.0` is owned by this wrapper and released exactly once.
    fn drop(&mut self) { unsafe { gf_evg_surface_delete(self.0.as_ptr()) } }
}
impl Surface {
    pub fn new(width: u32, height: u32, fmt: GF_PixelFormat) -> Result<Self, EvgError> {
        Self::from_pixmap(Pixmap::new(width, height, fmt)?)
    }
    pub fn from_pixmap(pixm: Pixmap) -> Result<Self, EvgError> {
        // SAFETY: the returned allocation is uniquely owned by `Surface`.
        let ptr = NonNull::new(unsafe { gf_evg_surface_new(Bool::GF_FALSE) })
            .ok_or_else(EvgError::out_of_memory)?;
        let surf = Self(ptr, Some(pixm));
        let pixm = surf.1.as_ref().expect("Surface always owns its Pixmap");
        evg_result!(gf_evg_surface_attach_to_buffer(surf.0.as_ptr(), // XXX:
            pixm.data.as_ptr().cast_mut(), pixm.width, pixm.height, 0, 0, pixm.format))?;
        //evg_debug!(gf_evg_surface_clear(surf, &mut bbox, 0xFF000000));
        Ok(surf)
    }
    pub fn end(mut self) -> Pixmap { self.1.take().expect("Surface always owns its Pixmap") }

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

    pub fn clear(&mut self, bbox: Option<&GF_IRect>,
        color: GF_Color) -> Result<(), EvgError> {
        evg_result!(gf_evg_surface_clear(self.0.as_ptr(),
            bbox.map_or(null_mut(), |bbox| bbox as *const _ as _), color))
    }

    pub fn set_clipper(&mut self, clip: Option<&GF_IRect>) -> Result<(), EvgError> {
        evg_result!(gf_evg_surface_set_clipper(self.0.as_ptr(),
            clip.map_or(null_mut(), |clip| clip as *const _ as _)))
    }

    pub fn set_raster_level(&mut self, level: GF_RasterQuality) {
        unsafe { gf_evg_surface_set_raster_level(self.0.as_ptr(), level) };
    }

    pub fn set_composite_mode(&mut self, mode: GF_EVGCompositeMode) {
        unsafe { gf_evg_surface_set_composite_mode(self.0.as_ptr(), mode) };
    }

    pub fn set_matrix(&mut self, mat: Option<&GF_Matrix2D>) {
        evg_debug!(gf_evg_surface_set_matrix(self.0.as_ptr(),
            mat.map_or(null_mut(), |mat| mat as *const _ as _)));
    }
}

pub struct Pixmap { data: Vec<u8>,
    width: u32, height: u32, format: GF_PixelFormat
}

impl GF_PixelFormat {
    pub fn bpp(&self) -> Option<u32> {
        Some(match self {
            GF_PixelFormat::GF_PIXEL_RGBA => 4,
            //GF_PixelFormat::GF_PIXEL_RGB_565 => 2,
            _ => return None
        })
    }
}

impl Pixmap {
    pub fn new(width: u32, height: u32,
        format: GF_PixelFormat) -> Result<Self, EvgError> {
        let len = width * format.bpp().ok_or(EvgError::bad_parameter())? * height;
        Ok(Self { data: vec![0; len as _], width, height, format })
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
    #[test] fn fill_stroke() -> Result<(), Box<dyn std::error::Error>> {
        let mut surf = Surface::new(1024, 512, GF_PixelFormat::GF_PIXEL_RGBA)?;
        let (mut path, mut pens) = (VGPath::new()?, GF_PenSettings::default());
        let mut sten = Stencil::new(GF_StencilType::GF_STENCIL_SOLID)?;

        path.add_rect(GF_Rect { x: 256.into(), y: 384.into(),
            width: 512.into(), height: 256.into() });
        // RUSTDOCFLAGS="-Z unstable-options --nocapture" cargo +nightly test #--doc

        /* path.move_to((rect.x, rect.y));
        path.line_to((rect.x + rect.width, rect.y));
        path.line_to((rect.x + rect.width, rect.y - rect.height));
        path.line_to((rect.x,  rect.y - rect.height));
        path.line_to((rect.x,  rect.y));
        path.close();  path.print_out(); */

        sten.set_color(0xFF0000FF); surf.fill_path(&path, &sten)?;
        sten.set_color(0xAA00FF00); pens.width = 10.into();
        surf.stroke_path(&path, &sten, &pens)?;

        let pixm = surf.end();
        pixm.save_png("target/demo_evg.png")?;  Ok(())
    }
}
