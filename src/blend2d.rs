/****************************************************************
 * $ID: blend2d.rs  	Fri 27 Oct 2023 08:44:33+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#![allow(non_upper_case_globals, clippy::new_without_default, clippy::enum_variant_names)]

//pub mod blend2d  {    // https://blend2d.com
use std::{ffi::CString, marker::PhantomData};
use core::{mem, ptr::{self, null, null_mut}, slice::from_raw_parts};

pub use b2d_ffi::{BLFormat, BLPoint, BLMatrix2D, BLFontMatrix, BLRgba, BLRgba64, BLRgba32,
    BLFillRule, BLStrokeCap, BLStrokeJoin, BLCompOp, BLImageScaleFilter, BLRectI, BLRect,
    BLBox, BLLine, BLArc, BLCircle, BLEllipse, BLTriangle, BLRoundRect, BLHitTest,
    BLLinearGradientValues, BLRadialGradientValues, BLConicGradientValues,
};

#[cfg(feature = "b2d_sfp")] #[allow(non_camel_case_types)] type f64 = f32;
#[cfg(feature = "b2d_sfp")] type F32 = core::primitive::f64; // XXX: API wrapper differ in f32
#[cfg(not(feature = "b2d_sfp"))] type F32 = f32;

#[allow(unused, non_camel_case_types, non_snake_case)] // blend2d_bindings
mod b2d_ffi { include!("../target/bindings/blend2d.rs"); }  use b2d_ffi::*;
// concat!(env!("OUT_DIR"), "/blend2d.rs")  // BGEN_DIR

#[allow(unused)] /*#[macro_export] */macro_rules! safe_dbg {
    //($v:expr$(,$g:expr)?) => { unsafe { $v } };
    ($v:expr,$g:expr) => { match unsafe { $v } { // as u32
        //eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($v), &res);
        res => { if res != $g { dbg!(res); } res } } };
    ($v:expr) => { safe_dbg!($v, 0) };
}

macro_rules! bl_result {
    ($v:expr) => {{ let res = unsafe { $v };
        if res == BLResultCode::BL_SUCCESS as u32 { Ok(()) } else { Err(BLErr(res)) }
    }};
}

macro_rules! bl_debug {
    ($v:expr) => {{ let result = bl_result!($v); debug_assert!(result.is_ok()); }};
}

/// SAFETY: This helper is private and used only for Blend2D C structs
/// whose documented default/initial state is the all-zero bit pattern.
/// BLObjectCore: https://blend2d.com/doc/group__bl__object.html
fn object_init<T>() -> T { unsafe { mem::zeroed() } }

pub struct BLContext(BLContextCore, Option<BLImage>);
impl Drop for BLContext {
    fn drop(&mut self) { bl_debug!(bl_context_destroy(&mut self.0)); }
}

impl BLContext { //  https://blend2d.com/doc/group__bl__rendering.html
    pub fn new(w: u32, h: u32, fmt: BLFormat) -> Result<Self, BLErr> {
        Self::from_image(BLImage::new(w, h, fmt)?)
    }
    pub fn from_image(mut img: BLImage) -> Result<Self, BLErr> {
        let mut ctx = object_init();
        bl_result!(bl_context_init_as(&mut ctx, &mut img.0, null()))?;
        Ok(Self(ctx, Some(img)))
    }
    pub fn get_target_image(&self) -> &BLImage {
        self.1.as_ref().expect("BLContext always owns its target image")
    }
    pub fn get_target_size(&self) -> BLSizeI {
        let mut sz = (0., 0.).into();
        bl_debug!(bl_context_get_target_size(&self.0, &mut sz));
        (sz.w as u32, sz.h as u32).into()
    }

