/****************************************************************
 * $ID: blend2d.rs  	Fri 27 Oct 2023 08:44:33+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#![allow(non_upper_case_globals)] #![allow(clippy::enum_variant_names)]
#![allow(unused)] #![allow(clippy::new_without_default)]

//pub mod blend2d  {    // https://blend2d.com
use std::ffi::CString;
use core::{mem, ptr::{self, null, null_mut}};

pub use b2d_ffi::{BLFormat, BLPoint, BLMatrix2D, BLImageData, BLRgba, BLRgba64,
    BLRgba32, BLFillRule, BLStrokeCap, BLStrokeJoin, BLCompOp, BLImageScaleFilter, BLRectI,
    BLRect, BLBox, BLLine, BLArc, BLCircle, BLEllipse, BLTriangle, BLRoundRect, BLHitTest,
    BLLinearGradientValues, BLRadialGradientValues, BLConicGradientValues};

#[cfg(feature = "b2d_sfp")] #[allow(non_camel_case_types)] type f64 = f32;
#[cfg(feature = "b2d_sfp")] type F32 = core::primitive::f64; // XXX: API wrapper differ in f32
#[cfg(not(feature = "b2d_sfp"))] type F32 = f32;

#[allow(non_camel_case_types)] //#[allow(non_upper_case_globals)]     // blend2d_bindings
mod b2d_ffi { include!("../target/bindings/blend2d.rs"); }  use b2d_ffi::*;
// concat!(env!("OUT_DIR"), "/blend2d.rs")  // BGEN_DIR

/*#[macro_export] */macro_rules! safe_dbg { //($v:expr$(,$g:expr)?) => { unsafe { $v } };
    ($v:expr,$g:expr) => { match unsafe { $v } { // as u32
        //eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($v), &res);
        res => { if res != $g { dbg!(res); } res } } };
    ($v:expr) => { safe_dbg!($v, 0) };
}

//  https://blend2d.com/doc/group__bl__object.html
fn object_init<T>() -> T { // for BLObjectCore
    let mut s = mem::MaybeUninit::<T>::uninit();
    unsafe { ptr::write_bytes(s.as_mut_ptr(), 0, 1); s.assume_init() }
}

pub struct BLContext(BLContextCore);
impl Drop for BLContext {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_context_destroy(&mut self.0)); }
}

impl BLContext { //  https://blend2d.com/doc/group__bl__rendering.html
    #[inline] pub fn new(img: &mut BLImage) -> Self {
        /* let mut  cci: BLContextCreateInfo = object_init();
        let mut info: BLRuntimeSystemInfo = object_init();
        safe_dbg!(bl_runtime_query_info(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
                &mut info as *mut _ as _));
        cci.thread_count = info.thread_count; // XXX: depends on BL_BUILD_NO_JIT? */

        let mut ctx = object_init();
        safe_dbg!(bl_context_init_as(&mut ctx, &mut img.0, null())); // &cci

