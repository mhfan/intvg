/****************************************************************
 * $ID: blend2d.rs  	Fri 27 Oct 2023 08:44:33+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#![allow(non_snake_case)] #![allow(unused)] #![allow(clippy::new_without_default)]

//pub mod blend2d  {    // https://blend2d.com
use core::{mem, ptr::{self, null, null_mut}};
use std::ffi::CString;

pub use b2d_ffi::{BLFormat, BLPoint, BLMatrix2D, BLImageData, BLRgba, BLRgba64,
    BLRgba32, BLFillRule, BLStrokeCap, BLStrokeJoin, BLCompOp, BLImageScaleFilter,
    BLRect, BLBox, BLLine, BLArc, BLCircle, BLEllipse, BLTriangle, BLRoundRect, BLHitTest,
    BLLinearGradientValues, BLRadialGradientValues, BLConicGradientValues};

#[allow(non_camel_case_types)] //#[allow(non_upper_case_globals)]     // blend2d_bindings
mod b2d_ffi { include!(concat!(env!("OUT_DIR"), "/blend2d.rs")); }  use b2d_ffi::*;

/*#[macro_export] */macro_rules! safe_dbg { //($v:expr$(,$g:expr)?) => { unsafe { $v } };
    ($v:expr,$g:expr) => { match unsafe { $v } { // as u32
        //eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($v), &res);
        res => { if res != $g { dbg!(res); } res } } };
    ($v:expr) => { safe_dbg!($v, 0) };
}

//  https://blend2d.com/doc/group__blend2d__api__object.html
fn object_init<T>() -> T { // for BLObjectCore
    let mut s = mem::MaybeUninit::<T>::uninit();
    unsafe { ptr::write_bytes(s.as_mut_ptr(), 0, 1); s.assume_init() }
}

pub struct BLContext(BLContextCore);
impl Drop for BLContext {
    #[inline] fn drop(&mut self) { safe_dbg!(blContextDestroy(&mut self.0)); }
}

impl BLContext { //  https://blend2d.com/doc/group__blend2d__api__rendering.html
    #[inline] pub fn new(img: &mut BLImage) -> Self {
        /* let mut  cci: BLContextCreateInfo = object_init();
        let mut info: BLRuntimeSystemInfo = object_init();
        safe_dbg!(blRuntimeQueryInfo(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
                &mut info as *mut _ as _));
        cci.threadCount = info.threadCount; // XXX: depends on BL_BUILD_NO_JIT? */

        let mut ctx = object_init();
        safe_dbg!(blContextInitAs(&mut ctx, &mut img.0, null())); // &cci
        //safe_dbg!(blContextBegin(&mut ctx, &mut img.0, null())); // &cci