    pub fn fill_geometry<T: B2DGeometry>(&mut self, geom: &T) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_geometry(&mut self.0, T::GEOM_T, geom.as_ptr()))
    }
    pub fn fill_all_rgba32(&mut self, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_all_rgba32(&mut self.0, color.value))
    }
    pub fn fill_geometry_rgba32<T: B2DGeometry>(&mut self,
        geom: &T, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_geometry_rgba32(&mut self.0, T::GEOM_T,
            geom.as_ptr(), color.value))
    }
    pub fn fill_geometry_ext<T: B2DGeometry>(&mut self,
        geom: &T, style: &dyn B2DStyle) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_geometry_ext(&mut self.0, T::GEOM_T,
            geom.as_ptr(), style.as_ptr()))
    }

    pub fn set_fill_style(&mut self, style: &dyn B2DStyle) {
        bl_debug!(bl_context_set_fill_style(&mut self.0, style.as_ptr()));
    }
    pub fn set_fill_rule(&mut self, fill_rule: BLFillRule) {
        bl_debug!(bl_context_set_fill_rule(&mut self.0, fill_rule));
    }
    pub fn set_fill_alpha(&mut self, alpha: f64) {
        bl_debug!(bl_context_set_fill_alpha(&mut self.0, alpha as _));
    }

    pub fn set_stroke_alpha(&mut self, alpha: f64) {
        bl_debug!(bl_context_set_stroke_alpha(&mut self.0, alpha as _));
    }
    pub fn set_stroke_style(&mut self, style: &dyn B2DStyle) {
        bl_debug!(bl_context_set_stroke_style(&mut self.0, style.as_ptr()));
    }
    pub fn set_stroke_width(&mut self, width: f64) {
        bl_debug!(bl_context_set_stroke_width(&mut self.0, width as _));
    }
    pub fn set_stroke_caps(&mut self, caps: BLStrokeCap) {
        bl_debug!(bl_context_set_stroke_caps(&mut self.0, caps));
    }
    pub fn set_stroke_caps2(&mut self, sc: BLStrokeCap, ec: BLStrokeCap) {
        bl_debug!(bl_context_set_stroke_cap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_START, sc));
        bl_debug!(bl_context_set_stroke_cap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_END, ec));
    }
    pub fn set_stroke_join(&mut self, join: BLStrokeJoin) {
        bl_debug!(bl_context_set_stroke_join(&mut self.0, join));
    }
    pub fn set_stroke_miter_limit(&mut self, limit: f64) {
        bl_debug!(bl_context_set_stroke_miter_limit(&mut self.0, limit as _));
    }
    pub fn set_stroke_dash(&mut self, offset: f64,
        dash: &[f64]) -> Result<(), BLErr> {
        let dash = BLArrayFP::new(dash)?;
        bl_result!(bl_context_set_stroke_dash_offset(&mut self.0, offset as _))?;
        bl_result!(bl_context_set_stroke_dash_array(&mut self.0, &dash.0))
    }
    pub fn set_stroke_options(&mut self, options: &BLStrokeOptions) {
        bl_debug!(bl_context_set_stroke_options(&mut self.0, &options.0));
    }

    pub fn stroke_geometry<T: B2DGeometry>(&mut self, geom: &T) -> Result<(), BLErr> {
        bl_result!(bl_context_stroke_geometry(&mut self.0, T::GEOM_T, geom.as_ptr()))
    }
    pub fn stroke_geometry_rgba32<T: B2DGeometry>(&mut self,
        geom: &T, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_stroke_geometry_rgba32(&mut self.0, T::GEOM_T,
            geom.as_ptr(), color.value))
    }
    pub fn stroke_geometry_ext<T: B2DGeometry>(&mut self,
        geom: &T, style: &dyn B2DStyle) -> Result<(), BLErr> {
        bl_result!(bl_context_stroke_geometry_ext(&mut self.0, T::GEOM_T,
            geom.as_ptr(), style.as_ptr()))
    }

    pub fn fill_utf8_text_d_rgba32(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_utf8_text_d_rgba32(&mut self.0, &origin, &font.0,
            text.as_ptr().cast(), text.len(), color.value))
    }
    pub fn fill_utf8_text_d_ext(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_utf8_text_d_ext(&mut self.0, &origin, &font.0,
            text.as_ptr().cast(), text.len(), style.as_ptr()))
    }

    pub fn stroke_utf8_text_d_rgba32(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_stroke_utf8_text_d_rgba32(&mut self.0, &origin, &font.0,
            text.as_ptr().cast(), text.len(), color.value))
    }
    pub fn stroke_utf8_text_d_ext(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) -> Result<(), BLErr> {
        bl_result!(bl_context_stroke_utf8_text_d_ext(&mut self.0, &origin, &font.0,
            text.as_ptr().cast(), text.len(), style.as_ptr()))
    }

    pub fn blit_image_d(&mut self, origin: BLPoint,
        img: &BLImage, img_area: &BLRectI) -> Result<(), BLErr> {
        bl_result!(bl_context_blit_image_d(&mut self.0, &origin, &img.0, img_area))
    }
    pub fn blit_scaled_image_d(&mut self, rect: &BLRect,
        img: &BLImage, img_area: &BLRectI) -> Result<(), BLErr> {
        bl_result!(bl_context_blit_scaled_image_d(&mut self.0, rect, &img.0, img_area))
    }

    pub fn fill_mask_d_rgba32(&mut self, origin: BLPoint,
        mask: &BLImage, area: &BLRectI, color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_mask_d_rgba32(&mut self.0,
            &origin, &mask.0, area, color.value))
    }
    pub fn fill_mask_d_ext(&mut self, origin: BLPoint,
        mask: &BLImage, area: &BLRectI, style: &dyn B2DStyle) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_mask_d_ext(&mut self.0,
            &origin, &mask.0, area, style.as_ptr()))
    }

    pub fn clip_to_rect_d(&mut self, clip: &BLRect) -> Result<(), BLErr> {
        bl_result!(bl_context_clip_to_rect_d(&mut self.0, clip))
    }
    pub fn restore_clipping(&mut self) -> Result<(), BLErr> {
        bl_result!(bl_context_restore_clipping(&mut self.0))
    }

    pub fn fill_rect_i_rgba32(&mut self, rect: &BLRectI,
        color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_context_fill_rect_i_rgba32(&mut self.0, rect, color.value))
    }
    pub fn clear_rect_d(&mut self, rect: &BLRect) -> Result<(), BLErr> {
        bl_result!(bl_context_clear_rect_d(&mut self.0, rect))
    }
    pub fn clear_all(&mut self) -> Result<(), BLErr> {
        bl_result!(bl_context_clear_all(&mut self.0))
    }

    pub fn user_to_meta(&mut self) { bl_debug!(bl_context_user_to_meta(&mut self.0)); }
    pub fn user_transform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::identity();
        bl_debug!(bl_context_get_user_transform(&self.0, &mut mat)); mat
    }
    pub fn meta_transform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::identity();
        bl_debug!(bl_context_get_meta_transform(&self.0, &mut mat)); mat
    }
    pub fn final_transform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::identity();
        bl_debug!(bl_context_get_final_transform(&self.0, &mut mat)); mat
    }
    pub fn reset_transform(&mut self, mat: Option<&BLMatrix2D>) {
        if let Some(mat) = mat {
            bl_debug!(bl_context_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            bl_debug!(bl_context_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }
    }
    pub fn apply_transform(&mut self, mat: &BLMatrix2D) {
        bl_debug!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }

    pub fn scale(&mut self, sl: BLVec2D) {
        let values = [sl.0, sl.1];
        bl_debug!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_SCALE, values.as_ptr().cast()));
    }
    pub fn translate(&mut self, pos: BLPoint) {
        let values = [pos.x, pos.y];
        bl_debug!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSLATE, values.as_ptr().cast()));
    }
    pub fn rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let values = [angle, origin.x, origin.y];
        bl_debug!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_ROTATE_PT, values.as_ptr().cast()));
    }

    pub fn set_comp_op(&mut self, cop: BLCompOp) {
        bl_debug!(bl_context_set_comp_op(&mut self.0, cop));
    }
    pub fn set_global_alpha(&mut self, alpha: f64) {
        bl_debug!(bl_context_set_global_alpha(&mut self.0, alpha as _));
    }
    /// value: BLRenderingQuality, BLGradientQuality, BLPatternQuality
    pub fn set_hint(&mut self, hint: BLContextHint, value: u32) {
        bl_debug!(bl_context_set_hint(&mut self.0, hint, value));
    }

    pub fn restore(&mut self) -> Result<(), BLErr> {
        bl_result!(bl_context_restore(&mut self.0, null()))
    }
    pub fn save (&mut self) -> Result<(), BLErr> {
        bl_result!(bl_context_save(&mut self.0, null_mut()))
    }
    pub fn flush(&mut self) -> Result<(), BLErr> {
        bl_result!(bl_context_flush(&mut self.0,
            BLContextFlushFlags::BL_CONTEXT_FLUSH_SYNC))
    }
    pub fn end(mut self) -> Result<BLImage, BLErr> {
        bl_result!(bl_context_end(&mut self.0))?;
        Ok(self.1.take().expect("BLContext always owns its target image"))
    }

    pub fn show_rtinfo() -> Result<(), BLErr> {
        let mut  info: BLRuntimeBuildInfo  = object_init();
        let mut sinfo: BLRuntimeSystemInfo = object_init();

        bl_result!(bl_runtime_query_info(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_BUILD,
            &mut  info as *mut _ as _))?;
        bl_result!(bl_runtime_query_info(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
            &mut sinfo as *mut _ as _))?;

        println!(r#"Build & System Information: {{
  Version: {}.{}.{}
  BuildMode: Embed
  BuildType: {}
    RuntimeCpuFeatures: {:#x}
   BaselineCpuFeatures: {:#x}
  SupportedCpuFeatures: {:#x}
  CpuArch: {} [{} bit]
  OperatingSystem: {}
  Compiler: {}
  MaxImageSize: {}
  MaxThreadCount: {}
     ThreadCount: {}
  ThreadStackSize: {}
  AllocationGranularity: {}
}}"#,       info.major_version, info.minor_version, info.patch_version,
            if info.build_type == BLRuntimeBuildType::BL_RUNTIME_BUILD_TYPE_DEBUG as u32 {
                "Debug" } else { "Release" },
            sinfo.cpu_features, info.baseline_cpu_features, info.supported_cpu_features,
            std::env::consts::ARCH, mem::size_of::<*const i32>() * 8, std::env::consts::OS,
            info.compiler_info.iter().map(|i| char::from(*i as u8)).collect::<String>(),
            info.max_image_size, info.max_thread_count, sinfo.thread_count,
            sinfo.thread_stack_size, sinfo.allocation_granularity);     Ok(())
    }
}

pub struct BLImage(BLImageCore);
impl Drop for BLImage { fn drop(&mut self) { bl_debug!(bl_image_destroy(&mut self.0)); } }

impl BLImage { //  https://blend2d.com/doc/group__bl__imaging.html
    pub fn new(w: u32, h: u32, fmt: BLFormat) -> Result<Self, BLErr> {
        let mut img = object_init();
        //bl_result!(bl_image_create(&mut img, w as _, h as _, fmt))?;
        bl_result!(bl_image_init_as(&mut img, w as _, h as _, fmt))?;
        Ok(Self(img))
    }