        //safe_dbg!(bl_context_fill_all_rgba32(&mut ctx, 0x00000000));
        //safe_dbg!(bl_context_clear_all(&mut ctx));     // would make it transparent
        Self(ctx)
    }
    #[inline] pub fn begin(&mut self, img: &mut BLImage) {
        safe_dbg!(bl_context_begin(&mut self.0, &mut img.0, null())); // &cci
    }
    #[inline] pub fn get_target_image(&self) -> BLImage {
        let mut img = object_init();
        safe_dbg!(bl_image_assign_weak(&mut img,
            bl_context_get_target_image(&self.0)));     BLImage(img)
    }
    #[inline] pub fn get_target_size(&self) -> BLSizeI {
        let mut sz = (0., 0.).into();
        safe_dbg!(bl_context_get_target_size(&self.0, &mut sz));    //sz
        (sz.w as u32, sz.h as u32).into()
    }

    #[inline] pub fn fill_geometry<T: B2DGeometry>(&mut self, geom: &T) {
        safe_dbg!(bl_context_fill_geometry(&mut self.0, T::GEOM_T, geom as *const _ as _));
    }
    #[inline] pub fn fill_all_rgba32(&mut self, color: BLRgba32) {
        safe_dbg!(bl_context_fill_all_rgba32(&mut self.0, color.value));
    }
    #[inline] pub fn fill_geometry_rgba32<T: B2DGeometry>(&mut self,
        geom: &T, color: BLRgba32) {
        safe_dbg!(bl_context_fill_geometry_rgba32(&mut self.0, T::GEOM_T,
            geom as *const _ as _, color.value));
    }
    #[inline] pub fn fill_geometry_ext<T: B2DGeometry>(&mut self,
        geom: &T, style: &dyn B2DStyle) {
        safe_dbg!(bl_context_fill_geometry_ext(&mut self.0, T::GEOM_T,
            geom as *const _ as _, style as *const _ as _));
    }

    #[inline] pub fn set_fill_style(&mut self, style: &dyn B2DStyle) {
        safe_dbg!(bl_context_set_fill_style(&mut self.0, style as *const _ as _));
    }
    #[inline] pub fn set_fill_rule(&mut self, fill_rule: BLFillRule) {
        safe_dbg!(bl_context_set_fill_rule(&mut self.0, fill_rule));
    }
    #[inline] pub fn set_fill_alpha(&mut self, alpha: f64) {
        safe_dbg!(bl_context_set_fill_alpha(&mut self.0, alpha as _));
    }

    #[inline] pub fn set_stroke_alpha(&mut self, alpha: f64) {
        safe_dbg!(bl_context_set_stroke_alpha(&mut self.0, alpha as _));
    }
    #[inline] pub fn set_stroke_style(&mut self, style: &dyn B2DStyle) {
        safe_dbg!(bl_context_set_stroke_style(&mut self.0, style as *const _ as _));
    }
    #[inline] pub fn set_stroke_width(&mut self, width: f64) {
        safe_dbg!(bl_context_set_stroke_width(&mut self.0, width as _));
    }
    #[inline] pub fn set_stroke_caps(&mut self, caps: BLStrokeCap) {
        safe_dbg!(bl_context_set_stroke_caps(&mut self.0, caps));
    }
    #[inline] pub fn set_stroke_caps2(&mut self, sc: BLStrokeCap, ec: BLStrokeCap) {
        safe_dbg!(bl_context_set_stroke_cap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_START, sc));
        safe_dbg!(bl_context_set_stroke_cap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_END, ec));
    }
    #[inline] pub fn set_stroke_join(&mut self, join: BLStrokeJoin) {
        safe_dbg!(bl_context_set_stroke_join(&mut self.0, join));
    }
    #[inline] pub fn set_stroke_miter_limit(&mut self, miter_limit: f64) {
        safe_dbg!(bl_context_set_stroke_miter_limit(&mut self.0, miter_limit as _));
    }
    #[inline] pub fn set_stroke_dash(&mut self, offset: f64, dash: &[f64]) {
        safe_dbg!(bl_context_set_stroke_dash_offset(&mut self.0, offset as _));
        safe_dbg!(bl_context_set_stroke_dash_array(&mut self.0, &BLArrayFP::new(dash).0));
    }
    #[inline] pub fn set_stroke_options(&mut self, options: BLStrokeOptions) {
        safe_dbg!(bl_context_set_stroke_options(&mut self.0, &options.0));
    }

    #[inline] pub fn stroke_geometry<T: B2DGeometry>(&mut self, geom: &T) {
        safe_dbg!(bl_context_stroke_geometry(&mut self.0,
            T::GEOM_T, geom as *const _ as _));
    }

    #[inline] pub fn stroke_geometry_rgba32<T: B2DGeometry>(&mut self,
        geom: &T, color: BLRgba32) {
        safe_dbg!(bl_context_stroke_geometry_rgba32(&mut self.0, T::GEOM_T,
            geom as *const _ as _, color.value));
    }
    #[inline] pub fn stroke_geometry_ext<T: B2DGeometry>(&mut self,
        geom: &T, style: &dyn B2DStyle) {
        safe_dbg!(bl_context_stroke_geometry_ext(&mut self.0, T::GEOM_T,
            geom as *const _ as _, style as *const _ as _));
    }

    // TODO: textPath - to render text along the shape of a path

    #[inline] pub fn fill_utf8_text_d_rgba32(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(bl_context_fill_utf8_text_d_rgba32(&mut self.0, &origin, &font.0,
            cstr.as_ptr(), text.len(), color.value));
    }
    #[inline] pub fn fill_utf8_text_d_ext(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(bl_context_fill_utf8_text_d_ext(&mut self.0, &origin, &font.0,
            cstr.as_ptr(), text.len(), style as *const _ as _));
    }

    #[inline] pub fn stroke_utf8_text_d_rgba32(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(bl_context_stroke_utf8_text_d_rgba32(&mut self.0, &origin, &font.0,
            cstr.as_ptr(), text.len(), color.value));
    }
    #[inline] pub fn stroke_utf8_text_d_ext(&mut self, origin: BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(bl_context_stroke_utf8_text_d_ext(&mut self.0, &origin, &font.0,
            cstr.as_ptr(), text.len(), style as *const _ as _));
    }

    #[inline] pub fn blit_image_d(&mut self, origin: BLPoint,
        img: &BLImage, img_area: &BLRectI) {
        safe_dbg!(bl_context_blit_image_d(&mut self.0, &origin, &img.0, img_area));
    }
    #[inline] pub fn blit_scaled_image_d(&mut self, rect: &BLRect,
        img: &BLImage,  img_area: &BLRectI) {
        safe_dbg!(bl_context_blit_scaled_image_d(&mut self.0, rect, &img.0, img_area));
    }

    #[inline] pub fn fill_mask_d_rgba32(&mut self, origin: BLPoint,
        mask: &BLImage, mask_area: &BLRectI, color: BLRgba32) {
        safe_dbg!(bl_context_fill_mask_d_rgba32(&mut self.0,
            &origin, &mask.0, mask_area, color.value));
    }
    #[inline] pub fn fill_mask_d_ext(&mut self, origin: BLPoint,
        mask: &BLImage, mask_area: &BLRectI, style: &dyn B2DStyle) {
        safe_dbg!(bl_context_fill_mask_d_ext(&mut self.0,
            &origin, &mask.0, mask_area, style as *const _ as _));
    }

    #[inline] pub fn clip_to_rect_d(&mut self, clip: &mut BLRect) {
        safe_dbg!(bl_context_clip_to_rect_d(&mut self.0, clip));
    }
    #[inline] pub fn restore_clipping(&mut self) {
        safe_dbg!(bl_context_restore_clipping(&mut self.0));
    }

    #[inline] pub fn fill_rect_i_rgba32(&mut self, rect: &BLRectI, color: BLRgba32) {
        safe_dbg!(bl_context_fill_rect_i_rgba32(&mut self.0, rect, color.value));
    }
    #[inline] pub fn clear_rect_d(&mut self, rect: &BLRect) {
        safe_dbg!(bl_context_clear_rect_d(&mut self.0, rect));
    }
    #[inline] pub fn clear_all(&mut self) { safe_dbg!(bl_context_clear_all(&mut self.0)); }

    #[inline] pub fn user_to_meta(&mut self) {
        safe_dbg!(bl_context_user_to_meta(&mut self.0));
    }
    /// get transform matrix from context, kind: *0* - meta, *1* - user, *2* - final
    #[inline] pub fn get_transform(&self, kind: u8) -> BLMatrix2D {
        let mut mat = BLMatrix2D::identity();   match kind {
            0 => safe_dbg!(bl_context_get_meta_transform (&self.0, &mut mat)),
            2 => safe_dbg!(bl_context_get_final_transform(&self.0, &mut mat)),
            _ => safe_dbg!(bl_context_get_user_transform (&self.0, &mut mat)),
        };  mat
    }
    #[inline] pub fn reset_transform(&mut self, mat: Option<&BLMatrix2D>) {
        //let mut lmat = BLMatrix2D::identity();
        //    safe_dbg!(bl_context_get_user_transform(&self.0, &mut lmat));
        if let Some(mat) = mat {
            safe_dbg!(bl_context_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            safe_dbg!(bl_context_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }   //lmat
    }
    #[inline] pub fn apply_transform(&mut self, mat: &BLMatrix2D) {     // multiply
        safe_dbg!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }

    #[inline] pub fn scale(&mut self, sl: BLVec2D) {
        safe_dbg!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_SCALE, BLArrayFP::new(&[sl.0, sl.1]).get_data()));
    }
    #[inline] pub fn translate(&mut self, pos: BLPoint) {
        safe_dbg!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSLATE,
            BLArrayFP::new(&[pos.x, pos.y]).get_data()));
    }
    #[inline] pub fn rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let rot = BLArrayFP::new(&[angle, origin.x, origin.y]);
        safe_dbg!(bl_context_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_ROTATE_PT, rot.get_data()));
        /* if let Some(origin) = origin { ... } else {
            safe_dbg!(bl_context_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ROTATE, &angle as *const _ as _));
        } */
    }

    #[inline] pub fn set_comp_op(&mut self, cop: BLCompOp) {
        safe_dbg!(bl_context_set_comp_op(&mut self.0, cop));
    }
    #[inline] pub fn set_global_alpha(&mut self, alpha: f64) {
        safe_dbg!(bl_context_set_global_alpha(&mut self.0, alpha as _));
    }
    /// value: BLRenderingQuality, BLGradientQuality, BLPatternQuality
    #[inline] pub fn set_hint(&mut self, hint: BLContextHint, value: u32) {
        safe_dbg!(bl_context_set_hint(&mut self.0, hint, value));
    }

    #[inline] pub fn restore(&mut self) {
        safe_dbg!(bl_context_restore(&mut self.0, null()));
    }
    #[inline] pub fn save (&mut self) {
        safe_dbg!(bl_context_save(&mut self.0, null_mut()));
    }
    #[inline] pub fn flush(&mut self/*, flag: BLContextFlushFlags*/) {
        //  This actually flushes all render calls and synchronizes.
        safe_dbg!(bl_context_flush(&mut self.0, BLContextFlushFlags::BL_CONTEXT_FLUSH_SYNC));
    }
    //#[inline] pub fn reset(&mut self) { safe_dbg!(bl_context_reset(&mut self.0)); }
    #[inline] pub fn end(&mut self) { safe_dbg!(bl_context_end(&mut self.0)); }
    //  Detach the rendering context from `img`. (mapped to Reset)

    pub fn show_rtinfo() {
        let mut  info: BLRuntimeBuildInfo  = object_init();
        let mut sinfo: BLRuntimeSystemInfo = object_init();

        safe_dbg!(bl_runtime_query_info(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_BUILD,
            &mut  info as *mut _ as _));
        safe_dbg!(bl_runtime_query_info(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
            &mut sinfo as *mut _ as _));

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
            sinfo.thread_stack_size, sinfo.allocation_granularity);
    }
}