        //safe_dbg!(blContextFillAllRgba32(&mut ctx, 0x00000000));
        //safe_dbg!(blContextClearAll(&mut ctx));     // would make it transparent
        Self(ctx)
    }

    #[inline] pub fn fillAllRgba32(&mut self, color: BLRgba32) {
        safe_dbg!(blContextFillAllRgba32(&mut self.0, color.value));
    }
    #[inline] pub fn fillGeometryRgba32<T: B2DGeometry>(&mut self, geom: &T, color: BLRgba32) {
        safe_dbg!(blContextFillGeometryRgba32(&mut self.0, T::GEOM_T,
            geom as *const _ as _, color.value));
    }
    #[inline] pub fn fillGeometryExt<T: B2DGeometry>(&mut self, geom: &T, style: &dyn B2DStyle) {
        safe_dbg!(blContextFillGeometryExt(&mut self.0, T::GEOM_T,
            geom as *const _ as _, style as *const _ as _));
    }

    #[inline] pub fn setFillStyle(&mut self, style: &dyn B2DStyle) {
        safe_dbg!(blContextSetFillStyle(&mut self.0, style as *const _ as _));
    }
    #[inline] pub fn setFillRule (&mut self, fillRule: BLFillRule) {
        safe_dbg!(blContextSetFillRule(&mut self.0, fillRule));
    }
    #[inline] pub fn setFillAlpha(&mut self, alpha: f32) {
        safe_dbg!(blContextSetFillAlpha(&mut self.0, alpha as _));
    }

    #[inline] pub fn setStrokeAlpha(&mut self, alpha: f32) {
        safe_dbg!(blContextSetStrokeAlpha(&mut self.0, alpha as _));
    }
    #[inline] pub fn setStrokeStyle(&mut self, style: &dyn B2DStyle) {
        safe_dbg!(blContextSetStrokeStyle(&mut self.0, style as *const _ as _));
    }
    #[inline] pub fn setStrokeWidth(&mut self, width: f32) {
        safe_dbg!(blContextSetStrokeWidth(&mut self.0, width as _));
    }
    #[inline] pub fn setStrokeCaps (&mut self, caps: BLStrokeCap) {
        safe_dbg!(blContextSetStrokeCaps(&mut self.0, caps));
    }
    #[inline] pub fn setStrokeCaps2(&mut self, sc: BLStrokeCap, ec: BLStrokeCap) {
        safe_dbg!(blContextSetStrokeCap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_START, sc));
        safe_dbg!(blContextSetStrokeCap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_END, ec));
    }
    #[inline] pub fn setStrokeJoin(&mut self, join: BLStrokeJoin) {
        safe_dbg!(blContextSetStrokeJoin(&mut self.0, join));
    }
    #[inline] pub fn setStrokeMiterLimit(&mut self, miter_limit: f32) {
        safe_dbg!(blContextSetStrokeMiterLimit(&mut self.0, miter_limit as _));
    }
    #[inline] pub fn setStrokeDash(&mut self, offset: f32, dash: &[f32]) {
        safe_dbg!(blContextSetStrokeDashOffset(&mut self.0, offset as _));
        safe_dbg!(blContextSetStrokeDashArray (&mut self.0, &BLDashArray::new(dash).0));
    }
    #[inline] pub fn setStrokeOptions(&mut self, options: BLStrokeOptions) {
        safe_dbg!(blContextSetStrokeOptions(&mut self.0, &options.0));
    }

    #[inline] pub fn strokeGeometryRgba32<T: B2DGeometry>(&mut self, geom: &T, color: BLRgba32) {
        safe_dbg!(blContextStrokeGeometryRgba32(&mut self.0, T::GEOM_T,
            geom as *const _ as _, color.value));
    }
    #[inline] pub fn strokeGeometryExt<T: B2DGeometry>(&mut self, geom: &T, style: &dyn B2DStyle) {
        safe_dbg!(blContextStrokeGeometryExt(&mut self.0, T::GEOM_T,
            geom as *const _ as _, style as *const _ as _));
    }

    // TODO: textPath - to render text along the shape of a path

    #[inline] pub fn fillUtf8TextDRgba32(&mut self, origin: &BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(blContextFillUtf8TextDRgba32(&mut self.0, origin, &font.0,
            cstr.as_ptr(), text.len(), color.value));
    }
    #[inline] pub fn fillUtf8TextDExt(&mut self, origin: &BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(blContextFillUtf8TextDExt(&mut self.0, origin, &font.0,
            cstr.as_ptr(), text.len(), style as *const _ as _));
    }

    #[inline] pub fn strokeUtf8TextDRgba32(&mut self, origin: &BLPoint,
        font: &BLFont, text: &str, color: BLRgba32) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(blContextStrokeUtf8TextDRgba32(&mut self.0, origin, &font.0,
            cstr.as_ptr(), text.len(), color.value));
    }
    #[inline] pub fn strokeUtf8TextDExt(&mut self, origin: &BLPoint,
        font: &BLFont, text: &str, style: &dyn B2DStyle) {
        let cstr = CString::new(text).unwrap();
        safe_dbg!(blContextStrokeUtf8TextDExt(&mut self.0, origin, &font.0,
            cstr.as_ptr(), text.len(), style as *const _ as _));
    }

    #[inline] pub fn blitImageD(&mut self, origin: &BLPoint, img: &BLImage, imgArea: &BLRectI) {
        safe_dbg!(blContextBlitImageD(&mut self.0, origin, &img.0, imgArea));
    }
    #[inline] pub fn blitScaledImageD(&mut self, rect: &BLRect,
        img: &BLImage,  imgArea: &BLRectI) {
        safe_dbg!(blContextBlitScaledImageD(&mut self.0, rect, &img.0, imgArea));
    }

    #[inline] pub fn fillMaskDRgba32(&mut self, origin: &BLPoint,
        mask: &BLImage, maskArea: &BLRectI, color: BLRgba32) {
        safe_dbg!(blContextFillMaskDRgba32(&mut self.0, origin, &mask.0, maskArea, color.value));
    }
    #[inline] pub fn fillMaskDExt(&mut self, origin: &BLPoint,
        mask: &BLImage, maskArea: &BLRectI, style: &dyn B2DStyle) {
        safe_dbg!(blContextFillMaskDExt(&mut self.0, origin, &mask.0, maskArea,
            style as *const _ as _));
    }

    #[inline] pub fn clipToRectD(&mut self, clip: &mut BLRect) {
        safe_dbg!(blContextClipToRectD(&mut self.0, clip));
    }
    #[inline] pub fn restoreClipping(&mut self) {
        safe_dbg!(blContextRestoreClipping(&mut self.0));
    }

    #[inline] pub fn clearRectD(&mut self, rect: &BLRect) {
        safe_dbg!(blContextClearRectD(&mut self.0, rect));
    }
    #[inline] pub fn clearAll(&mut self) { safe_dbg!(blContextClearAll(&mut self.0)); }

    #[inline] pub fn moveUserToMeta(&mut self) { safe_dbg!(blContextUserToMeta(&mut self.0)); }
    #[inline] pub fn getUserTransform(&self) -> BLMatrix2D {
        let mut mat = BLMatrix2D::new();
        //safe_dbg!(blContextGetFinalTransform(&self.0, &mut mat));
        //safe_dbg!(blContextGetMetaTransform(&self.0, &mut mat));
        safe_dbg!(blContextGetUserTransform(&self.0, &mut mat));    mat
    }
    #[inline] pub fn reset_transform(&mut self, mat: Option<&BLMatrix2D>) -> BLMatrix2D {
        let mut lmat = BLMatrix2D::new();
            safe_dbg!(blContextGetUserTransform(&self.0, &mut lmat));
        if let Some(mat) = mat {
            safe_dbg!(blContextApplyTransformOp(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_ASSIGN, mat as *const _ as _));
        } else {
            safe_dbg!(blContextApplyTransformOp(&mut self.0,
                BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
        }   lmat
    }
    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }
    #[inline] pub fn scale(&mut self, sx: f32, sy: f32) {
        #[cfg(feature = "b2d_sfp")] let scale = &[sx, sy];
        #[cfg(not(feature = "b2d_sfp"))] let scale = &[sx as _, sy as f64];
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_SCALE, scale.as_ptr() as _));
    }
    #[inline] pub fn translate(&mut self, tx: f32, ty: f32) {
        #[cfg(feature = "b2d_sfp")] let pos = &[tx, ty];
        #[cfg(not(feature = "b2d_sfp"))] let pos = &[tx as _, ty as f64];
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSLATE, pos.as_ptr() as _));
    }
    #[inline] pub fn rotate(&mut self, angle: f32, orig: Option<(f32, f32)>) {
        let orig = orig.unwrap_or((0., 0.));
        #[cfg(feature = "b2d_sfp")] let rot = &[angle, orig.0, orig.1];
        #[cfg(not(feature = "b2d_sfp"))]
            let rot = &[angle as _, orig.0 as _, orig.1 as f64];
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_ROTATE_PT, rot.as_ptr() as _));
    }

    #[inline] pub fn setCompOp(&mut self, cop: BLCompOp) {
        safe_dbg!(blContextSetCompOp(&mut self.0, cop));
    }
    #[inline] pub fn setGlobalAlpha(&mut self, alpha: f32) {
        safe_dbg!(blContextSetGlobalAlpha(&mut self.0, alpha as _));
    }

    #[inline] pub fn restore(&mut self) { safe_dbg!(blContextRestore(&mut self.0, null())); }
    #[inline] pub fn save (&mut self) { safe_dbg!(blContextSave(&mut self.0, null_mut())); }
    #[inline] pub fn flush(&mut self/*, flag: BLContextFlushFlags*/) {
        //  This actually flushes all render calls and synchronizes.
        safe_dbg!(blContextFlush(&mut self.0, BLContextFlushFlags::BL_CONTEXT_FLUSH_SYNC));
    }
    //#[inline] pub fn reset(&mut self) { safe_dbg!(blContextReset(&mut self.0)); }
    #[inline] pub fn end(&mut self) { safe_dbg!(blContextEnd(&mut self.0)); }
    //  Detach the rendering context from `img`. (mapped to Reset)

    pub fn show_rtinfo() {
        let mut  info: BLRuntimeBuildInfo  = object_init();
        let mut sinfo: BLRuntimeSystemInfo = object_init();

        safe_dbg!(blRuntimeQueryInfo(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_BUILD,
            &mut  info as *mut _ as _));
        safe_dbg!(blRuntimeQueryInfo(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
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
}}"#,       info.majorVersion, info.minorVersion, info.patchVersion,
            if info.buildType == BLRuntimeBuildType::BL_RUNTIME_BUILD_TYPE_DEBUG as u32 {
                "Debug" } else { "Release" },
            sinfo.cpuFeatures, info.baselineCpuFeatures, info.supportedCpuFeatures,
            std::env::consts::ARCH, mem::size_of::<*const i32>() * 8, std::env::consts::OS,
            info.compilerInfo.iter().map(|i| char::from(*i as u8)).collect::<String>(),
            info.maxImageSize, info.maxThreadCount, sinfo.threadCount,
            sinfo.threadStackSize, sinfo.allocationGranularity);
    }
}