    /// Creates an image backed directly by `buf`, without copying or freeing it.
    /// `buf` must cover every complete stride, including last-row padding.
    ///
    /// # Safety
    ///
    /// `buf` must remain allocated at the same address and must not be accessed
    /// elsewhere until the image and all Blend2D objects derived from it are dropped.
    /// The image layout calculations must fit in `u32`.
    pub unsafe fn from_buffer(w: u32, h: u32, fmt: BLFormat,
        buf: &mut [u8], stride:  u32) -> Result<BLImage, BLErr> {
        if buf.len() < (stride * h) as _ { return Err(BLErr::invalid_value()); }

        let (mut img, data) = (object_init(), buf.as_mut_ptr());
        bl_result!(bl_image_init_as_from_data(&mut img, w as _, h as _, fmt, data as _,
            stride as _, BLDataAccessFlags::BL_DATA_ACCESS_RW, None, null_mut()))?;
        Ok(Self(img))
    }

    pub fn to_rgba_inplace(&mut self) -> Result<(), BLErr> { // 0xAARRGGBB -> 0xAABBGGRR
        let imgd = self.data();
        if  imgd.format != BLFormat::BL_FORMAT_PRGB32 as u32 {
            return Err(BLErr::invalid_value());
        }

        let mut di = object_init::<BLFormatInfo>();
        bl_result!(bl_format_info_query(&mut di, BLFormat::BL_FORMAT_PRGB32))?;
        let si = unsafe { ptr::read(&di) };

        let rgba_opt = unsafe { &mut di.__bindgen_anon_1.__bindgen_anon_2 };
        rgba_opt.r_shift =  0; rgba_opt.g_shift =  8; rgba_opt.b_shift = 16;

        let mut conv = object_init(); // XXX: BLPixelConverter
        bl_result!(bl_pixel_converter_create(&mut conv, &di, &si,
            BLPixelConverterCreateFlags::BL_PIXEL_CONVERTER_CREATE_NO_FLAGS))?;
        let result = bl_result!(bl_pixel_converter_convert(&conv,
            imgd.pixel_data,  imgd.stride, imgd.pixel_data, imgd.stride,
            imgd.size.w as _, imgd.size.h as _, null()));
        let cleanup = bl_result!(bl_pixel_converter_destroy(&mut conv));
        result.and(cleanup)
    }

    fn data(&self) -> b2d_ffi::BLImageData {
        let mut data = object_init();
        bl_debug!(bl_image_get_data(&self.0, &mut data));
        data
    }
    /// Returns contiguous rows; negative-stride images cannot be represented
    /// by one forward Rust slice and return `None`.
    pub fn pixels(&self) -> Option<&[u8]> {
        let data = self.data();
        if  data.stride < 0 { return None }
        let len = data.stride as usize * data.size.h as usize;
        if  len == 0 { return Some(&[]); }
        let ptr = ptr::NonNull::new(data.pixel_data.cast())?;
        // SAFETY: Blend2D owns this pixel region and the returned slice borrows
        // `self`, preventing mutation or destruction of the image while in use.
        Some(unsafe { from_raw_parts(ptr.as_ptr(), len) })
    }
    pub fn stride(&self) -> isize { self.data().stride }
    pub fn height(&self) -> u32 { self.data().size.h as _ }
    pub fn width (&self) -> u32 { self.data().size.w as _ }

    pub fn read_from_data(data: &[u8]) -> Result<Self, BLErr> {
        let mut img = object_init();
        bl_result!(bl_image_init(&mut img))?;
        let mut img = Self(img);

        bl_result!(bl_image_read_from_data(&mut img.0,
            data.as_ptr() as _, data.len(), null()))?;
        Ok(img)
    }
    pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut img = object_init();
        bl_result!(bl_image_init(&mut img))?;
        let mut img = Self(img);

        let file = CString::new(file).map_err(|_|
            BLErr(BLResultCode::BL_ERROR_INVALID_STRING as _))?;
        bl_result!(bl_image_read_from_file(&mut img.0, file.as_ptr(), null()))?;
        Ok(img)
    }

    pub fn scale(&mut self, src: &BLImage,
        dst_w: u32, dst_h: u32, filter: BLImageScaleFilter) -> Result<(), BLErr> {
        bl_result!(bl_image_scale(&mut self.0, &src.0, &(dst_w, dst_h).into(), filter))
    }

    pub fn write_to_file<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), BLErr> {
        let cstr = CString::new(file).map_err(|_|
            BLErr(BLResultCode::BL_ERROR_INVALID_STRING as _))?;
        bl_result!(bl_image_write_to_file(&self.0, cstr.as_ptr(), null()))?;    Ok(())
    }
}

#[path = "text_path.rs"] mod text_path;
pub use text_path::{TextPathOptions, TextPathPaint};
#[path = "clipping_mask.rs"] mod clipping_mask;

// Future shaping options (direction, script, language, and OpenType features)
// belong beside this RAII wrapper if BLFont's defaults are no longer sufficient.
pub struct BLGlyphBuffer(BLGlyphBufferCore);
impl Drop for BLGlyphBuffer {
    fn drop(&mut self) { bl_debug!(bl_glyph_buffer_destroy(&mut self.0)); }
}

impl BLGlyphBuffer {
    pub fn new(text: &str) -> Result<Self, BLErr> {
        let mut core = object_init();
        bl_debug!(bl_glyph_buffer_init(&mut core));
        let mut buffer = Self(core);

        bl_result!(bl_glyph_buffer_set_text(&mut buffer.0, text.as_ptr().cast(),
            text.len(), BLTextEncoding::BL_TEXT_ENCODING_UTF8))?;
        Ok(buffer)
    }

    pub fn items(&self) -> impl Iterator<Item = (BLGlyphId, &BLGlyphPlacement)> + '_ {
        // SAFETY: `self.0` is a live glyph buffer for the duration of the call.
        let len = unsafe { bl_glyph_buffer_get_size(&self.0) };
        let (ids, placements): (&[BLGlyphId], &[BLGlyphPlacement]) =
            if len == 0 { (&[], &[]) } else { unsafe {(
                // SAFETY: after successful shaping Blend2D exposes parallel glyph
                // and placement arrays of `len` elements owned by this buffer.
                from_raw_parts(bl_glyph_buffer_get_content(&self.0), len),
                from_raw_parts(bl_glyph_buffer_get_placement_data(&self.0), len),
            )} };
        ids.iter().copied().zip(placements)
    }
}

impl BLFontMatrix {
    // Keep this linear-only: glyph outlines already receive this font matrix.
    // A future writing-mode implementation may expose inline and block vectors.
    pub(crate) fn map_point(&self, pt: BLPoint) -> BLPoint {
        // SAFETY: `bl_font_get_matrix` initializes this union member.
        let [m00, m01, m10, m11] = unsafe { *self.__bindgen_anon_1.m };
        BLPoint { x: pt.x * m00 + pt.y * m10, y: pt.x * m01 + pt.y * m11 }
    }
}