pub struct BLImage(BLImageCore);
impl Drop for BLImage { #[inline] fn drop(&mut self) {
    safe_dbg!(bl_image_destroy(&mut self.0)); } }
impl BLImage { //  https://blend2d.com/doc/group__bl__imaging.html
    #[inline] pub fn new(w: u32, h: u32, fmt: BLFormat) -> Self {
        let mut img = object_init();
        //safe_dbg!(bl_image_create(&mut img, w as _, h as _, fmt));
        safe_dbg!(bl_image_init_as(&mut img, w as _, h as _, fmt));    Self(img)
    }

    #[inline] pub fn from_buffer(w: u32, h: u32, fmt: BLFormat, buf: &mut [u8],
        stride: i32) -> Self {  let mut img = object_init();
        safe_dbg!(bl_image_init_as_from_data(&mut img, w as _, h as _, fmt,
            buf.as_mut_ptr() as _, stride as _, BLDataAccessFlags::BL_DATA_ACCESS_RW,
            None, null_mut()));     Self(img)
    }

    pub fn to_rgba_inplace(&mut self) {     // 0xAARRGGGBB -> 0xAABBGGRR
        let mut di = object_init::<BLFormatInfo>();
        safe_dbg!(bl_format_info_query(&mut di, BLFormat::BL_FORMAT_PRGB32));
        let si = unsafe { ptr::read(&di) };

        let rgba_opt = unsafe {
            &mut di.__bindgen_anon_1.__bindgen_anon_2 };
        rgba_opt.r_shift =  0; rgba_opt.g_shift =  8; rgba_opt.b_shift = 16;

        let mut imgd = BLImageData::new();
        safe_dbg!(bl_image_get_data(&self.0, &mut imgd));

        let mut conv = object_init();
        safe_dbg!(bl_pixel_converter_create(&mut conv, &di, &si,
            BLPixelConverterCreateFlags::BL_PIXEL_CONVERTER_CREATE_NO_FLAGS));
        safe_dbg!(bl_pixel_converter_convert(&conv, imgd.pixel_data, imgd.stride, // dst
            imgd.pixel_data, imgd.stride, imgd.size.w as _, imgd.size.h as _, null()));
        safe_dbg!(bl_pixel_converter_destroy(&mut conv));
    }

    #[inline] pub fn get_data(&self) -> BLImageData {
        let mut imgd = BLImageData::new();
        safe_dbg!(bl_image_get_data(&self.0, &mut imgd));  imgd
    }

    #[inline] pub fn read_from_data(data: &[u8]) -> Result<Self, BLErr> {
        let mut img = object_init();    safe_dbg!(bl_image_init(&mut img));
        let res = unsafe { bl_image_read_from_data(&mut img,
            data.as_ptr() as _, data.len(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(img)) }
    }
    #[inline] pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut img = object_init();    safe_dbg!(bl_image_init(&mut img));
        let file = CString::new(file).unwrap();
        let res = unsafe { bl_image_read_from_file(&mut img, file.as_ptr(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(img)) }
    }

    #[inline] pub fn scale(&mut self, src: &BLImage,
        dst_w: u32, dst_h: u32, filter: BLImageScaleFilter) {
        safe_dbg!(bl_image_scale(&mut self.0, &src.0, &(dst_w, dst_h).into(), filter));
    }

    #[inline] pub fn write_to_file<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), BLErr> {
        let cstr = CString::new(file).unwrap();
        let res = unsafe { bl_image_write_to_file(&self.0, cstr.as_ptr(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(()) }
    }
    #[inline] pub fn save_png<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), BLErr> {
        self.write_to_file(file)?;    Ok(())
    }
}

impl BLImageData {
    #[inline] pub fn new() -> Self {
        Self { pixel_data: null_mut(), stride: 0, size: (0, 0).into(), format: 0, flags: 0 }
    }
    #[inline] pub fn pixels(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.pixel_data as *const u8,
            (self.stride as i32 * self.size.h) as usize) }
    }
    #[inline] pub fn stride(&self) -> i32 { self.stride as _ }
    #[inline] pub fn height(&self) -> u32 { self.size.h as _ }
    #[inline] pub fn width (&self) -> u32 { self.size.w as _ }
    //#[inline] pub fn format(&self) -> u32 { self.format }
    //#[inline] pub fn flags (&self) -> u32 { self.flags }
}

pub struct BLFont(BLFontCore); //  https://blend2d.com/doc/group__bl__text.html
impl Drop for BLFont { #[inline] fn drop(&mut self) {
    safe_dbg!(bl_font_destroy(&mut self.0)); } }
impl BLFont {   // TODO: a bunch of interfaces need to be regarded
    #[inline] pub fn new(face: &BLFontFace, size: f32) -> Self {
        let mut font = object_init();   safe_dbg!(bl_font_init(&mut font));

        safe_dbg!(bl_font_create_from_face(&mut font, &face.0, size));
        //safe_dbg!(bl_font_create_from_face_with_settings(&mut font, &face.0, size,
        //    feature_settings, variation_settings));
        Self(font)
    }
}

pub struct BLFontFace(BLFontFaceCore);
impl Drop for BLFontFace {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_font_face_destroy(&mut self.0)); }
}

impl BLFontFace {
    #[inline] pub fn new(data: &[u8]) -> Result<Self, BLErr> {
        let mut face  = object_init();
        safe_dbg!(bl_font_face_init(&mut face));

        let mut fdata = object_init();
        safe_dbg!(bl_font_data_init(&mut fdata));
        //let cstr = CString::new(file).unwrap();
        //let res = unsafe { bl_font_data_create_from_file(&mut fdata,
        //    cstr.as_ptr(), BLFileReadFlags::BL_FILE_READ_NO_FLAGS) };
        let res = unsafe { bl_font_data_create_from_data(&mut fdata,
            data.as_ptr() as _, data.len(), None, null_mut()) };
        if is_error(res) { return Err(BLErr(res)) }