pub struct BLImage(BLImageCore);
impl Drop for BLImage { #[inline] fn drop(&mut self) { safe_dbg!(blImageDestroy(&mut self.0)); } }
impl BLImage { //  https://blend2d.com/doc/group__blend2d__api__imaging.html
    #[inline] pub fn new(w: u32, h: u32, fmt: BLFormat) -> Self {
        let mut img = object_init();
        //safe_dbg!(blImageCreate(&mut img, w as _, h as _, fmt));
        safe_dbg!(blImageInitAs(&mut img, w as _, h as _, fmt));    Self(img)
    }

    #[inline] pub fn from_buffer(w: u32, h: u32, fmt: BLFormat, buf: &mut [u8],
        stride: u32) -> Self {  let mut img = object_init();
        safe_dbg!(blImageInitAsFromData(&mut img, w as _, h as _, fmt, buf.as_mut_ptr() as _,
            stride as _, BLDataAccessFlags::BL_DATA_ACCESS_RW, None, null_mut()));  Self(img)
    }

    pub fn to_rgba_inplace(&mut self) {     // 0xAARRGGGBB -> 0xAABBGGRR
        let mut di = object_init::<BLFormatInfo>();
        safe_dbg!(blFormatInfoQuery(&mut di, BLFormat::BL_FORMAT_PRGB32));
        let si = unsafe { ptr::read(&di) };

        let mut rgba_opt = unsafe {
            ptr::read(&di.__bindgen_anon_1.__bindgen_anon_2) };
        rgba_opt.rShift =  0; rgba_opt.gShift =  8; rgba_opt.bShift = 16;
        unsafe { ptr::write(&mut di.__bindgen_anon_1.__bindgen_anon_2, rgba_opt); }

        let mut imgd = BLImageData::new();
        safe_dbg!(blImageGetData(&self.0, &mut imgd));

        let mut conv = object_init();
        safe_dbg!(blPixelConverterCreate(&mut conv, &di, &si,
            BLPixelConverterCreateFlags::BL_PIXEL_CONVERTER_CREATE_NO_FLAGS));
        safe_dbg!(blPixelConverterConvert(&conv, imgd.pixelData, imgd.stride, // dst
            imgd.pixelData, imgd.stride, imgd.size.w as _, imgd.size.h as _, null()));
        safe_dbg!(blPixelConverterDestroy(&mut conv));
    }

    #[inline] pub fn getData(&self) -> BLImageData {
        let mut imgd = BLImageData::new();
        safe_dbg!(blImageGetData(&self.0, &mut imgd));  imgd
    }

    #[inline] pub fn readFromData(data: &[u8]) -> Result<Self, BLErr> {
        let mut img = object_init();    safe_dbg!(blImageInit(&mut img));
        let res = unsafe { blImageReadFromData(&mut img,
            data.as_ptr() as _, data.len(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(img)) }
    }
    #[inline] pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut img = object_init();    safe_dbg!(blImageInit(&mut img));
        let file = CString::new(file).unwrap();
        let res = unsafe { blImageReadFromFile(&mut img, file.as_ptr(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(img)) }
    }

    #[inline] pub fn scale(&mut self, src: &BLImage,
        dstW: u32, dstH: u32, filter: BLImageScaleFilter) {
        safe_dbg!(blImageScale(&mut self.0, &src.0, &(dstW, dstH).into(), filter));
    }

    #[inline] pub fn writeToFile<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), BLErr> {
        let cstr = CString::new(file).unwrap();
        let res = unsafe { blImageWriteToFile(&self.0, cstr.as_ptr(), null()) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(()) }
    }
    #[inline] pub fn save_png<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), BLErr> {
        self.writeToFile(file)?;    Ok(())
    }
}