//  https://blend2d.com/doc/group__bl__text.html
pub struct BLFont<'a>(BLFontCore, PhantomData<&'a [u8]>);
impl<'a> Drop for BLFont<'a> { fn drop(&mut self) { bl_debug!(bl_font_destroy(&mut self.0)); } }
impl<'a> BLFont<'a> {   // TODO: a bunch of interfaces need to be regarded
    pub fn new(face: &BLFontFace<'a>, size: f32) -> Result<Self, BLErr> {
        let mut font = object_init();
        bl_result!(bl_font_init(&mut font))?;
        let mut font = Self(font, PhantomData);

        bl_result!(bl_font_create_from_face(&mut font.0, &face.0, size))?;
        //bl_result!(bl_font_create_from_face_with_settings(&mut font.0, &face.core, size,
        //    feature_settings, variation_settings))?;
        Ok(font)
    }
    pub fn shape(&self, text: &str) -> Result<BLGlyphBuffer, BLErr> {
        let mut buffer = BLGlyphBuffer::new(text)?;
        bl_result!(bl_font_shape(&self.0, &mut buffer.0))?;
        Ok(buffer)
    }
    pub fn get_matrix(&self) -> BLFontMatrix {
        let mut matrix = object_init();
        bl_debug!(bl_font_get_matrix(&self.0, &mut matrix));
        matrix
    }
}

// Blend2D references external font bytes instead of copying them.
pub struct BLFontFace<'a>(BLFontFaceCore, PhantomData<&'a [u8]>);
impl<'a> Drop for BLFontFace<'a> {
    fn drop(&mut self) { bl_debug!(bl_font_face_destroy(&mut self.0)); }
}

impl<'a> BLFontFace<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, BLErr> {
        let mut core = object_init();
        bl_result!(bl_font_face_init(&mut core))?;
        let mut face = Self(core, PhantomData);

        let mut fdata = object_init();
        bl_result!(bl_font_data_init(&mut fdata))?;
        let mut fdata = BLFontData(fdata);

        bl_result!(bl_font_data_create_from_data(&mut fdata.0,
            data.as_ptr() as _, data.len(), None, null_mut()))?;
        bl_result!(bl_font_face_create_from_data(&mut face.0, &fdata.0, 0))?;
        Ok(face)
    }

    pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut core = object_init();
        bl_result!(bl_font_face_init(&mut core))?;
        let mut face = Self(core, PhantomData);

        let cstr = CString::new(file).map_err(|_|
            BLErr(BLResultCode::BL_ERROR_INVALID_STRING as _))?;
        bl_result!(bl_font_face_create_from_file(&mut face.0, cstr.as_ptr(),
            BLFileReadFlags::BL_FILE_READ_NO_FLAGS))?;
        Ok(face)
    }
}

struct BLFontData(BLFontDataCore);
impl Drop for BLFontData {
    fn drop(&mut self) { bl_debug!(bl_font_data_destroy(&mut self.0)); }
}

pub struct BLStrokeOptions(BLStrokeOptionsCore);
impl Drop for BLStrokeOptions {
    fn drop(&mut self) { bl_debug!(bl_stroke_options_destroy(&mut self.0)); }
}
impl BLStrokeOptions {
    pub fn new() -> Self {
        let mut option = object_init();
        bl_debug!(bl_stroke_options_init(&mut option));
        Self(option)
    }

    pub fn set_width(&mut self, width: f64) { self.0.width = width as _; }
    pub fn set_miter_limit(&mut self, miter: f64) { self.0.miter_limit = miter as _; }

    pub fn set_options(&mut self, sc: BLStrokeCap, ec: BLStrokeCap,
        join: BLStrokeJoin/*, to: BLStrokeTransformOrder*/) {   // XXX:
        // SAFETY: bindgen represents the active public option fields as a union;
        // Blend2D initializes this member in `bl_stroke_options_init`.
        let options = unsafe { &mut self.0.__bindgen_anon_1.__bindgen_anon_1 };
        options.start_cap = sc as _;     options.end_cap = ec as _;
        options.join = join as _;       //options.transform_order = to as _;
    }

    pub fn set_dash(&mut self, offset: f64, dash: &[f64]) -> Result<(), BLErr> {
        self.0.dash_offset = offset as _;
        let dash = BLArrayFP::new(dash)?;

        //bl_result!(bl_array_assign_deep(&mut self.0.dash_array, &dash.0))
        bl_result!(bl_array_assign_deep(
            &mut (&mut self.0.__bindgen_anon_2.dash_array)._base, &dash.0))
    }
}

impl Default for BLApproximationOptions { fn default() -> Self { Self::new() } }
impl BLApproximationOptions {
    fn new() -> Self { Self {
        flatten_mode: BLFlattenMode::BL_FLATTEN_MODE_DEFAULT as _,
        offset_mode: BLOffsetMode::BL_OFFSET_MODE_DEFAULT as _, reserved_flags: [0; 6],
        flatten_tolerance: 0.20, simplify_tolerance: 0.05, offset_parameter: 0.414_213_56
    } }
}

struct BLArrayFP(BLArrayCore);
impl Drop for BLArrayFP {
    fn drop(&mut self) { bl_debug!(bl_array_destroy(&mut self.0)); }
}
impl BLArrayFP {
    pub fn new(data: &[f64]) -> Result<Self, BLErr> {
        let mut array = object_init();
        if cfg!(feature = "b2d_sfp") {
            bl_result!(bl_array_init(&mut array,
                BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT32))?;
            let mut array = Self(array);
            bl_result!(bl_array_reserve(&mut array.0, data.len()))?;

            if mem::size_of::<f64>() == 4 {     // re-defined f64 = f32
                bl_result!(bl_array_assign_data(&mut array.0,
                    data.as_ptr() as _, data.len()))?;
            } else {
                for value in data {
                    bl_result!(bl_array_append_f32(&mut array.0, *value as _))?;
                }
            }   Ok(array)
        } else {
            bl_result!(bl_array_init(&mut array,
                BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT64))?;
            let mut array = Self(array);
            bl_result!(bl_array_reserve(&mut array.0, data.len()))?;

            if mem::size_of::<f64>() == 8 {
                bl_result!(bl_array_assign_data(&mut array.0,
                    data.as_ptr() as _, data.len()))?;
            } else {
                for value in data {
                    bl_result!(bl_array_append_f64(&mut array.0, *value as _))?;
                }
            }   Ok(array)
        }
    }
}

impl Default for BLMatrix2D { fn default() -> Self { Self::identity() } }
impl Clone   for BLMatrix2D { fn clone(&self) -> Self { Self::new(self.get_values()) } }

impl BLMatrix2D { //  https://blend2d.com/doc/structBLMatrix2D.html
    pub fn identity() -> Self { Self::new([1., 0., 0., 1., 0., 0.]) }

    pub fn new(values: [f64; 6]) -> Self {
        let mut mat: Self = object_init();
        // SAFETY: `[f64; 6]` is the active matrix representation of `BLMatrix2D`;
        // all bit patterns of its scalar elements are valid.
        unsafe { *mat.__bindgen_anon_1.m = values; }    mat
    }
    // SAFETY: `BLMatrix2D` is always constructed with its matrix member active.
    pub fn get_values(&self) -> [f64; 6] { unsafe { *self.__bindgen_anon_1.m } }
    pub fn set_translation(&mut self, pos: BLPoint) {
        *self = Self::new([1., 0., 0., 1., pos.x, pos.y]);
    }
    pub fn set_scaling(&mut self, sl: BLVec2D) {
        *self = Self::new([sl.0, 0., 0., sl.1, 0., 0.]);
    }
    pub fn set_skewing(&mut self, sk: BLVec2D) {
        bl_debug!(bl_matrix2d_set_skewing(self, sk.0, sk.1));
    }
    pub fn set_rotation(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        bl_debug!(bl_matrix2d_set_rotation(self, angle, origin.x, origin.y));
    }