        safe_dbg!(bl_font_face_create_from_data(&mut face, &fdata, 0));
        safe_dbg!(bl_font_data_destroy(&mut fdata));       Ok(Self(face))
    }

    #[inline] pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut face = object_init();
        safe_dbg!(bl_font_face_init(&mut face));
        let cstr = CString::new(file).unwrap();
        let res = unsafe { bl_font_face_create_from_file(&mut face, cstr.as_ptr(),
            BLFileReadFlags::BL_FILE_READ_NO_FLAGS) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(face)) }
    }
}

pub struct BLStrokeOptions(BLStrokeOptionsCore);
impl Drop for BLStrokeOptions {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_stroke_options_destroy(&mut self.0)); }
}
impl BLStrokeOptions {
    #[inline] pub fn new() -> Self { let mut option = object_init();
        safe_dbg!(bl_stroke_options_init(&mut option));    Self(option)
    }

    #[inline] pub fn set_width(&mut self, width: f64) { self.0.width = width as _; }
    #[inline] pub fn set_miter_limit(&mut self, miter: f64) {
        self.0.miter_limit = miter as _;
    }

    #[inline] pub fn set_options(&mut self, sc: BLStrokeCap, ec: BLStrokeCap,
        join: BLStrokeJoin/*, to: BLStrokeTransformOrder*/) {   // XXX:
        let options = unsafe {
            &mut self.0.__bindgen_anon_1.__bindgen_anon_1 };
        options.start_cap = sc as _;     options.end_cap = ec as _;
        options.join = join as _;       //options.transform_order = to as _;
    }

    #[inline] pub fn set_dash(&mut self, offset: f64, dash: &[f64]) {
        self.0.dash_offset = offset as _;
        //safe_dbg!(bl_array_assign_deep(&mut self.0.dash_array, &BLArrayFP::new(dash).0));
        safe_dbg!(bl_array_assign_deep(&mut (&mut self.0.__bindgen_anon_2.dash_array)._base,
            &BLArrayFP::new(dash).0));
    }
}

impl Default for BLApproximationOptions { #[inline] fn default() -> Self { Self::new() } }
impl BLApproximationOptions {
    #[inline] fn new() -> Self { Self {
        flatten_mode: BLFlattenMode::BL_FLATTEN_MODE_DEFAULT as _,
        offset_mode: BLOffsetMode::BL_OFFSET_MODE_DEFAULT as _, reserved_flags: [0; 6],
        flatten_tolerance: 0.20, simplify_tolerance: 0.05, offset_parameter: 0.414_213_56
    } }
}

struct BLArrayFP(BLArrayCore);
impl Drop for BLArrayFP {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_array_destroy(&mut self.0)); }
}
impl BLArrayFP {
    #[inline] pub fn new(data: &[f64]) -> Self { let mut array = object_init();
        safe_dbg!(bl_array_reserve(&mut array, data.len()));
        //safe_dbg!(bl_array_assign_data(&mut array, data.as_ptr() as _, data.len()));
        //safe_dbg!(bl_array_append_data(&mut array, data_ptr, data.len()));

        if cfg!(feature = "b2d_sfp") {  // mem::size_of::<f64>() == 4
            safe_dbg!(bl_array_init(&mut array, BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT32));
            data.iter().for_each(|v| {
                safe_dbg!(bl_array_append_f32(&mut array, *v as _)); });
        } else {
            safe_dbg!(bl_array_init(&mut array, BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT64));
            data.iter().for_each(|v| {
                safe_dbg!(bl_array_append_f64(&mut array, *v as _)); });
        }   Self(array)
    }

    #[inline] pub fn get_data(&self) -> *const std::os::raw::c_void {
        unsafe { bl_array_get_data(&self.0) }
    }
}

impl Default for BLMatrix2D { #[inline] fn default() -> Self { Self::identity() } }
impl Clone   for BLMatrix2D {
    #[inline] fn clone(&self) -> Self {
        let mut mat = object_init();
        safe_dbg!(bl_matrix2d_apply_op(&mut mat,
            BLTransformOp::BL_TRANSFORM_OP_ASSIGN, self as *const _ as _));     mat
    }
}

impl BLMatrix2D { //  https://blend2d.com/doc/structBLMatrix2D.html
    #[inline] pub fn identity() -> Self {
        let mut mat = object_init();
        safe_dbg!(bl_matrix2d_set_identity(&mut mat));     mat
    }

    #[inline] pub fn new(values: [f64; 6]) -> Self {
        let mut mat = Self::identity();
        unsafe { *mat.__bindgen_anon_1.m = values; }    mat
    }
    #[inline] pub fn get_values(&self) -> [f64; 6] { unsafe { *self.__bindgen_anon_1.m } }
    #[inline] pub fn set_translation(&mut self, pos: BLPoint) {
        safe_dbg!(bl_matrix2d_set_translation(self, pos.x, pos.y));
    }
    #[inline] pub fn set_scaling(&mut self, sl: BLVec2D) {
        safe_dbg!(bl_matrix2d_set_scaling(self, sl.0 as _, sl.1 as _));
    }
    #[inline] pub fn set_skewing(&mut self, sk: BLVec2D) {
        safe_dbg!(bl_matrix2d_set_skewing(self, sk.0, sk.1));
    }
    #[inline] pub fn set_rotation(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        safe_dbg!(bl_matrix2d_set_rotation(self, angle, origin.x, origin.y));
    }

    #[inline] pub fn translate(&mut self, pos: BLPoint) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_TRANSLATE,
            BLArrayFP::new(&[pos.x, pos.y]).get_data()));
        /* let mat = unsafe { &mut self.__bindgen_anon_1.__bindgen_anon_1 };
        mat.m20 += pos.x * mat.m00 + pos.y * mat.m10;
        mat.m21 += pos.x * mat.m01 + pos.y * mat.m11; */
    }
    #[inline] pub fn scale(&mut self, sl: BLVec2D) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_SCALE,
            BLArrayFP::new(&[sl.0, sl.1]).get_data()));
        /* let mat = unsafe { &mut self.__bindgen_anon_1.__bindgen_anon_1 };
        mat.m00 *= sl.x; mat.m01 *= sl.x;   mat.m10 *= sl.y; mat.m11 *= sl.y; */
    }
    #[inline] pub fn skew(&mut self, sk: BLVec2D) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_SKEW,
            BLArrayFP::new(&[sk.0, sk.1]).get_data()));
    }
    #[inline] pub fn rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let rot = BLArrayFP::new(&[angle, origin.x, origin.y]);
        safe_dbg!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_ROTATE_PT, rot.get_data()));
        /* if let Some(origin) = origin { ... } else {
            safe_dbg!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_ROTATE, &angle as *const _ as _));
        } */
    }

    #[inline] pub fn post_translate(&mut self, pos: BLPoint) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_POST_TRANSLATE,
            BLArrayFP::new(&[pos.x, pos.y]).get_data()));
        /* let mat = unsafe { &mut self.__bindgen_anon_1.__bindgen_anon_1 };
        mat.m20 += pos.x; mat.m21 += pos.y; */
    }
    #[inline] pub fn post_scale(&mut self, sl: BLVec2D) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_POST_SCALE,
            BLArrayFP::new(&[sl.0, sl.1]).get_data()));
        /* let mat = unsafe { &mut self.__bindgen_anon_1.__bindgen_anon_1 };
        mat.m00 *= sl.x; mat.m01 *= sl.y;   mat.m10 *= sl.x; mat.m11 *= sl.y;
        mat.m20 *= sl.x; mat.m21 *= sl.y; */
    }
    #[inline] pub fn post_skew(&mut self, sk: BLVec2D) {
        safe_dbg!(bl_matrix2d_apply_op(self, BLTransformOp::BL_TRANSFORM_OP_POST_SKEW,
            BLArrayFP::new(&[sk.0, sk.1]).get_data()));
    }
    #[inline] pub fn post_rotate(&mut self, angle: f64, origin: Option<BLPoint>) {
        let origin = origin.unwrap_or((0., 0.).into());
        let rot = BLArrayFP::new(&[angle, origin.x, origin.y]);
        safe_dbg!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_ROTATE_PT, rot.get_data()));
        /* if let Some(origin) = origin { ... } else {
            safe_dbg!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_POST_ROTATE, &angle as *const _ as _));
        } */
    }

    /*  | a b 0 |
        | c d 0 |
        | e f 1 | */
    /// A' = B * A (new = other * self)
    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }
    #[inline] pub fn post_transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(bl_matrix2d_apply_op(self,
            BLTransformOp::BL_TRANSFORM_OP_POST_TRANSFORM, mat as *const _ as _));
    }
    #[inline] pub fn reset(&mut self, mat: Option<&BLMatrix2D>) {
        if let Some(mat) = mat {
            safe_dbg!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            safe_dbg!(bl_matrix2d_apply_op(self,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }
    }
    #[inline] pub fn invert(&mut self) { safe_dbg!(bl_matrix2d_invert(self, self)); }

    #[inline] pub fn get_scaling(&self) -> BLVec2D {
        let mat = unsafe {
            &self.__bindgen_anon_1.__bindgen_anon_1 };  (mat.m00, mat.m10)
    }

    #[inline] pub fn map_point(&self, pt: BLPoint) -> BLPoint {
        let mut npt = BLPoint::new();
        safe_dbg!(bl_matrix2d_map_pointd_array(self, &mut npt, &pt, 1));    npt
    }
    #[inline] pub fn map_point_array(&self, pts: &mut [BLPoint]) {
        safe_dbg!(bl_matrix2d_map_pointd_array(self,
            pts.as_mut_ptr(), pts.as_ptr(), pts.len()));
    }
}
pub type BLVec2D = (f64, f64);     // (f64, f64), BLSize/BLPoint