impl BLImageData {
    #[inline] pub fn new() -> Self {
        Self { pixelData: null_mut(), stride: 0, size: (0, 0).into(), format: 0, flags: 0 }
    }
    #[inline] pub fn pixels(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pixelData as *const u8,
            (self.stride as i32 * self.size.h) as usize) }
    }
    #[inline] pub fn stride(&self) -> u32 { self.stride as _ }
    #[inline] pub fn height(&self) -> u32 { self.size.h as _ }
    #[inline] pub fn width (&self) -> u32 { self.size.w as _ }
    //#[inline] pub fn format(&self) -> u32 { self.format }
    //#[inline] pub fn flags (&self) -> u32 { self.flags }
}

pub struct BLFont(BLFontCore); //  https://blend2d.com/doc/group__blend2d__api__text.html
impl Drop for BLFont { #[inline] fn drop(&mut self) { safe_dbg!(blFontDestroy(&mut self.0)); } }
impl BLFont {   // TODO: a bunch of the interfaces need to regard
    #[inline] pub fn new(face: &BLFontFace, size: f32) -> Self {
        let mut font = object_init();   safe_dbg!(blFontInit(&mut font));

        safe_dbg!(blFontCreateFromFace(&mut font, &face.0, size));
        //safe_dbg!(blFontCreateFromFaceWithSettings(&mut font, &face.0, size,
        //    featureSettings, variationSettings));
        Self(font)
    }
}

pub struct BLFontFace(BLFontFaceCore);
impl Drop for BLFontFace {
    #[inline] fn drop(&mut self) { safe_dbg!(blFontFaceDestroy(&mut self.0)); }
}

impl BLFontFace {
    #[inline] pub fn new(data: &[u8]) -> Result<Self, BLErr> {
        let mut face  = object_init();  safe_dbg!(blFontFaceInit(&mut face));

        let mut fdata = object_init();  safe_dbg!(blFontDataInit(&mut fdata));
        //let cstr = CString::new(file).unwrap();
        //let res = unsafe { blFontDataCreateFromFile(&mut fdata,
        //    cstr.as_ptr(), BLFileReadFlags::BL_FILE_READ_NO_FLAGS) };
        let res = unsafe { blFontDataCreateFromData(&mut fdata,
            data.as_ptr() as _, data.len(), None, null_mut()) };
        if is_error(res) { return Err(BLErr(res)) }

        safe_dbg!(blFontFaceCreateFromData(&mut face, &fdata, 0));
        safe_dbg!(blFontDataDestroy(&mut fdata));       Ok(Self(face))
    }

    #[inline] pub fn from_file(file: &str) -> Result<Self, BLErr> {
        let mut face = object_init();   safe_dbg!(blFontFaceInit(&mut face));
        let cstr = CString::new(file).unwrap();
        let res = unsafe { blFontFaceCreateFromFile(&mut face, cstr.as_ptr(),
            BLFileReadFlags::BL_FILE_READ_NO_FLAGS) };
        if is_error(res) { Err(BLErr(res)) } else { Ok(Self(face)) }
    }
}

