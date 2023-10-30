/****************************************************************
 * $ID: blend2d.rs  	Fri 27 Oct 2023 08:44:33+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#![allow(non_snake_case)] #![allow(unused)] //#![allow(clippy::new_without_default)]

//pub mod blend2d  {    // https://blend2d.com
use core::ptr::null;

pub use b2d_ffi::{BLFormat, BLCompOp, BLStrokeCap, BLStrokeJoin, BLPoint, BLBox, BLRect,
    BLRgba32, BLRgba64, BLLinearGradientValues, BLRadialGradientValues, BLConicGradientValues};

#[allow(non_camel_case_types)] //#[allow(non_upper_case_globals)]     // blend2d_bindings
mod b2d_ffi { include!(concat!(env!("OUT_DIR"), "/blend2d.rs")); }  use b2d_ffi::*;

/*#[macro_export] */macro_rules! safe_dbg { //($v:expr$(,$g:expr)?) => { unsafe { $v } };
    ($v:expr,$g:expr) => { match unsafe { $v } { // as u32
        //eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($v), &res);
        res => { if res != $g { dbg!(res); } res } } };
    ($v:expr) => { safe_dbg!($v, 0) };
}

fn object_init<T>() -> T { // for BLObjectCore
    let mut s = std::mem::MaybeUninit::<T>::uninit();
    unsafe { std::ptr::write_bytes(s.as_mut_ptr(), 0, 1); s.assume_init() }
}

pub struct BLContext(BLContextCore);
impl Drop for BLContext {
    #[inline] fn drop(&mut self) { safe_dbg!(blContextDestroy(&mut self.0)); }
}

impl BLContext {
    #[inline] pub fn new(img: &mut BLImage) -> Self {
        /* let mut  cci: BLContextCreateInfo = object_init();
        let mut info: BLRuntimeSystemInfo = object_init();
        safe_dbg!(blRuntimeQueryInfo(BLRuntimeInfoType::BL_RUNTIME_INFO_TYPE_SYSTEM,
                &mut info as *mut _ as _));
        cci.threadCount = info.threadCount; */ // XXX: depends on BL_BUILD_NO_JIT?

        let mut ctx = object_init();
        safe_dbg!(blContextInitAs(&mut ctx, &mut img.0, null())); // &cci
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

    #[inline] pub fn setStrokeStyle(&mut self, style: &dyn B2DStyle) {
        safe_dbg!(blContextSetStrokeStyle(&mut self.0, style as *const _ as _));
    }

    #[inline] pub fn setStrokeCaps(&mut self, cap: BLStrokeCap) {
        //safe_dbg!(blContextSetStrokeCaps(&mut self.0, cap));
        safe_dbg!(blContextSetStrokeCap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_START, cap.clone()));
        safe_dbg!(blContextSetStrokeCap(&mut self.0,
            BLStrokeCapPosition::BL_STROKE_CAP_POSITION_END, cap));
    }

    #[inline] pub fn setStrokeJoin(&mut self, join: BLStrokeJoin) {
        safe_dbg!(blContextSetStrokeJoin(&mut self.0, join));
    }

    #[inline] pub fn setStrokeMiterLimit(&mut self, miter_limit: f32) {
        safe_dbg!(blContextSetStrokeMiterLimit(&mut self.0, miter_limit as _));
    }

    #[inline] pub fn setStrokeWidth(&mut self, width: f32) {
        safe_dbg!(blContextSetStrokeWidth(&mut self.0, width as _));
    }

    #[inline] pub fn strokeGeometryRgba32<T: B2DGeometry>(&mut self, geom: &T, color: BLRgba32) {
        safe_dbg!(blContextStrokeGeometryRgba32(&mut self.0, T::GEOM_T,
            geom as *const _ as _, color.value));
    }

    #[inline] pub fn strokeGeometryExt<T: B2DGeometry>(&mut self, geom: &T, style: &dyn B2DStyle) {
        safe_dbg!(blContextStrokeGeometryExt(&mut self.0, T::GEOM_T,
            geom as *const _ as _, style as *const _ as _));
    }

    #[inline] pub fn applyTransform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }

    #[inline] pub fn scale(&mut self, sx: f32, sy: f32) {
        #[cfg(feature = "b2d_sfp")] let scale = &[ sx, sy ];
        #[cfg(not(feature = "b2d_sfp"))] let scale = &[ sx as f64, sy as f64 ];
        safe_dbg!(blContextApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_SCALE, scale as *const _ as _));
    }

    #[inline] pub fn setCompOp(&mut self, cop: BLCompOp) {
        safe_dbg!(blContextSetCompOp(&mut self.0, cop));
    }

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
            std::env::consts::ARCH, std::mem::size_of::<*const i32>() * 8, std::env::consts::OS,
            info.compilerInfo.iter().map(|i| char::from(*i as u8)).collect::<String>(),
            info.maxImageSize, info.maxThreadCount, sinfo.threadCount,
            sinfo.threadStackSize, sinfo.allocationGranularity);
    }
}