pub struct BLPath(BLPathCore);  //  https://blend2d.com/doc/classBLPath.html
impl Drop for BLPath { #[inline] fn drop(&mut self) {
    safe_dbg!(bl_path_destroy(&mut self.0)); } }
impl BLPath {
    #[inline] pub fn new() -> Self { let mut path = object_init();
        safe_dbg!(bl_path_init(&mut path));   Self(path)
    }

    #[inline] pub fn move_to(&mut self, end: BLPoint) {
        safe_dbg!(bl_path_move_to(&mut self.0, end.x, end.y));
    }
    #[inline] pub fn line_to(&mut self, end: BLPoint) {
        safe_dbg!(bl_path_line_to(&mut self.0, end.x, end.y));
    }
    #[inline] pub fn quad_to(&mut self, cp: BLPoint, end: BLPoint) {
        safe_dbg!(bl_path_quad_to(&mut self.0, cp.x, cp.y, end.x, end.y));
    }
    #[inline] pub fn cubic_to(&mut self, c1: BLPoint, c2: BLPoint, end: BLPoint) {
        safe_dbg!(bl_path_cubic_to(&mut self.0, c1.x, c1.y, c2.x, c2.y, end.x, end.y));
    }

    #[inline] pub fn arc_to(&mut self, center: BLPoint, radii: BLVec2D,
        start: f64, sweep: f64) {
        safe_dbg!(bl_path_arc_to(&mut self.0, center.x, center.y,
            radii.0, radii.1, start as _, sweep as _, false));  //, force_move_to
    }
    #[inline] pub fn elliptic_arc_to(&mut self, radii: BLVec2D,    // svg_arc_to
        x_rot: f64, large: bool, sweep: bool, end: BLPoint) {
        safe_dbg!(bl_path_elliptic_arc_to(&mut self.0,
            radii.0, radii.1, x_rot as _, large, sweep, end.x, end.y));
        //  Adds an elliptic arc to the path that follows the SVG specification.
        //  https://www.w3.org/TR/SVG/paths.html#PathDataEllipticalArcCommands
    }
    #[inline] pub fn arc_quadrant_to(&mut self, corner: BLPoint, end: BLPoint) {
        safe_dbg!(bl_path_arc_quadrant_to(&mut self.0, corner.x, corner.y, end.x, end.y));
    }
    #[inline] pub fn poly_to(&mut self, poly: &[BLPoint]) {
        safe_dbg!(bl_path_poly_to(&mut self.0, poly.as_ptr(), poly.len()));
    }

    #[inline] pub fn close(&mut self) { safe_dbg!(bl_path_close(&mut self.0)); }
    //#[inline] pub fn clear(&mut self) { safe_dbg!(bl_path_clear(&mut self.0)); }
    #[inline] pub fn reset(&mut self) { safe_dbg!(bl_path_reset(&mut self.0)); }

    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(bl_path_transform(&mut self.0, null(), mat));
    }

    #[inline] pub fn reserve(&mut self, capacity: u32) {
        safe_dbg!(bl_path_reserve(&mut self.0, capacity as _));
    }
    #[inline] pub fn get_size(&self) -> u32 { unsafe { bl_path_get_size(&self.0) as _ } }
    #[inline] pub fn get_last_vertex(&self) -> Option<BLPoint> {
        let mut pt = BLPoint { x: 0.0, y: 0.0 };
        if is_error(unsafe { bl_path_get_last_vertex(&self.0, &mut pt) }) {
            None } else { Some(pt) }
    }
    #[inline] pub fn get_bounding_box(&self) -> Option<BLBox> {
        let mut bbox = BLBox::new();
        if is_error(unsafe { bl_path_get_bounding_box(&self.0, &mut bbox) }) {
            None } else { Some(bbox) }
    }
    #[inline] pub fn hit_test(&self, pt: BLPoint, fill_rule: BLFillRule) -> BLHitTest {
        unsafe { bl_path_hit_test(&self.0, &pt, fill_rule) }
    }

    #[inline] pub fn add_geometry<T: B2DGeometry>(&mut self, geom: &T, mat: &BLMatrix2D) {
        safe_dbg!(bl_path_add_geometry(&mut self.0, T::GEOM_T, geom as *const _ as _, mat,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    #[inline] pub fn add_path(&mut self, path: &BLPath) {
        safe_dbg!(bl_path_add_path(&mut self.0, &path.0, null()));
    }
    #[inline] pub fn add_transformed_path(&mut self, path: &BLPath, mat: &BLMatrix2D) {
        safe_dbg!(bl_path_add_transformed_path(&mut self.0, &path.0, null(), mat));
    }
    #[inline] pub fn add_stroked_path(&mut self, path: &BLPath,
        options: &BLStrokeOptions, approx: &BLApproximationOptions) {
        safe_dbg!(bl_path_add_stroked_path(&mut self.0, &path.0, null(), &options.0, approx));
    }

    #[inline] pub fn add_rect(&mut self, rect: &BLRect) {
        safe_dbg!(bl_path_add_rect_d(&mut self.0, rect,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }
    #[inline] pub fn add_box(&mut self, bbox: &BLBox) {
        safe_dbg!(bl_path_add_box_d(&mut self.0, bbox,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    #[inline] pub fn iter(&self) -> BLPathIter<'_> { unsafe { BLPathIter {
            cmd: bl_path_get_command_data(&self.0), idx: 0,
            vtx: core::slice::from_raw_parts(bl_path_get_vertex_data(&self.0),
                 bl_path_get_size(&self.0)),
    } } }
}