pub struct BLStrokeOptions(BLStrokeOptionsCore);
impl Drop for BLStrokeOptions {
    #[inline] fn drop(&mut self) { safe_dbg!(blStrokeOptionsDestroy(&mut self.0)); }
}
impl BLStrokeOptions {
    #[inline] pub fn new() -> Self { let mut option = object_init();
        safe_dbg!(blStrokeOptionsInit(&mut option));    Self(option)
    }

    #[inline] pub fn setWidth(&mut self, width: f32) { self.0.width = width as _; }
    #[inline] pub fn setMiterLimit(&mut self, miter: f32) { self.0.miterLimit = miter as _; }

    #[inline] pub fn setOptions(&mut self, sc: BLStrokeCap, ec: BLStrokeCap,
        join: BLStrokeJoin/*, to: BLStrokeTransformOrder*/) {   // XXX:
            let mut options = unsafe {
                ptr::read(&self.0.__bindgen_anon_1.__bindgen_anon_1) };
        options.startCap = sc as _;     options.endCap = ec as _;
        options.join = join as _;       //options.transformOrder = to as _;
        unsafe { ptr::write(&mut self.0.__bindgen_anon_1.__bindgen_anon_1, options); }
    }

    #[inline] pub fn setDash(&mut self, offset: f32, dash: &[f32]) {
        self.0.dashOffset = offset as _;
        safe_dbg!(blArrayAssignDeep(&mut self.0.dashArray, &BLDashArray::new(dash).0));
    }
}

impl Default for BLApproximationOptions { fn default() -> Self { Self::new() } }
impl BLApproximationOptions {
    #[inline] fn new() -> Self { Self {
        flattenMode: BLFlattenMode::BL_FLATTEN_MODE_DEFAULT as _,
        offsetMode: BLOffsetMode::BL_OFFSET_MODE_DEFAULT as _, reservedFlags: [0; 6],
        flattenTolerance: 0.20, simplifyTolerance: 0.05, offsetParameter: 0.414213562
    } }
}

struct BLDashArray(BLArrayCore);
impl Drop for BLDashArray {
    #[inline] fn drop(&mut self) { safe_dbg!(blArrayDestroy(&mut self.0)); }
}
impl BLDashArray {  #[cfg(feature = "b2d_sfp")]
    #[inline] pub fn new(data: &[f32]) -> Self { let mut array = object_init();
        safe_dbg!(blArrayInit(&mut array, BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT32));
        safe_dbg!(blArrayAppendData(&mut array, data.as_ptr() as _, data.len()));   Self(array)
    }

    #[cfg(not(feature = "b2d_sfp"))]
    #[inline] pub fn new(data: &[f32]) -> Self { let mut array = object_init();
        safe_dbg!(blArrayInit(&mut array, BLObjectType::BL_OBJECT_TYPE_ARRAY_FLOAT64));
        let data = data.iter().map(|v| *v as f64).collect::<Vec<_>>();
        //safe_dbg!(blArrayAssignData(&mut self.0, data.as_ptr() as _, data.len()));
        safe_dbg!(blArrayAppendData(&mut array, data.as_ptr() as _, data.len()));   Self(array)
    }
}

impl Default for BLMatrix2D { fn default() -> Self { Self::new() } }
impl BLMatrix2D { //  https://blend2d.com/doc/structBLMatrix2D.html
    #[inline] pub fn new() -> Self {
        let mut mat = object_init();
        safe_dbg!(blMatrix2DSetIdentity(&mut mat));     mat
    }
    #[inline] pub fn setTranslation(&mut self, pos: &BLPoint) {
        safe_dbg!(blMatrix2DSetTranslation(self, pos.x, pos.y));
    }
    #[inline] pub fn setScaling(&mut self, sx: f32, sy: f32) {
        safe_dbg!(blMatrix2DSetScaling(self, sx as _, sy as _));
    }
    #[inline] pub fn setSkewing(&mut self, skew: &BLPoint) {
        safe_dbg!(blMatrix2DSetSkewing(self, skew.x, skew.y));
    }
    #[inline] pub fn setRotation(&mut self, radius: f32, origin: &BLPoint) {
        safe_dbg!(blMatrix2DSetRotation(self, radius as _, origin.x, origin.y));
    }

    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blMatrix2DApplyOp(self, BLTransformOp::BL_TRANSFORM_OP_TRANSFORM,
            mat as *const _ as _));
    }
    #[inline] pub fn postTransform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blMatrix2DApplyOp(self, BLTransformOp::BL_TRANSFORM_OP_POST_TRANSFORM,
            mat as *const _ as _));
    }
    #[inline] pub fn reset(&mut self) {
        safe_dbg!(blMatrix2DApplyOp(self, BLTransformOp::BL_TRANSFORM_OP_RESET, null()));
    }
    #[inline] pub fn setIdentity(&mut self) { safe_dbg!(blMatrix2DSetIdentity(self)); }
    #[inline] pub fn invert(&mut self, src: &Self) { safe_dbg!(blMatrix2DInvert(self, src)); }

    #[inline] pub fn getScaling(&self) -> (f32, f32) {
        let mat = unsafe {
            ptr::read(&self.__bindgen_anon_1.__bindgen_anon_1) };
        (mat.m00 as _, mat.m10 as _)
    }

    #[inline] pub fn mapPointD(&self, pt: &BLPoint) -> BLPoint {
        let mut npt = BLPoint::new();
        safe_dbg!(blMatrix2DMapPointDArray(self, &mut npt, pt, 1));     npt
    }
    #[inline] pub fn mapPointDArray(&self, pts: &mut [BLPoint]) {
        safe_dbg!(blMatrix2DMapPointDArray(self, pts.as_mut_ptr(), pts.as_ptr(), pts.len()));
    }
}