    pub fn translate(&mut self, pos: BLPoint) {
        let values = [pos.x, pos.y];
        bl_debug!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_TRANSLATE,
            values.as_ptr().cast()));
    }
    pub fn scale(&mut self, sl: BLVec2D) {
        let values = [sl.0, sl.1];
        bl_debug!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_SCALE,
            values.as_ptr().cast()));
    }
    pub fn skew(&mut self, sk: BLVec2D) {
        let values = [sk.0, sk.1];
        bl_debug!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_SKEW,
            values.as_ptr().cast()));
    }
    pub fn rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let values = [angle, origin.x, origin.y];
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_ROTATE_PT, values.as_ptr().cast()));
    }

    pub fn post_translate(&mut self, pos: BLPoint) {
        let values = [pos.x, pos.y];
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_TRANSLATE, values.as_ptr().cast()));
    }
    pub fn post_scale(&mut self, sl: BLVec2D) {
        let values = [sl.0, sl.1];
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_SCALE, values.as_ptr().cast()));
    }
    pub fn post_skew(&mut self, sk: BLVec2D) {
        let values = [sk.0, sk.1];
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_SKEW, values.as_ptr().cast()));
    }
    pub fn post_rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let values = [angle, origin.x, origin.y];
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_ROTATE_PT, values.as_ptr().cast()));
    }

    /*  | a b 0 |
        | c d 0 |
        | e f 1 | */
    /// A' = B * A (new = other * self)
    pub fn transform(&mut self, mat: &BLMatrix2D) {
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }
    pub fn post_transform(&mut self, mat: &BLMatrix2D) {
        bl_debug!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_TRANSFORM, mat as *const _ as _));
    }
    pub fn reset(&mut self, mat: Option<&BLMatrix2D>) {
        if let Some(mat) = mat {
            bl_debug!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            bl_debug!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }
    }
    pub fn invert(&mut self) -> Result<(), BLErr> {
        bl_result!(bl_matrix2d_invert(self, self))
    }

    pub fn get_scaling(&self) -> BLVec2D {
        // SAFETY: `BLMatrix2D` is always constructed with its matrix member active.
        let mat = unsafe { &self.__bindgen_anon_1.__bindgen_anon_1 };
        (mat.m00, mat.m10)
    }

    pub fn map_point(&self, pt: BLPoint) -> BLPoint {
        let mut npt = BLPoint::new();
        bl_debug!(bl_matrix2d_map_pointd_array(self, &mut npt, &pt, 1));
        npt
    }
    pub fn map_point_array(&self, pts: &mut [BLPoint]) {
        bl_debug!(bl_matrix2d_map_pointd_array(self,
            pts.as_mut_ptr(), pts.as_ptr(), pts.len()));
    }
}
pub type BLVec2D = (f64, f64);     // (f64, f64), BLSize/BLPoint

/// https://blend2d.com/doc/classBLPath.html
#[repr(transparent)] pub struct BLPath(BLPathCore);
impl Drop for BLPath { fn drop(&mut self) { bl_debug!(bl_path_destroy(&mut self.0)); } }
impl BLPath {
    pub fn new() -> Self {
        let mut path = object_init();
        bl_debug!(bl_path_init(&mut path));
        Self(path)
    }

    pub fn move_to(&mut self, end: BLPoint) {
        bl_debug!(bl_path_move_to(&mut self.0, end.x, end.y));
    }
    pub fn line_to(&mut self, end: BLPoint) {
        bl_debug!(bl_path_line_to(&mut self.0, end.x, end.y));
    }
    pub fn quad_to(&mut self, cp: BLPoint, end: BLPoint) {
        bl_debug!(bl_path_quad_to(&mut self.0, cp.x, cp.y, end.x, end.y));
    }
    pub fn cubic_to(&mut self, c1: BLPoint, c2: BLPoint, end: BLPoint) {
        bl_debug!(bl_path_cubic_to(&mut self.0,
            c1.x, c1.y, c2.x, c2.y, end.x, end.y));
    }

    pub fn arc_to(&mut self, center: BLPoint, radii: BLVec2D,
        start: f64, sweep: f64) -> Result<(), BLErr> {
        bl_result!(bl_path_arc_to(&mut self.0, center.x, center.y,
            radii.0, radii.1, start as _, sweep as _, false)) // force_move_to
    }
    pub fn elliptic_arc_to(&mut self, radii: BLVec2D,    // svg_arc_to
        x_rot: f64, large: bool, sweep: bool, end: BLPoint) -> Result<(), BLErr> {
        //  Adds an elliptic arc to the path that follows the SVG specification.
        //  https://www.w3.org/TR/SVG/paths.html#PathDataEllipticalArcCommands
        bl_result!(bl_path_elliptic_arc_to(&mut self.0,
            radii.0, radii.1, x_rot as _, large, sweep, end.x, end.y))
    }
    pub fn arc_quadrant_to(&mut self, corner: BLPoint,
        end: BLPoint) -> Result<(), BLErr> {
        bl_result!(bl_path_arc_quadrant_to(&mut self.0, corner.x, corner.y, end.x, end.y))
    }
    pub fn poly_to(&mut self, poly: &[BLPoint]) {
        bl_debug!(bl_path_poly_to(&mut self.0, poly.as_ptr(), poly.len()));
    }

    pub fn close(&mut self) { bl_debug!(bl_path_close(&mut self.0)); }
    //pub fn clear(&mut self) { bl_debug!(bl_path_clear(&mut self.0)); }
    pub fn reset(&mut self) { bl_debug!(bl_path_reset(&mut self.0)); }

    pub fn transform(&mut self, mat: &BLMatrix2D) -> Result<(), BLErr> {
        bl_result!(bl_path_transform(&mut self.0, null(), mat))
    }

    pub fn reserve(&mut self, capacity: u32) -> Result<(), BLErr> {
        bl_result!(bl_path_reserve(&mut self.0, capacity as _))
    }
    pub fn get_size(&self) -> u32 {
        // SAFETY: `self.0` is a live Blend2D path for the duration of the call.
        unsafe { bl_path_get_size(&self.0) as _ }
    }
    pub fn get_last_vertex(&self) -> Result<BLPoint, BLErr> {
        let mut pt = BLPoint { x: 0.0, y: 0.0 };
        bl_result!(bl_path_get_last_vertex(&self.0, &mut pt))?;
        Ok(pt)
    }
    pub fn get_bounding_box(&self) -> Result<BLBox, BLErr> {
        let mut bbox = BLBox::new();
        bl_result!(bl_path_get_bounding_box(&self.0, &mut bbox))?;
        Ok(bbox)
    }
    pub fn hit_test(&self, pt: BLPoint, fill_rule: BLFillRule) -> BLHitTest {
        // SAFETY: both pointers refer to live values for the duration of the call.
        unsafe { bl_path_hit_test(&self.0, &pt, fill_rule) }
    }

    pub fn add_geometry<T: B2DGeometry>(&mut self, geom: &T,
        mat: &BLMatrix2D) -> Result<(), BLErr> {
        bl_result!(bl_path_add_geometry(&mut self.0, T::GEOM_T,
            geom.as_ptr(), mat, BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW))
    }