pub enum BLPathItem {
    QuadTo (BLPoint, BLPoint),
    //ConicTo(BLPoint, f64, BLPoint),
    CubicTo(BLPoint, BLPoint, BLPoint),
    MoveTo (BLPoint),  LineTo(BLPoint), Close,
}

pub struct BLPathIter<'a> { cmd: *const u8, vtx: &'a [BLPoint], idx: u32, }     // usize
impl<'a> Iterator for BLPathIter<'a> {  type Item = BLPathItem;
    fn next(&mut self) -> Option<Self::Item> {
        let idx =  self.idx as usize;
        if  idx == self.vtx.len() { return None }   use {BLPathCmd::*, BLPathItem::*};
        Some(match unsafe { mem::transmute::<u32, BLPathCmd>(*self.cmd.add(idx) as _) } {
            BL_PATH_CMD_MOVE => { self.idx += 1; MoveTo(self.vtx[idx]) }
            BL_PATH_CMD_ON   => { self.idx += 1; LineTo(self.vtx[idx]) }
            BL_PATH_CMD_QUAD => { self.idx += 2; QuadTo(self.vtx[idx], self.vtx[idx + 1]) }
            BL_PATH_CMD_CUBIC => { self.idx += 3;
                CubicTo(self.vtx[idx], self.vtx[idx + 1], self.vtx[idx + 2]) }
            //BL_PATH_CMD_CONIC => { self.idx += 3; ... }
            BL_PATH_CMD_CLOSE => { self.idx += 1; Close }
            _ => return None,
        })
    }
}

impl BLLine {
    #[inline] pub fn new(s: BLPoint, e: BLPoint) -> Self {
        Self { x0: s.x, y0: s.y, x1: e.x, y1: e.y }
    }
}
impl BLArc {
    #[inline] pub fn new(c: BLPoint, r: BLVec2D, start: f64, sweep: f64) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.0, ry: r.1, start: start as _, sweep: sweep as _ }
    }
}
impl BLCircle {
    #[inline] pub fn new(c: BLPoint, r: f64) -> Self { Self { cx: c.x, cy: c.y, r: r as _ } }
}
impl BLEllipse {
    #[inline] pub fn new(c: BLPoint, r: BLVec2D) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.0, ry: r.1 }
     }
}
impl BLTriangle {
    #[inline] pub fn new(a: BLPoint, b: BLPoint, c: BLPoint) -> Self {
        Self { x0: a.x, y0: a.y, x1: b.x, y1: b.y, x2: c.x, y2: c.y }
     }
}
impl BLRoundRect {
    #[inline] pub fn new(rect: &BLRect, radius: f64) -> Self { Self {
        x: rect.x, y: rect.y, w: rect.w, h: rect.h, rx: radius as _, ry: radius as _
    } }
}

impl BLBox  { #[inline] pub fn new() -> Self { Self { x0: 0., y0: 0., x1: 0., y1: 0. } } }
impl BLRect { #[inline] pub fn new() -> Self { Self { x : 0., y : 0.,  w: 0.,  h: 0. } } }
impl From<(BLPoint, BLSize)> for BLRect {
    #[inline] fn from((lt, sz): (BLPoint, BLSize)) -> Self {
        Self { x: lt.x, y: lt.y, w: sz.w, h: sz.h }
    }
}
impl From<(BLPoint, BLPoint)> for BLRect {
    #[inline] fn from((lt, rb): (BLPoint, BLPoint)) -> Self {
        Self { x: lt.x, y: lt.y, w: rb.x - lt.x, h: rb.y - lt.y }   // .abs()?
    }
}
impl From<(BLPoint, BLPoint)> for BLBox {
    #[inline] fn from((lt, rb): (BLPoint, BLPoint)) -> Self {
        Self { x0: lt.x, y0: lt.y, x1: rb.x, y1: rb.y }
    }
}

impl From<(u32, u32, u32, u32)> for BLRectI {
    #[inline] fn from((x, y, w, h): (u32, u32, u32, u32)) -> Self {
        Self { x: x as _, y: y as _, w: w as _, h: h as _ }
    }
}
impl From<BLRectI> for BLRect {
    #[inline] fn from(rect: BLRectI) -> Self {
        Self { x: rect.x as _, y: rect.y as _, w: rect.w as _, h: rect.h as _ }
    }
}