pub struct BLImage(BLImageCore);
impl Drop for BLImage { #[inline] fn drop(&mut self) { safe_dbg!(blImageDestroy(&mut self.0)); } }
impl BLImage {
    #[inline] pub fn new(w: u32, h: u32, fmt: BLFormat) -> Self {
        let mut img = object_init();
        safe_dbg!(blImageInitAs(&mut img, w as i32, h as i32, fmt));     Self(img)
    }

    #[inline] pub fn readFromFile(file: &str) -> Self {
        let mut img = object_init();
        let file = std::ffi::CString::new(file).unwrap();
        safe_dbg!(blImageReadFromFile(&mut img, file.as_ptr(), null())); Self(img)
    }

    #[inline] pub fn save_png<S: Into<Vec<u8>>>(&self, file: S) -> Result<(), &str> {
        self.writeToFile(file);     Ok(())
    }

    #[inline] pub fn writeToFile<S: Into<Vec<u8>>>(&self, file: S) {
        let file = std::ffi::CString::new(file).unwrap();
        safe_dbg!(blImageWriteToFile(&self.0 , file.as_ptr(), null()));
    }
}

pub struct BLPath(BLPathCore);
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
    #[inline] pub fn ellipticArcTo(&mut self, rp: &BLPoint, x_rot: f32, // svg_arc_to
        large: bool, sweep: bool, pt: &BLPoint) {
        safe_dbg!(blPathEllipticArcTo(&mut self.0,
            rp.x, rp.y, x_rot as _, large, sweep, pt.x, pt.y));
    }

    #[inline] pub fn close(&mut self) { safe_dbg!(blPathClose(&mut self.0)); }
    #[inline] pub fn reset(&mut self) { safe_dbg!(blPathReset(&mut self.0)); }

    #[inline] pub fn getSize(&self) -> u32 { safe_dbg!(blPathGetSize(&self.0)) as u32 }
    #[inline] pub fn getLastVertex(&self) -> Option<BLPoint> {
        let mut pt = BLPoint { x: 0.0, y: 0.0 };
        if safe_dbg!(blPathGetLastVertex(&self.0, &mut pt)) == BLResultCode::BL_SUCCESS as u32 {
            Some(pt) } else { None }
    }

    #[inline] pub fn addGeometry<T: B2DGeometry>(&mut self, geom: &T) {
        safe_dbg!(blPathAddGeometry(&mut self.0, T::GEOM_T, geom as *const _ as _, null(),
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    #[inline] pub fn transform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blPathTransform(&mut self.0, null(), mat as *const _ as _));
    }

    #[inline] pub fn addRectI(&mut self, rect: &BLRectI) {
        safe_dbg!(blPathAddRectI(&mut self.0, rect,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
    }

    #[inline] pub fn addBox(&mut self, bbox: &BLBox) {
        safe_dbg!(blPathAddBoxD(&mut self.0, bbox,
            BLGeometryDirection::BL_GEOMETRY_DIRECTION_CW));
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

pub trait B2DGeometry { const GEOM_T: BLGeometryType; }
impl B2DGeometry for BLPath {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_PATH;
}
impl B2DGeometry for BLLine {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_LINE;
}
impl B2DGeometry for BLBox  {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_BOXD;
}
impl B2DGeometry for BLBoxI {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_BOXI;
}
impl B2DGeometry for BLRect {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_RECTD;
}
impl B2DGeometry for BLRectI {
    const GEOM_T: BLGeometryType = BLGeometryType::BL_GEOMETRY_TYPE_RECTI;
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

impl From<(i32, i32)> for BLPointI { fn from(v: (i32, i32)) -> Self { Self { x: v.0, y: v.1 } } }
impl From<(f64, f64)> for BLPoint  {
    fn from(v: (f64, f64)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(f32, f32)> for BLPoint  {
    fn from(v: (f32, f32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}
impl From<(i32, i32)> for BLPoint  {
    fn from(v: (i32, i32)) -> Self { Self { x: v.0 as _, y: v.1 as _ } }
}

impl From<(f64, f64, f64, f64)> for BLBox {
    fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(f32, f32, f32, f32)> for BLBox {
    fn from(v: (f32, f32, f32, f32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(i32, i32, i32, i32)> for BLBox {
    fn from(v: (i32, i32, i32, i32)) -> Self {
        Self { x0: v.0 as _, y0: v.1 as _, x1: v.2 as _, y1: v.3 as _ }
    }
}
impl From<(i32, i32, i32, i32)> for BLBoxI {
    fn from(v: (i32, i32, i32, i32)) -> Self { Self { x0: v.0, y0: v.1, x1: v.2, y1: v.3 } }
}

impl From<(f64, f64, f64, f64)> for BLRect {
    fn from(v: (f64, f64, f64, f64)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(f32, f32, f32, f32)> for BLRect {
    fn from(v: (f32, f32, f32, f32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(i32, i32, i32, i32)> for BLRect {
    fn from(v: (i32, i32, i32, i32)) -> Self {
        Self { x: v.0 as _, y: v.1 as _, w: v.2 as _, h: v.3 as _ }
    }
}
impl From<(i32, i32, i32, i32)> for BLRectI {
    fn from(v: (i32, i32, i32, i32)) -> Self { Self { x: v.0, y: v.1, w: v.2, h: v.3 } }
}

impl From<BLRgba32> for BLRgba64 { fn from(v: BLRgba32) -> Self { v.value.into() } }
impl From<u32> for BLRgba32 { fn from(value: u32) -> Self {  Self { value } } } // 0xAARRGGBB
impl From<u32> for BLRgba64 {
    fn from(v: u32) -> Self { Self { value:
        ((((v >> 24) as u64) << 48) | ((((v >> 16) & 0xFF) as u64) << 32) |
        ((((v >>  8) & 0xFF) as u64) << 16) |  ((v & 0xFF) as u64)) * 0x0101
    } }
}

pub struct BLGradient(BLGradientCore);
impl Drop for BLGradient {
    #[inline] fn drop(&mut self) { safe_dbg!(blGradientDestroy(&mut self.0)); }
}

impl BLGradient {
    #[inline] pub fn new<T: B2DGradient>(gv: &T/*, stops: &[BLGradientStop]*/) -> Self {
        let mut grd = object_init();
        safe_dbg!(blGradientInitAs(&mut grd, T::GR_TYPE, gv as *const _ as _,
            BLExtendMode::BL_EXTEND_MODE_PAD, null(), 0, //stops.as_ptr(), stops.len(),
            null()));   Self(grd)
    }

    #[inline] pub fn addStop(&mut self, offset: f32, color: BLRgba64) {
        safe_dbg!(blGradientAddStopRgba64(&mut self.0, offset as _, color.value));
    }

    #[inline] pub fn applyTransform(&mut self, mat: &BLMatrix2D) {
        safe_dbg!(blGradientApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_TRANSFORM, mat as *const _ as _));
    }

    #[inline] pub fn scale(&mut self, sx: f32, sy: f32) {
        #[cfg(feature = "b2d_sfp")] let scale = &[ sx, sy ];
        #[cfg(not(feature = "b2d_sfp"))] let scale = &[ sx as f64, sy as f64 ];
        safe_dbg!(blGradientApplyTransformOp(&mut self.0,
            BLTransformOp::BL_TRANSFORM_OP_SCALE, scale as *const _ as _));
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
    #[inline] pub fn new(cp: &BLPoint, fp: &BLPoint, radius: f32) -> Self {
        Self { x0: cp.x, y0: cp.y, x1: fp.x, y1: fp.y, r0: radius as _ }
    }
}

impl BLConicGradientValues {
    #[inline] pub fn new(pt: &BLPoint, angle: f32) -> Self {
        Self { x0: pt.x, y0: pt.y, angle: angle as _ }
    }
}

pub struct BLVar(BLVarCore);
impl Drop for BLVar {
    #[inline] fn drop(&mut self) { safe_dbg!(blVarDestroy(&mut self.0 as *mut _ as _)); }
}

impl BLVar {
    #[inline] pub fn initRgba32(rgba32: BLRgba32) -> Self {
        let mut color: BLVarCore = object_init();
        safe_dbg!(blVarInitRgba32(&mut color as *mut _ as _, rgba32.value));    Self(color)
    }
}

pub trait B2DStyle {}
impl B2DStyle for BLVar {}
impl B2DStyle for BLPattern {}
impl B2DStyle for BLGradient {}
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

// BLFontFace, BLFont, Text, BLGlyphBuffer, BLGlyphRun
// TODO: textPath, to render text along the shape of a path

//}

#[cfg(test)] mod tests { use super::*;
    #[test] fn blend2d_logo() {
        let mut img = BLImage::new(480, 480, BLFormat::BL_FORMAT_PRGB32);
        let mut ctx = BLContext::new(&mut img);

        let mut radial = BLGradient::new(&BLRadialGradientValues::new(
            &(180, 180).into(), &(180, 180).into(), 180.0));
        radial.addStop(0.0, 0xFFFFFFFF.into());
        radial.addStop(1.0, 0xFFFF6F3F.into());

        ctx.fillGeometryExt(&BLCircle::new(&(180, 180).into(), 160.0), &radial);

        let mut linear = BLGradient::new(&BLLinearGradientValues::new(
            &(195, 195).into(), &(470, 470).into()));
        linear.addStop(0.0, 0xFFFFFFFF.into());
        linear.addStop(1.0, 0xFF3F9FFF.into());

        // XXX: not implemented yet? raster/rastercontext.cpp, fillUnclippedMaskD, 3648
        ctx.setCompOp(BLCompOp::BL_COMP_OP_DIFFERENCE); // default is SRC_OVER composition
        ctx.fillGeometryExt(&BLRoundRect::new(&(195, 195, 270, 270).into(), 25.0), &linear);

        img.writeToFile("target/logo_b2d.png");
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
        img.writeToFile("target/demo_b2d.png");
        //BLContext::show_rtinfo();
    }
}