    pub fn add_path(&mut self, path: &BLPath) -> Result<(), BLErr> {
        bl_result!(bl_path_add_path(&mut self.0, &path.0, null()))
    }
    pub fn add_transformed_path(&mut self, path: &BLPath,
        mat: &BLMatrix2D) -> Result<(), BLErr> {
        bl_result!(bl_path_add_transformed_path(&mut self.0, &path.0, null(), mat))
    }
    pub fn add_stroked_path(&mut self, path: &BLPath,
        options: &BLStrokeOptions, approx: &BLApproximationOptions) -> Result<(), BLErr> {
        bl_result!(bl_path_add_stroked_path(
            &mut self.0, &path.0, null(), &options.0, approx))
    }

    pub fn add_rect(&mut self, rect: &BLRect) {
        bl_debug!(bl_path_add_rect_d(&mut self.0, rect,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }
    pub fn add_box(&mut self, bbox: &BLBox) {
        bl_debug!(bl_path_add_box_d(&mut self.0, bbox,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    pub fn iter(&self) -> BLPathIter<'_> {
        // SAFETY: `self.0` is a live Blend2D path for the duration of the call.
        let len = unsafe { bl_path_get_size(&self.0) };
        let (cmd, vtx) = if len == 0 { (&[][..], &[][..]) } else { unsafe {
            // SAFETY: Blend2D stores parallel command/vertex arrays with `len` entries,
            // and the iterator borrow prevents path mutation or drop.
                (from_raw_parts(bl_path_get_command_data(&self.0), len),
                 from_raw_parts(bl_path_get_vertex_data(&self.0), len))
        } };
        BLPathIter { cmd, vtx, idx: 0 }
    }
}

pub enum BLPathItem {
    QuadTo (BLPoint, BLPoint),
    //ConicTo(BLPoint, f64, BLPoint),
    CubicTo(BLPoint, BLPoint, BLPoint),
    MoveTo (BLPoint),  LineTo(BLPoint), Close,
}

pub struct BLPathIter<'a> { cmd: &'a [u8], vtx: &'a [BLPoint], idx: usize, }
impl<'a> Iterator for BLPathIter<'a> {  type Item = BLPathItem;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        let cmd = *self.cmd.get(idx)?;

        let advance = if cmd == BLPathCmd::BL_PATH_CMD_MOVE as u8 ||
            cmd == BLPathCmd::BL_PATH_CMD_ON as u8 ||
            cmd == BLPathCmd::BL_PATH_CMD_CLOSE as u8 { 1
        } else if cmd == BLPathCmd::BL_PATH_CMD_QUAD as u8 { 2
        } else if cmd == BLPathCmd::BL_PATH_CMD_CUBIC as u8 { 3
        } else { self.idx = self.cmd.len(); return None; };

        if  idx + advance > self.vtx.len() {
                 self.idx = self.cmd.len(); return None;
        }   use BLPathItem::*;

        let item = if cmd == BLPathCmd::BL_PATH_CMD_MOVE as u8 {
             MoveTo(self.vtx[idx])
        } else if cmd == BLPathCmd::BL_PATH_CMD_ON as u8 {
             LineTo(self.vtx[idx])
        } else if cmd == BLPathCmd::BL_PATH_CMD_QUAD as u8 {
             QuadTo(self.vtx[idx], self.vtx[idx + 1])
        } else if cmd == BLPathCmd::BL_PATH_CMD_CUBIC as u8 {
            CubicTo(self.vtx[idx], self.vtx[idx + 1], self.vtx[idx + 2])
        } else if cmd == BLPathCmd::BL_PATH_CMD_CLOSE as u8 { Close
        } else { unreachable!() };

        self.idx += advance;
        Some(item)
    }
}

impl BLLine {
    pub fn new(s: BLPoint, e: BLPoint) -> Self {
        Self { x0: s.x, y0: s.y, x1: e.x, y1: e.y }
    }
}
impl BLArc {
    pub fn new(c: BLPoint, r: BLVec2D, start: f64, sweep: f64) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.0, ry: r.1, start: start as _, sweep: sweep as _ }
    }
}
impl BLCircle {
    pub fn new(c: BLPoint, r: f64) -> Self { Self { cx: c.x, cy: c.y, r: r as _ } }
}
impl BLEllipse {
    pub fn new(c: BLPoint, r: BLVec2D) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.0, ry: r.1 }
    }
}
impl BLTriangle {
    pub fn new(a: BLPoint, b: BLPoint, c: BLPoint) -> Self {
        Self { x0: a.x, y0: a.y, x1: b.x, y1: b.y, x2: c.x, y2: c.y }
     }
}
impl BLRoundRect {
    pub fn new(rect: &BLRect, radius: f64) -> Self { Self {
        x: rect.x, y: rect.y, w: rect.w, h: rect.h, rx: radius as _, ry: radius as _
    } }
}

impl BLBox  { pub fn new() -> Self { Self { x0: 0., y0: 0., x1: 0., y1: 0. } } }
impl BLRect { pub fn new() -> Self { Self { x : 0., y : 0.,  w: 0.,  h: 0. } } }
impl From<(BLPoint, BLSize)> for BLRect {
    fn from((lt, sz): (BLPoint, BLSize)) -> Self {
        Self { x: lt.x, y: lt.y, w: sz.w, h: sz.h }
    }
}
impl From<(BLPoint, BLPoint)> for BLRect {
    fn from((lt, rb): (BLPoint, BLPoint)) -> Self {
        Self { x: lt.x, y: lt.y, w: rb.x - lt.x, h: rb.y - lt.y }   // .abs()?
    }
}
impl From<(BLPoint, BLPoint)> for BLBox {
    fn from((lt, rb): (BLPoint, BLPoint)) -> Self {
        Self { x0: lt.x, y0: lt.y, x1: rb.x, y1: rb.y }
    }
}

impl From<(u32, u32, u32, u32)> for BLRectI {
    fn from((x, y, w, h): (u32, u32, u32, u32)) -> Self {
        Self { x: x as _, y: y as _, w: w as _, h: h as _ }
    }
}
impl From<BLRectI> for BLRect {
    fn from(rect: BLRectI) -> Self {
        Self { x: rect.x as _, y: rect.y as _, w: rect.w as _, h: rect.h as _ }
    }
}

impl Default for BLPoint { fn default() -> Self { Self::new() } }
impl BLPoint {
    pub fn new() -> Self { Self { x : 0., y : 0. } }
    pub fn x(&self) -> f64 { self.x as _ }
    pub fn y(&self) -> f64 { self.y as _ }
}

impl BLSizeI {
    pub fn width (&self) -> u32 { self.w as _ }
    pub fn height(&self) -> u32 { self.h as _ }
}

// Prevent external implementations from mismatching an FFI type tag and pointer layout.
mod sealed { pub trait Sealed {} }

pub trait B2DGeometry: sealed::Sealed {
    #[doc(hidden)] fn as_ptr(&self) -> *const BLUnknown;
    const GEOM_T: BLGeometryType;
}
macro_rules! impl_geometry {
    ($ty:ty, $kind:ident) => {
        impl sealed::Sealed for $ty {}
        impl B2DGeometry for $ty {
            const GEOM_T: BLGeometryType = BLGeometryType::$kind;
            fn as_ptr(&self) -> *const BLUnknown { (self as *const Self).cast() }
        }
    };
}
impl_geometry!(BLPath, BL_GEOMETRY_TYPE_PATH);
impl_geometry!(BLLine, BL_GEOMETRY_TYPE_LINE);
impl_geometry!(BLArc, BL_GEOMETRY_TYPE_ARC);
impl_geometry!(BLBox, BL_GEOMETRY_TYPE_BOXD);
impl_geometry!(BLRect, BL_GEOMETRY_TYPE_RECTD);
impl_geometry!(BLCircle, BL_GEOMETRY_TYPE_CIRCLE);
impl_geometry!(BLEllipse, BL_GEOMETRY_TYPE_ELLIPSE);
impl_geometry!(BLTriangle, BL_GEOMETRY_TYPE_TRIANGLE);
impl_geometry!(BLRoundRect, BL_GEOMETRY_TYPE_ROUND_RECT);