impl Default for BLPoint { #[inline] fn default() -> Self { Self::new() } }
impl BLPoint {
    #[inline] pub fn new() -> Self { Self { x : 0., y : 0. } }
    #[inline] pub fn x(&self) -> f64 { self.x as _ }
    #[inline] pub fn y(&self) -> f64 { self.y as _ }
}

impl BLSizeI {
    #[inline] pub fn width (&self) -> u32 { self.w as _ }
    #[inline] pub fn height(&self) -> u32 { self.h as _ }
}

pub trait B2DGeometry { const GEOM_T: BLGeometryType; }
impl B2DGeometry for BLPath {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_PATH;
}
impl B2DGeometry for BLLine {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_LINE;
}
impl B2DGeometry for BLArc {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_ARC;
}
impl B2DGeometry for BLBox  {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_BOXD;
}
impl B2DGeometry for BLRect {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_RECTD;
}
impl B2DGeometry for BLCircle {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_CIRCLE;
}
impl B2DGeometry for BLEllipse {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_ELLIPSE;
}
impl B2DGeometry for BLTriangle {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_TRIANGLE;
}
impl B2DGeometry for BLRoundRect {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_ROUND_RECT;
}

impl From<(f64, f64)> for BLPoint {
    #[inline] fn from((x, y): (f64, f64)) -> Self { Self { x: x as _, y: y as _ } }
}
impl From<(F32, F32)> for BLPoint {
    #[inline] fn from(v: (F32, F32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(u32, u32)> for BLPoint {
    #[inline] fn from(v: (u32, u32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(i32, i32)> for BLPoint {
    #[inline] fn from(v: (i32, i32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<BLPoint> for (f64, f64) {
    #[inline] fn from(val: BLPoint) -> Self { (val.x as _, val.y as _) }
}
impl From<BLPoint> for (F32, F32) {
    #[inline] fn from(val: BLPoint) -> Self { (val.x as _, val.y as _) }
}
impl Clone for BLPoint { #[inline] fn clone(&self) -> Self { *self } }
impl Copy  for BLPoint {}

impl From<(i32, i32)> for BLSizeI {
    #[inline] fn from((w, h): (i32, i32)) -> Self { Self { w, h } }
}
impl From<(u32, u32)> for BLSizeI {
    #[inline] fn from(v: (u32, u32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f64, f64)> for BLSize  {
    #[inline] fn from(v: (f64, f64)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(F32, F32)> for BLSize  {
    #[inline] fn from(v: (F32, F32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f64, f64, f64, f64)> for BLBox {
    #[inline] fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(F32, F32, F32, F32)> for BLBox {
    #[inline] fn from(v: (F32, F32, F32, F32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(u32, u32, u32, u32)> for BLBox {
    #[inline] fn from(v: (u32, u32, u32, u32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}

impl From<(f64, f64, f64, f64)> for BLRect {
    #[inline] fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(F32, F32, F32, F32)> for BLRect {
    #[inline] fn from(v: (F32, F32, F32, F32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(u32, u32, u32, u32)> for BLRect {
    #[inline] fn from(v: (u32, u32, u32, u32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}

impl From<u32> for BLRgba32 { #[inline] fn from(value: u32) -> Self { Self { value } } }
impl From<(u8, u8, u8, u8)> for BLRgba32 {  // (r, g, b, a) -> 0xAARRGGBB
    #[inline] fn from(val: (u8, u8, u8, u8)) -> Self { Self { value:
        ((val.3 as u32) << 24) | ((val.0 as u32) << 16) |
        ((val.1 as u32) <<  8) |  (val.2 as u32)
    } }
}
impl From<(f32, f32, f32, f32)> for BLRgba32 {
    #[inline] fn from(val: (f32, f32, f32, f32)) -> Self {
        const MAX: f32 = u8::MAX as _;
        Self::new((val.0 * MAX + 0.5) as _, (val.1 * MAX + 0.5) as _,
                  (val.2 * MAX + 0.5) as _, (val.3 * MAX + 0.5) as _)
    }
}
impl BLRgba32 {
    #[inline] pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self { Self { value:
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    } }
    #[inline] pub fn a(&self) -> u8 { (self.value >> 24) as _ }
    #[inline] pub fn r(&self) -> u8 { (self.value >> 16) as _ }
    #[inline] pub fn g(&self) -> u8 { (self.value >>  8) as _ }
    #[inline] pub fn b(&self) -> u8 {  self.value as _ }
}

impl From<BLRgba32> for  BLRgba64 {
    #[inline] fn from(v: BLRgba32) -> Self { v.value.into() } }
impl From<u32> for BLRgba64 {
    #[inline] fn from(v: u32) -> Self { Self { value:
        ((((v >> 16) & 0xFF) as u64) << 40) | (((v >>  24) as u64) << 56) |
        ((((v >>  8) & 0xFF) as u64) << 24) | (((v & 0xFF) as u64) <<  8)
    } }
}
impl From<(f32, f32, f32, f32)> for BLRgba64 {
    #[inline] fn from(val: (f32, f32, f32, f32)) -> Self {
        const MAX: f32 = u16::MAX as _;
        Self::new((val.0 * MAX + 0.5) as _, (val.1 * MAX + 0.5) as _,
                  (val.2 * MAX + 0.5) as _, (val.3 * MAX + 0.5) as _)
    }
}
impl BLRgba64 {
    #[inline] pub fn new(r: u16, g: u16, b: u16, a: u16) -> Self { Self { value:
        ((a as u64) << 48) | ((r as u64) << 32) | ((g as u64) << 16) | (b as u64)
    } }
    #[inline] pub fn a(&self) -> u16 { (self.value >> 48) as _ }
    #[inline] pub fn r(&self) -> u16 { (self.value >> 32) as _ }
    #[inline] pub fn g(&self) -> u16 { (self.value >> 16) as _ }
    #[inline] pub fn b(&self) -> u16 {  self.value as _ }
}

impl From<(u8, u8, u8, u8)> for BLRgba {
    #[inline] fn from(val: (u8, u8, u8, u8)) -> Self {
        const MAX: f32 = u8::MAX as _;
        Self { r: val.0 as f32 / MAX, g: val.1 as f32 / MAX,
               b: val.2 as f32 / MAX, a: val.3 as f32 / MAX }
    }
}
impl From<(f32, f32, f32, f32)> for BLRgba {
    #[inline] fn from(val: (f32, f32, f32, f32)) -> Self {
        Self { r: val.0, g: val.1, b: val.2, a: val.3 }
    }
}
impl BLRgba {
    #[inline] pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!((0.0..=1.).contains(&r) && (0.0..=1.).contains(&g) &&
                      (0.0..=1.).contains(&b) && (0.0..=1.).contains(&a));
        Self { r, g, b, a }
    }
    #[inline] pub fn a(&self) -> f32 { self.a }
    #[inline] pub fn r(&self) -> f32 { self.r }
    #[inline] pub fn g(&self) -> f32 { self.g }
    #[inline] pub fn b(&self) -> f32 { self.b }
}

impl Clone for BLRgba64 { #[inline] fn clone(&self) -> Self { *self } }
impl Clone for BLRgba32 { #[inline] fn clone(&self) -> Self { *self } }
impl Clone for BLRgba   { #[inline] fn clone(&self) -> Self { *self } }
impl Copy  for BLRgba64 {}
impl Copy  for BLRgba32 {}
impl Copy  for BLRgba   {}

pub struct BLGradient(BLGradientCore);
impl Drop for BLGradient {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_gradient_destroy(&mut self.0)); }
}

impl BLGradient {
    #[inline] pub fn new<T: B2DGradient>(gv: &T/*, stops: &[BLGradientStop]*/) -> Self {
        let mut grd = object_init();
        safe_dbg!(bl_gradient_init_as(&mut grd, T::GR_TYPE, gv as *const _ as _,
            BLExtendMode::BL_EXTEND_MODE_PAD, null(), 0, null()));   Self(grd)
    }                                         //stops.as_ptr(), stops.len(),