pub struct BLPath(BLPathCore);  //  https://blend2d.com/doc/classBLPath.html
impl Drop for BLPath { #[inline] fn drop(&mut self) { safe_dbg!(blPathDestroy(&mut self.0)); } }
impl BLPath {
    #[inline] pub fn new() -> Self { let mut path = object_init();
        safe_dbg!(blPathInit(&mut path));   Self(path)
    }

    #[inline] pub fn moveTo(&mut self, pt: &BLPoint) {
        safe_dbg!(blPathMoveTo(&mut self.0, pt.x, pt.y));
    }
    #[inline] pub fn lineTo(&mut self, pt: &BLPoint) {
        safe_dbg!(blPathLineTo(&mut self.0, pt.x, pt.y));
    }
    #[inline] pub fn quadTo(&mut self, cp: &BLPoint, pt: &BLPoint) {
        safe_dbg!(blPathQuadTo(&mut self.0, cp.x, cp.y, pt.x, pt.y));
    }
    #[inline] pub fn cubicTo(&mut self, c1: &BLPoint, c2: &BLPoint, pt: &BLPoint) {
        safe_dbg!(blPathCubicTo(&mut self.0, c1.x, c1.y, c2.x, c2.y, pt.x, pt.y));
    }

    #[inline] pub fn arcTo(&mut self, center: &BLPoint, radius: &BLPoint,
        start: f32, sweep: f32) { //, forceMoveTo
        safe_dbg!(blPathArcTo(&mut self.0, center.x, center.y, radius.x, radius.y,
            start as _, sweep as _, false));
    }
    #[inline] pub fn ellipticArcTo(&mut self, rp: &BLPoint, x_rot: f32, // svg_arc_to
        large: bool, sweep: bool, pt: &BLPoint) {
        safe_dbg!(blPathEllipticArcTo(&mut self.0,
            rp.x, rp.y, x_rot as _, large, sweep, pt.x, pt.y));
        //  Adds an elliptic arc to the path that follows the SVG specification.
        //  https://www.w3.org/TR/SVG/paths.html#PathDataEllipticalArcCommands
    }
    #[inline] pub fn arcQuadrantTo(&mut self, corner: &BLPoint, end: &BLPoint) {
        safe_dbg!(blPathArcQuadrantTo(&mut self.0, corner.x, corner.y, end.x, end.y));
    }
    #[inline] pub fn polyTo(&mut self, poly: &[BLPoint]) {
        safe_dbg!(blPathPolyTo(&mut self.0, poly.as_ptr(), poly.len()));
    }

    #[inline] pub fn close(&mut self) { safe_dbg!(blPathClose(&mut self.0)); }
    //#[inline] pub fn clear(&mut self) { safe_dbg!(blPathClear(&mut self.0)); }
    #[inline] pub fn reset(&mut self) { safe_dbg!(blPathReset(&mut self.0)); }

    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blPathTransform(&mut self.0, null(), mat));
    }

    #[inline] pub fn getSize(&self) -> u32 { unsafe { blPathGetSize(&self.0) as _ } }
    #[inline] pub fn getLastVertex(&self) -> Option<BLPoint> {
        let mut pt = BLPoint { x: 0.0, y: 0.0 };
        if is_error(unsafe { blPathGetLastVertex(&self.0, &mut pt) }) { None } else { Some(pt) }
    }
    #[inline] pub fn getBoundingBox(&self) -> Option<BLBox> {   let mut bbox = BLBox::new();
        if is_error(unsafe { blPathGetBoundingBox(&self.0, &mut bbox) }) {
            None } else { Some(bbox) }
    }
    #[inline] pub fn hitTest(&self, pt: &BLPoint, fillRule: BLFillRule) -> BLHitTest {
        unsafe { blPathHitTest(&self.0, pt, fillRule) }
    }

    #[inline] pub fn addGeometry<T: B2DGeometry>(&mut self, geom: &T, mat: &BLMatrix2D) {
        safe_dbg!(blPathAddGeometry(&mut self.0, T::GEOM_T, geom as *const _ as _, mat,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    #[inline] pub fn addPath(&mut self, path: &BLPath) {
        safe_dbg!(blPathAddPath(&mut self.0, &path.0, null()));
    }
    #[inline] pub fn addTransformedPath(&mut self, path: &BLPath, mat: &BLMatrix2D) {
        safe_dbg!(blPathAddTransformedPath(&mut self.0, &path.0, null(), mat));
    }
    #[inline] pub fn addStrokedPath(&mut self, path: &BLPath,
        options: &BLStrokeOptions, approx: &BLApproximationOptions) {
        safe_dbg!(blPathAddStrokedPath(&mut self.0, &path.0, null(), &options.0, approx));
    }

    #[inline] pub fn addRect(&mut self, rect: &BLRect) {
        safe_dbg!(blPathAddRectD(&mut self.0, rect,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }
    #[inline] pub fn addBox (&mut self, bbox: &BLBox) {
        safe_dbg!(blPathAddBoxD(&mut self.0, bbox,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }
}

impl BLLine {
    #[inline] pub fn new(s: &BLPoint, e: &BLPoint) -> Self {
        Self { x0: s.x, y0: s.y, x1: e.x, y1: e.y }
    }
}
impl BLArc {
    #[inline] pub fn new(c: &BLPoint, r: &BLPoint, start: f32, sweep: f32) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.x, ry: r.y, start: start as _, sweep: sweep as _ }
    }
}
impl BLCircle {
    #[inline] pub fn new(c: &BLPoint, r: f32) -> Self { Self { cx: c.x, cy: c.y, r: r as _ } }
}
impl BLEllipse {
    #[inline] pub fn new(c: &BLPoint, r: &BLPoint) -> Self {
        Self { cx: c.x, cy: c.y, rx: r.x, ry: r.y }
     }
}
impl BLTriangle {
    #[inline] pub fn new(a: &BLPoint, b: &BLPoint, c: &BLPoint) -> Self {
        Self { x0: a.x, y0: a.y, x1: b.x, y1: b.y, x2: c.x, y2: c.y }
     }
}
impl BLRoundRect {
    #[inline] pub fn new(rect: &BLRect, radius: f32) -> Self {
        Self { x: rect.x, y: rect.y, w: rect.w, h: rect.h, rx: radius as _, ry: radius as _ }
    }
}