impl From<(f64, f64)> for BLPoint {
    fn from((x, y): (f64, f64)) -> Self { Self { x: x as _, y: y as _ } }
}
impl From<(F32, F32)> for BLPoint {
    fn from(v: (F32, F32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(u32, u32)> for BLPoint {
    fn from(v: (u32, u32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(i32, i32)> for BLPoint {
    fn from(v: (i32, i32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<BLPoint> for (f64, f64) {
    fn from(val: BLPoint) -> Self { (val.x as _, val.y as _) }
}
impl From<BLPoint> for (F32, F32) {
    fn from(val: BLPoint) -> Self { (val.x as _, val.y as _) }
}
impl Clone for BLPoint { fn clone(&self) -> Self { *self } }
impl Copy  for BLPoint {}

impl From<(i32, i32)> for BLSizeI {
    fn from((w, h): (i32, i32)) -> Self { Self { w, h } }
}
impl From<(u32, u32)> for BLSizeI {
    fn from(v: (u32, u32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f64, f64)> for BLSize  {
    fn from(v: (f64, f64)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(F32, F32)> for BLSize  {
    fn from(v: (F32, F32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f64, f64, f64, f64)> for BLBox {
    fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(F32, F32, F32, F32)> for BLBox {
    fn from(v: (F32, F32, F32, F32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(u32, u32, u32, u32)> for BLBox {
    fn from(v: (u32, u32, u32, u32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}

impl From<(f64, f64, f64, f64)> for BLRect {
    fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(F32, F32, F32, F32)> for BLRect {
    fn from(v: (F32, F32, F32, F32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(u32, u32, u32, u32)> for BLRect {
    fn from(v: (u32, u32, u32, u32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}

impl From<u32> for BLRgba32 { fn from(value: u32) -> Self { Self { value } } }
impl From<(u8, u8, u8, u8)> for BLRgba32 {  // (r, g, b, a) -> 0xAARRGGBB
    fn from(val: (u8, u8, u8, u8)) -> Self { Self { value:
        ((val.3 as u32) << 24) | ((val.0 as u32) << 16) |
        ((val.1 as u32) <<  8) |  (val.2 as u32)
    } }
}
impl From<(f32, f32, f32, f32)> for BLRgba32 {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        const MAX: f32 = u8::MAX as _;
        Self::new((val.0 * MAX + 0.5) as _, (val.1 * MAX + 0.5) as _,
                  (val.2 * MAX + 0.5) as _, (val.3 * MAX + 0.5) as _)
    }
}
impl BLRgba32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self { Self { value:
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    } }
    pub fn a(&self) -> u8 { (self.value >> 24) as _ }
    pub fn r(&self) -> u8 { (self.value >> 16) as _ }
    pub fn g(&self) -> u8 { (self.value >>  8) as _ }
    pub fn b(&self) -> u8 {  self.value as _ }
}

impl From<BLRgba32> for  BLRgba64 { fn from(v: BLRgba32) -> Self { v.value.into() } }
impl From<u32> for BLRgba64 {
    fn from(v: u32) -> Self {
        let expand = |shift| u64::from((v >> shift) as u8) * 0x0101;
        Self { value: (expand(24) << 48) | (expand(16) << 32) |
                      (expand( 8) << 16) |  expand( 0)
        }
    }
}
impl From<(f32, f32, f32, f32)> for BLRgba64 {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        const MAX: f32 = u16::MAX as _;
        Self::new((val.0 * MAX + 0.5) as _, (val.1 * MAX + 0.5) as _,
                  (val.2 * MAX + 0.5) as _, (val.3 * MAX + 0.5) as _)
    }
}
impl BLRgba64 {
    pub fn new(r: u16, g: u16, b: u16, a: u16) -> Self { Self { value:
        ((a as u64) << 48) | ((r as u64) << 32) | ((g as u64) << 16) | (b as u64)
    } }
    pub fn a(&self) -> u16 { (self.value >> 48) as _ }
    pub fn r(&self) -> u16 { (self.value >> 32) as _ }
    pub fn g(&self) -> u16 { (self.value >> 16) as _ }
    pub fn b(&self) -> u16 {  self.value as _ }
}

impl From<(u8, u8, u8, u8)> for BLRgba {
    fn from(val: (u8, u8, u8, u8)) -> Self {
        const MAX: f32 = u8::MAX as _;
        Self { r: val.0 as f32 / MAX, g: val.1 as f32 / MAX,
               b: val.2 as f32 / MAX, a: val.3 as f32 / MAX }
    }
}
impl From<(f32, f32, f32, f32)> for BLRgba {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        Self { r: val.0, g: val.1, b: val.2, a: val.3 }
    }
}
impl BLRgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!((0.0..=1.).contains(&r) && (0.0..=1.).contains(&g) &&
                      (0.0..=1.).contains(&b) && (0.0..=1.).contains(&a));
        Self { r, g, b, a }
    }
    pub fn a(&self) -> f32 { self.a }
    pub fn r(&self) -> f32 { self.r }
    pub fn g(&self) -> f32 { self.g }
    pub fn b(&self) -> f32 { self.b }
}

impl Clone for BLRgba64 { fn clone(&self) -> Self { *self } }
impl Clone for BLRgba32 { fn clone(&self) -> Self { *self } }
impl Clone for BLRgba   { fn clone(&self) -> Self { *self } }
impl Copy  for BLRgba64 {}
impl Copy  for BLRgba32 {}
impl Copy  for BLRgba   {}

#[repr(transparent)] pub struct BLGradient(BLGradientCore);
impl Drop for BLGradient {
    fn drop(&mut self) { bl_debug!(bl_gradient_destroy(&mut self.0)); }
}

impl BLGradient {
    pub fn new<T: B2DGradient>(gv: &T) -> Result<Self, BLErr> {
        let mut grd = object_init();
        bl_result!(bl_gradient_init_as(&mut grd, T::GR_TYPE, gv.as_ptr(),
            BLExtendMode::BL_EXTEND_MODE_PAD, null(), 0, null()))?;
        Ok(Self(grd))
    }

    pub fn add_stop(&mut self, offset: f32,
        color: BLRgba32) -> Result<(), BLErr> {
        bl_result!(bl_gradient_add_stop_rgba32(&mut self.0, offset as _, color.value))
    }
    pub fn with_stops(mut self,
        stops: &[BLGradientStop]) -> Result<Self, BLErr> {
        bl_result!(bl_gradient_assign_stops(&mut self.0, stops.as_ptr(), stops.len()))?;
        Ok(self)
    }

    pub fn get_transform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::default();
        bl_debug!(bl_gradient_get_transform(&self.0, &mut mat as _));
        mat
    }
    pub fn reset_transform(&mut self,
        mat: Option<&BLMatrix2D>) -> Result<(), BLErr> {
        if let Some(mat) = mat {
            bl_result!(bl_gradient_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _))
        } else {
            bl_result!(bl_gradient_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()))
        }
    }
    pub fn apply_transform(&mut self, mat: &BLMatrix2D) -> Result<(), BLErr> {
        bl_result!(bl_gradient_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _))
    }
    // ignore other matrix related APIs: scale/skew/rotate/translate, ...
}

impl From<(f32, BLRgba32)> for BLGradientStop {
    fn from(val: (f32, BLRgba32)) -> Self {
        Self { offset: val.0 as _, rgba: val.1.into() }
    }
}
impl From<(f64, BLRgba64)> for BLGradientStop {
    fn from(val: (f64, BLRgba64)) -> Self { Self { offset: val.0 as _, rgba: val.1 } }
}

pub trait B2DGradient: sealed::Sealed {
    #[doc(hidden)] fn as_ptr(&self) -> *const BLUnknown;
    const GR_TYPE: BLGradientType;
}
macro_rules! impl_gradient {
    ($ty:ty, $kind:ident) => {
        impl sealed::Sealed for $ty {}
        impl B2DGradient for $ty {
            const GR_TYPE: BLGradientType = BLGradientType::$kind;
            fn as_ptr(&self) -> *const BLUnknown { (self as *const Self).cast() }
        }
    };
}
impl_gradient!(BLLinearGradientValues, BL_GRADIENT_TYPE_LINEAR);
impl_gradient!(BLRadialGradientValues, BL_GRADIENT_TYPE_RADIAL);
impl_gradient!(BLConicGradientValues,  BL_GRADIENT_TYPE_CONIC);

impl BLLinearGradientValues {
    pub fn new(p0: BLPoint, p1: BLPoint) -> Self {
        Self { x0: p0.x, y0: p0.y, x1: p1.x, y1: p1.y }
    }
}
impl BLRadialGradientValues { // center/focal point
    pub fn new(cp: BLPoint, fp: BLPoint, radii: BLVec2D) -> Self {
        Self { x0: cp.x, y0: cp.y, x1: fp.x, y1: fp.y, r0: radii.0 as _, r1: radii.1 as _ }
    }
}
impl BLConicGradientValues {
    pub fn new(pt: BLPoint, angle: f64, repeat: f64) -> Self {
        Self { x0: pt.x, y0: pt.y, angle: angle as _, repeat: repeat as _ }
    }
}

#[repr(transparent)] pub struct BLSolidColor(BLVarCore);
impl Drop for BLSolidColor {
    fn drop(&mut self) { bl_debug!(bl_var_destroy(&mut self.0 as *mut _ as _)); }
}
impl BLSolidColor {
    pub fn init_rgba32(rgba32: BLRgba32) -> Result<Self, BLErr> {
        let mut color: BLVarCore = object_init();
        bl_result!(bl_var_init_rgba32(&mut color as *mut _ as _, rgba32.value))?;
        Ok(Self(color))
    }
    pub fn init_rgba64(rgba64: BLRgba64) -> Result<Self, BLErr> {
        let mut color: BLVarCore = object_init();
        bl_result!(bl_var_init_rgba64(&mut color as *mut _ as _, rgba64.value))?;
        Ok(Self(color))
    }
    pub fn init_rgba(rgba: BLRgba) -> Result<Self, BLErr> {
        let mut color: BLVarCore = object_init();
        bl_result!(bl_var_init_rgba(&mut color as *mut _ as _, &rgba))?;
        Ok(Self(color))
    }
}

pub trait B2DStyle: sealed::Sealed {
    #[doc(hidden)] fn as_ptr(&self) -> *const BLUnknown;
}
macro_rules! impl_style {
    ($ty:ty) => {
        impl sealed::Sealed for $ty {}
        impl B2DStyle for $ty {
            fn as_ptr(&self) -> *const BLUnknown { (self as *const Self).cast() }
        }
    };
}
// Style could be BLRgba, BLRgba32, BLRgba64, BLGradient, BLPattern, and BLVar.

#[repr(transparent)] pub struct BLPattern(BLPatternCore);
impl Drop for BLPattern {
    fn drop(&mut self) { bl_debug!(bl_pattern_destroy(&mut self.0)); }
}
impl BLPattern {
    pub fn new(img: &BLImage) -> Result<Self, BLErr> {
        let mut pat = object_init();
        bl_result!(bl_pattern_init_as(&mut pat, &img.0, null(),
            BLExtendMode::BL_EXTEND_MODE_REFLECT, null()))?;
        Ok(Self(pat))
    }
}
impl_style!(BLPattern);
impl_style!(BLGradient);
impl_style!(BLSolidColor);

#[derive(Debug)] pub struct BLErr(BLResult);
impl BLErr {
    pub fn code(&self) -> BLResult { self.0 }
    fn invalid_value() -> Self { Self(BLResultCode::BL_ERROR_INVALID_VALUE as _) }
}
impl core::fmt::Display for BLErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BLResultCode: {}", self.0)   // XXX: display exact error message
    }   // https://github.com/Veykril/blend2d-rs/blob/master/src/error.rs
}
impl std::error::Error  for BLErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

//}

#[cfg(test)] mod tests { use super::*;
    #[test] fn blend2d_logo() -> Result<(), BLErr> { // Pixel color format: 0xAARRGGBB
        let mut ctx = BLContext::new(480, 480, BLFormat::BL_FORMAT_PRGB32)?;
        //ctx.clear_all()?;

        let mut radial = BLGradient::new(&BLRadialGradientValues::new(
            (180, 180).into(), (180, 180).into(), (180.0, 0.)))?;
        radial.add_stop(0.0, 0xFFFFFFFF.into())?;
        radial.add_stop(1.0, 0xFFFF6F3F.into())?;

        ctx.fill_geometry_ext(&BLCircle::new((180, 180).into(), 160.0), &radial)?;

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            (195, 195).into(), (470, 470).into()))?;
        linear.add_stop(0.0, 0xFFFFFFFF.into())?;
        linear.add_stop(1.0, 0xFF3F9FFF.into())?;

        ctx.set_comp_op(BLCompOp::BL_COMP_OP_DIFFERENCE);
        ctx.fill_geometry_ext(
            &BLRoundRect::new(&(195, 195, 270, 270).into(), 25.0), &linear)?;
        //ctx.set_comp_op(BLCompOp::BL_COMP_OP_SRC_OVER); // restore to default

        let img = ctx.end()?;
        img.write_to_file("target/logo_b2d.png")
    }

    #[test] fn minimal_demo() -> Result<(), BLErr> {
        let mut ctx = BLContext::new(512, 512, BLFormat::BL_FORMAT_PRGB32)?;

        let mut path = BLPath::new();           path.move_to((26,  31).into());
        path.cubic_to((642, 132).into(), (587, -136).into(), (25, 464).into());
        path.cubic_to((882, 404).into(), (144,  267).into(), (27,  31).into());

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            (0, 0).into(), (0, 480).into()))?;
        linear.add_stop(0.0, 0xFFFFFFFF.into())?;
        linear.add_stop(0.5, 0xFFFF1F7F.into())?;
        linear.add_stop(1.0, 0xFF1F7FFF.into())?;

        ctx.set_stroke_width(10.0);
        //ctx.set_stroke_miter_limit(4.0);
        ctx.set_stroke_caps(BLStrokeCap ::BL_STROKE_CAP_ROUND);
        ctx.set_stroke_join(BLStrokeJoin::BL_STROKE_JOIN_ROUND);

        ctx.fill_geometry_rgba32(&path, 0xFFFFFFFF.into())?;
        ctx.stroke_geometry_ext (&path, &linear)?;

        let img = ctx.end()?;     //BLContext::show_rtinfo()?;
        img.write_to_file("target/demo_b2d.png")?; //env::var("OUT_DIR")
        Ok(())
    }
}