    #[inline] pub fn add_stop(&mut self, offset: f32, color: BLRgba32) {
        safe_dbg!(bl_gradient_add_stop_rgba32(&mut self.0, offset as _, color.value));
    }
    #[inline] pub fn with_stops(mut self, stops: &[BLGradientStop]) -> Self {
        safe_dbg!(bl_gradient_assign_stops(&mut self.0, stops.as_ptr(), stops.len()));  self
    }

    #[inline] pub fn get_transform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::default();
        safe_dbg!(bl_gradient_get_transform(&self.0, &mut mat as _));   mat
    }
    #[inline] pub fn reset_transform(&mut self, mat: Option<&BLMatrix2D>) {
        if let Some(mat) = mat {
            safe_dbg!(bl_gradient_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            safe_dbg!(bl_gradient_apply_transform_op(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }
    }
    #[inline] pub fn apply_transform(&mut self, mat: &BLMatrix2D) {     // multiply
        safe_dbg!(bl_gradient_apply_transform_op(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }
    // ignore other matrix related APIs: scale/skew/rotate/translate, ...
}

impl From<(f32, BLRgba32)> for BLGradientStop {
    #[inline] fn from(val: (f32, BLRgba32)) -> Self {
        Self { offset: val.0 as _, rgba: val.1.into() }
    }
}
impl From<(f64, BLRgba64)> for BLGradientStop {
    #[inline] fn from(val: (f64, BLRgba64)) -> Self { Self { offset: val.0 as _, rgba: val.1 } }
}

pub trait B2DGradient { const GR_TYPE: BLGradientType; }
impl B2DGradient for BLLinearGradientValues {
    const GR_TYPE: BLGradientType = BLGradientType::BL_GRADIENT_TYPE_LINEAR;
}
impl B2DGradient for BLRadialGradientValues {
    const GR_TYPE: BLGradientType = BLGradientType::BL_GRADIENT_TYPE_RADIAL;
}
impl B2DGradient for BLConicGradientValues {
    const GR_TYPE: BLGradientType = BLGradientType::BL_GRADIENT_TYPE_CONIC;
}

impl BLLinearGradientValues {
    #[inline] pub fn new(p0: BLPoint, p1: BLPoint) -> Self {
        Self { x0: p0.x, y0: p0.y, x1: p1.x, y1: p1.y }
    }
}
impl BLRadialGradientValues { // center/focal point
    #[inline] pub fn new(cp: BLPoint, fp: BLPoint, radii: BLVec2D) -> Self {
        Self { x0: cp.x, y0: cp.y, x1: fp.x, y1: fp.y, r0: radii.0 as _, r1: radii.1 as _ }
    }
}
impl BLConicGradientValues {
    #[inline] pub fn new(pt: BLPoint, angle: f64, repeat: f64) -> Self {
        Self { x0: pt.x, y0: pt.y, angle: angle as _, repeat: repeat as _ }
    }
}

pub struct BLSolidColor(BLVarCore);
impl Drop for BLSolidColor {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_var_destroy(&mut self.0 as *mut _ as _)); }
}
impl BLSolidColor {
    #[inline] pub fn init_rgba32(rgba32: BLRgba32) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(bl_var_init_rgba32(&mut color as *mut _ as _,  rgba32.value));
        Self(color)
    }
    #[inline] pub fn init_rgba64(rgba64: BLRgba64) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(bl_var_init_rgba64(&mut color as *mut _ as _,  rgba64.value));
        Self(color)
    }
    #[inline] pub fn init_rgba  (rgba:   BLRgba) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(bl_var_init_rgba  (&mut color as *mut _ as _, &rgba));
        Self(color)
    }
}

pub trait B2DStyle {}
impl B2DStyle for BLPattern {}
impl B2DStyle for BLGradient {}
impl B2DStyle for BLSolidColor {}
// Style could be BLRgba, BLRgba32, BLRgba64, BLGradient, BLPattern, and BLVar.

pub struct BLPattern(BLPatternCore);
impl Drop for BLPattern {
    #[inline] fn drop(&mut self) { safe_dbg!(bl_pattern_destroy(&mut self.0)); }
}
impl BLPattern {
    #[inline] pub fn new(img: &BLImage) -> Self {
        let mut pat = object_init();
        safe_dbg!(bl_pattern_init_as(&mut pat, &img.0, null(),
            BLExtendMode::BL_EXTEND_MODE_REFLECT, null()));     Self(pat)
    }
}

#[derive(Debug)] pub struct BLErr(BLResult);
impl core::fmt::Display for BLErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BLResultCode: {}", self.0)   // XXX: display exact error message
    }   // https://github.com/Veykril/blend2d-rs/blob/master/src/error.rs
}
impl std::error::Error  for BLErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

#[inline] fn is_error(res: BLResult) -> bool { res != BLResultCode::BL_SUCCESS as u32 }

//}

#[cfg(test)] mod tests { use super::*;
    #[test] fn blend2d_logo() {     // Pixel color format: 0xAARRGGBB
        let mut img = BLImage::new(480, 480, BLFormat::BL_FORMAT_PRGB32);
        let mut ctx = BLContext::new(&mut img);     //ctx.clear_all();

        let mut radial = BLGradient::new(&BLRadialGradientValues::new(
            (180, 180).into(), (180, 180).into(), (180.0, 0.)));
        radial.add_stop(0.0, 0xFFFFFFFF.into());
        radial.add_stop(1.0, 0xFFFF6F3F.into());

        ctx.fill_geometry_ext(&BLCircle::new((180, 180).into(), 160.0), &radial);

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            (195, 195).into(), (470, 470).into()));
        linear.add_stop(0.0, 0xFFFFFFFF.into());
        linear.add_stop(1.0, 0xFF3F9FFF.into());

        ctx.set_comp_op(BLCompOp::BL_COMP_OP_DIFFERENCE);
        ctx.fill_geometry_ext(&BLRoundRect::new(&(195, 195, 270, 270).into(), 25.0), &linear);
        //ctx.set_comp_op(BLCompOp::BL_COMP_OP_SRC_OVER);   // restore to default

        let _ = img.write_to_file("target/logo_b2d.png");
    }

    #[test] fn minimal_demo() {
        let mut img = BLImage::new(512, 512, BLFormat::BL_FORMAT_PRGB32);
        let mut ctx = BLContext::new(&mut img);
        let mut path = BLPath::new();

        path. move_to(( 26,  31).into());
        path.cubic_to((642, 132).into(), (587, -136).into(), (25, 464).into());
        path.cubic_to((882, 404).into(), (144,  267).into(), (27,  31).into());

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            (0, 0).into(), (0, 480).into()));
        linear.add_stop(0.0, 0xFFFFFFFF.into());
        linear.add_stop(0.5, 0xFFFF1F7F.into());
        linear.add_stop(1.0, 0xFF1F7FFF.into());

        ctx.set_stroke_width(10.0);
        //ctx.set_stroke_miter_limit(4.0);
        ctx.set_stroke_caps(BLStrokeCap ::BL_STROKE_CAP_ROUND);
        ctx.set_stroke_join(BLStrokeJoin::BL_STROKE_JOIN_ROUND);

        ctx.fill_geometry_rgba32(&path, 0xFFFFFFFF.into());
        ctx.stroke_geometry_ext (&path, &linear);
        let _ = img.write_to_file("target/demo_b2d.png");  //env::var("OUT_DIR")
        //BLContext::show_rtinfo();
    }
}