impl BLBox   { #[inline] pub fn new() -> Self { Self { x0: 0., y0: 0., x1: 0., y1: 0. } } }
impl BLRect  { #[inline] pub fn new() -> Self { Self { x : 0., y : 0., w: 0., h: 0. } } }
impl BLPoint {
    #[inline] pub fn new() -> Self { Self { x : 0., y : 0. } }
    #[cfg(feature = "b2d_sfp")] #[inline] pub fn x(&self) -> f32 { self.x }
    #[cfg(feature = "b2d_sfp")] #[inline] pub fn y(&self) -> f32 { self.y }
    #[cfg(not(feature = "b2d_sfp"))] #[inline] pub fn x(&self) -> f64 { self.x }
    #[cfg(not(feature = "b2d_sfp"))] #[inline] pub fn y(&self) -> f64 { self.y }
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
    #[inline] fn from(v: (f64, f64)) -> Self { Self { x: v.0, y: v.1 } }
}
impl From<(f32, f32)> for BLPoint {
    #[inline] fn from(v: (f32, f32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(u32, u32)> for BLPoint {
    #[inline] fn from(v: (u32, u32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(i32, i32)> for BLPoint {
    #[inline] fn from(v: (i32, i32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<BLPoint> for (f32, f32) {
    #[inline] fn from(val: BLPoint) -> Self { (val.x as _, val.y as _) }
}
impl Clone for BLPoint { #[inline] fn clone(&self) -> Self { *self } }
impl Copy  for BLPoint {}

impl From<(i32, i32)> for BLSizeI {
    #[inline] fn from(v: (i32, i32)) -> Self { Self { w: v.0, h: v.1 } }
}
impl From<(u32, u32)> for BLSizeI {
    #[inline] fn from(v: (u32, u32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f32, f32)> for BLSize  {
    #[inline] fn from(v: (f32, f32)) -> Self { Self { w: v.0 as _, h: v.1 as _ } }
}
impl From<(f64, f64, f64, f64)> for BLBox {
    #[inline] fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(f32, f32, f32, f32)> for BLBox {
    #[inline] fn from(v: (f32, f32, f32, f32)) -> Self {
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
impl From<(f32, f32, f32, f32)> for BLRect {
    #[inline] fn from(v: (f32, f32, f32, f32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(u32, u32, u32, u32)> for BLRect {
    #[inline] fn from(v: (u32, u32, u32, u32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}

impl From<u32> for BLRgba32 { #[inline] fn from(value: u32) -> Self { Self {   value } } }
impl From<(u8, u8, u8, u8)> for BLRgba32 {  // (r, g, b, a) -> 0xAARRGGBB
    #[inline] fn from(val: (u8, u8, u8, u8)) -> Self { Self { value:
        ((val.3 as u32) << 24) | ((val.0 as u32) << 16) | ((val.1 as u32) << 8) | (val.2 as u32)
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

impl From<BLRgba32> for BLRgba64 { #[inline] fn from(v: BLRgba32)  -> Self { v.value.into() } }
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
    #[inline] fn drop(&mut self) { safe_dbg!(blGradientDestroy(&mut self.0)); }
}

impl BLGradient {
    #[inline] pub fn new<T: B2DGradient>(gv: &T/*, stops: &[BLGradientStop]*/) -> Self {
        let mut grd = object_init();
        safe_dbg!(blGradientInitAs(&mut grd, T::GR_TYPE, gv as *const _ as _,
            BLExtendMode::BL_EXTEND_MODE_PAD, null(), 0, null()));   Self(grd)
    }                                         //stops.as_ptr(), stops.len(),

    #[inline] pub fn addStop(&mut self, offset: f32, color: BLRgba32) {
        safe_dbg!(blGradientAddStopRgba32(&mut self.0, offset as _, color.value));
    }

    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blGradientApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }
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
    #[inline] pub fn new(p0: &BLPoint, p1: &BLPoint) -> Self {
        Self { x0: p0.x, y0: p0.y, x1: p1.x, y1: p1.y }
    }
}
impl BLRadialGradientValues { // center/focal point
    #[inline] pub fn new(cp: &BLPoint, fp: &BLPoint, r0: f32, r1: f32) -> Self {
        Self { x0: cp.x, y0: cp.y, x1: fp.x, y1: fp.y, r0: r0 as _, r1: r1 as _ }
    }
}
impl BLConicGradientValues {
    #[inline] pub fn new(pt: &BLPoint, angle: f32, repeat: f32) -> Self {
        Self { x0: pt.x, y0: pt.y, angle: angle as _, repeat: repeat as _ }
    }
}

pub struct BLSolidColor(BLVarCore);
impl Drop for BLSolidColor {
    #[inline] fn drop(&mut self) { safe_dbg!(blVarDestroy(&mut self.0 as *mut _ as _)); }
}
impl BLSolidColor {
    #[inline] pub fn initRgba32(rgba32: BLRgba32) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(blVarInitRgba32(&mut color as *mut _ as _,  rgba32.value));   Self(color)
    }
    #[inline] pub fn initRgba64(rgba64: BLRgba64) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(blVarInitRgba64(&mut color as *mut _ as _,  rgba64.value));   Self(color)
    }
    #[inline] pub fn initRgba  (rgba:   BLRgba) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(blVarInitRgba  (&mut color as *mut _ as _, &rgba));           Self(color)
    }
}

pub trait B2DStyle {}
impl B2DStyle for BLPattern {}
impl B2DStyle for BLGradient {}
impl B2DStyle for BLSolidColor {}
// Style could be BLRgba, BLRgba32, BLRgba64, BLGradient, BLPattern, and BLVar.

pub struct BLPattern(BLPatternCore);
impl Drop for BLPattern {
    #[inline] fn drop(&mut self) { safe_dbg!(blPatternDestroy(&mut self.0)); }
}
impl BLPattern {
    #[inline] pub fn new(img: &BLImage) -> Self { let mut pat = object_init();
        safe_dbg!(blPatternInitAs(&mut pat, &img.0, null(),
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
        let mut ctx = BLContext::new(&mut img);     //ctx.clearAll();

        let mut radial = BLGradient::new(&BLRadialGradientValues::new(
            &(180, 180).into(), &(180, 180).into(), 180.0, 0.));
        radial.addStop(0.0, 0xFFFFFFFF.into());
        radial.addStop(1.0, 0xFFFF6F3F.into());

        ctx.fillGeometryExt(&BLCircle::new(&(180, 180).into(), 160.0), &radial);

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            &(195, 195).into(), &(470, 470).into()));
        linear.addStop(0.0, 0xFFFFFFFF.into());
        linear.addStop(1.0, 0xFF3F9FFF.into());

        ctx.setCompOp(BLCompOp::BL_COMP_OP_DIFFERENCE);
        ctx.fillGeometryExt(&BLRoundRect::new(&(195, 195, 270, 270).into(), 25.0), &linear);
        //ctx.setCompOp(BLCompOp::BL_COMP_OP_SRC_OVER);   // restore to default

        let _ = img.writeToFile("target/logo_b2d.png");
    }

    #[test] fn minimal_demo() {
        let mut img = BLImage::new(512, 512, BLFormat::BL_FORMAT_PRGB32);
        let mut ctx = BLContext::new(&mut img);
        let mut path = BLPath::new();

        path. moveTo(&( 26,  31).into());
        path.cubicTo(&(642, 132).into(), &(587, -136).into(), &(25, 464).into());
        path.cubicTo(&(882, 404).into(), &(144,  267).into(), &(27,  31).into());

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            &(0, 0).into(), &(0, 480).into()));
        linear.addStop(0.0, 0xFFFFFFFF.into());
        linear.addStop(0.5, 0xFFFF1F7F.into());
        linear.addStop(1.0, 0xFF1F7FFF.into());

        ctx.setStrokeWidth(10.0);
        //ctx.setStrokeMiterLimit(4.0);
        ctx.setStrokeCaps(BLStrokeCap ::BL_STROKE_CAP_ROUND);
        ctx.setStrokeJoin(BLStrokeJoin::BL_STROKE_JOIN_ROUND);

        ctx.fillGeometryRgba32(&path, 0xFFFFFFFF.into());
        ctx.strokeGeometryExt (&path, &linear);
        let _ = img.writeToFile("target/demo_b2d.png");
        //BLContext::show_rtinfo();
    }
}

