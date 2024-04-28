/****************************************************************
 * $ID: amanithvg.rs  	Thu 02 Nov 2023 17:31:23+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#![allow(non_snake_case)] #![allow(non_camel_case_types)]
#![allow(unused)] #![allow(clippy::too_many_arguments)]

//pub mod openvg {   // https://www.amanithvg.com // avg_bindings.rs
mod ovg_ffi { include!(concat!(env!("OUT_DIR"), "/openvg.rs")); }    use ovg_ffi::*;

const VG_PATH_FORMAT_STANDARD: i32 = 0;

//  https://github.com/weekit/WeeKit/blob/master/src/draw.rs
//  Utilities for drawing text and shapes.

//use font::*;

pub struct Canvas { w: u32, h: u32, }   // Represents a drawing area.

impl Canvas {
    pub fn new(w: u32, h: u32) -> Canvas {  // Creates a new Canvas and clear to white.
        let canvas = Canvas { w, h };
        canvas.background(255, 255, 255); reset();
        unsafe { vgLoadIdentity(); }    canvas
    }

    /// Clears the canvas to a solid background color.
    pub fn background(&self, r: u8, g: u8, b: u8) { self.background_rgba(r, g, b, 1.0); }

    /// Clears the canvas to a background color with alpha.
    pub fn background_rgba(&self, r: u8, g: u8, b: u8, a: f32) {
        unsafe {    let color = rgba(r, g, b, a);
            vgSetfv(VGParamType::VG_CLEAR_COLOR, 4, color.as_ptr());
            vgClear(0, 0, self.w as _, self.h as _);
        }
    }

    /// Clears the window to previously set background colour.
    pub fn window_clear(&self) { unsafe { vgClear(0, 0, self.w as _, self.h as _); } }

    /// Clears a given rectangle in window coordinates (unaffected by transformations).
    pub fn area_clear(x: u32, y: u32, w: u32, h: u32) {
        unsafe { vgClear(x as _, y as _, w as _, h as _); }
    }
}

/// Resets drawing colors to black and stroke width to zero.
pub fn reset() {
    set_fill  (&rgba(0, 0, 0, 1.0));
    set_stroke(&rgba(0, 0, 0, 1.0));
    stroke_width(0.0);
}

/*  Returns the width of a text string at the specified font and size.
pub fn text_width(s: &str, f: &Font, pointsize: u32) -> f32 {
    let (size, mut tw) = (pointsize as VGfloat, 0.0);
    for c in s.chars() {
        let glyph_index = f.character_map[c as _];
        if  glyph_index != -1 {
            tw += size * f.glyph_advances[glyph_index as _] as _ / 65536.0;
        }
    }   tw
}

/// Renders a string of text at a specified location, size, using the specified font glyphs.
pub fn text(x: VGfloat, y: VGfloat, s: &str, f: &Font, pointsize: u32) {
    let (mut mm, mut xx, size) = ([0.0; 9], x, pointsize as _);
    unsafe {    vgGetMatrix(&mut mm as _);
        for c in s.chars() {
            let glyph_index = f.character_map[c as _];
            if  glyph_index == -1 { continue; }
            let mat = [size, 0.0, 0.0, 0.0, size, 0.0, xx, y, 1.0];

            vgLoadMatrix(&mm  as _);
            vgMultMatrix(&mat as _);
            vgDrawPath(f.glyphs[glyph_index as _],
                VGPaintMode::VG_FILL_PATH as _ | VGPaintMode::VG_STROKE_PATH as _);

            xx += size * f.glyph_advances[glyph_index as _] as _ / 65536.0;
        }   vgLoadMatrix(&mm as _);
    }
}

/// Draws text centered on (x,y).
pub fn text_mid(x: VGfloat, y: VGfloat, s: &str, f: &Font, pointsize: u32) {
    text(x - (text_width(s, f, pointsize) / 2.0), y, s, f, pointsize);
}

/// Draws text with its end aligned to (x,y).
pub fn text_end(x: VGfloat, y: VGfloat, s: &str, f: &Font, pointsize: u32) {
    text(x - text_width(s, f, pointsize), y, s, f, pointsize);
}

/// Reports a font's height.
pub fn text_height(f: &Font, pointsize: u32) -> VGfloat {
    (f.font_height * pointsize as i32) as _ / 65536.0
}

/// Reports a font's depth (how far under the baseline it goes).
pub fn text_depth(f: &Font, pointsize: u32) -> VGfloat {
    (-f.descender_height * pointsize as i32) as _ / 65536.0
} */

// Transformations

/// Rotates the coordinate system around angle r.
pub fn rotate(r: VGfloat) { unsafe { vgRotate(r); } }

/// Translates the coordinate system to x,y.
pub fn translate(x: VGfloat, y: VGfloat) { unsafe { vgTranslate(x, y); } }

/// Shears the x coordinate by x degrees, the y coordinate by y degrees.
pub fn shear(x: VGfloat, y: VGfloat) { unsafe { vgShear(x, y); } }

pub fn scale(x: VGfloat, y: VGfloat) { unsafe { vgScale(x, y); } }  // Scales by x, y.

// Style functions

fn set_fill(color: &[VGfloat]) {    /// Sets the fill color.
    unsafe {    let fill_paint = vgCreatePaint();
        vgSetParameteri (fill_paint, VGPaintParamType::VG_PAINT_TYPE  as _,
            VGPaintType::VG_PAINT_TYPE_COLOR as _);
        vgSetParameterfv(fill_paint, VGPaintParamType::VG_PAINT_COLOR as _, 4, color.as_ptr());
        vgSetPaint(fill_paint, VGPaintMode::VG_FILL_PATH as _);
        vgDestroyPaint(fill_paint);
    }
}

fn set_stroke(color: &[VGfloat]) {  /// Sets the stroke color.
    unsafe {    let stroke_paint = vgCreatePaint();
        vgSetParameteri (stroke_paint, VGPaintParamType::VG_PAINT_TYPE as _,
            VGPaintType::VG_PAINT_TYPE_COLOR as _);
        vgSetParameterfv(stroke_paint, VGPaintParamType::VG_PAINT_COLOR as _, 4, color.as_ptr());
        vgSetPaint(stroke_paint, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPaint(stroke_paint);
    }
}

pub fn stroke_width(width: VGfloat) { unsafe {  /// Sets the stroke width.
    vgSetf(VGParamType::VG_STROKE_LINE_WIDTH, width);
    vgSeti(VGParamType::VG_STROKE_CAP_STYLE, VGCapStyle::VG_CAP_BUTT as _);
    vgSeti(VGParamType::VG_STROKE_JOIN_STYLE, VGJoinStyle::VG_JOIN_MITER as _);
} }

// Color functions

/// Fills a color vectors from a RGBA quad.
pub fn rgba(r: u8, g: u8, b: u8, a: VGfloat) -> [VGfloat; 4] {
    let mut color = [0.0, 0.0, 0.0, 1.0];
    color[0] = r as VGfloat / 255.0;
    color[1] = g as VGfloat / 255.0;
    color[2] = b as VGfloat / 255.0;
    //if 0.0 <= a && a <= 1.0 { color[3] = a; }   color
    if (0.0..=1.0).contains(&a) { color[3] = a; }   color
}

/// Returns a solid color from a RGB triple.
pub fn rgb(r: u8, g: u8, b: u8) -> [VGfloat; 4] { rgba(r, g, b, 1.0) }

/// Sets color stops for gradients.
pub fn set_stop(paint: VGPaint, stops: &[VGfloat], n: i32) { unsafe {
    vgSetParameteri (paint, VGPaintParamType::VG_PAINT_COLOR_RAMP_SPREAD_MODE as _,
        VGColorRampSpreadMode::VG_COLOR_RAMP_SPREAD_REPEAT as _);
    vgSetParameteri (paint, VGPaintParamType::VG_PAINT_COLOR_RAMP_PREMULTIPLIED as _,
        VGboolean::VG_FALSE as _);
    vgSetParameterfv(paint, VGPaintParamType::VG_PAINT_COLOR_RAMP_STOPS as _,
        5 * n, stops.as_ptr());
    vgSetPaint(paint, VGPaintMode::VG_FILL_PATH as _);
} }

/// Fills with a linear gradient.
pub fn fill_linear_gradient(x1: VGfloat, y1: VGfloat, x2: VGfloat, y2: VGfloat,
    stops: &[VGfloat], ns: i32) {
    unsafe {    let paint = vgCreatePaint();
        let lgcoord = [x1, y1, x2, y2];
        vgSetParameteri (paint, VGPaintParamType::VG_PAINT_TYPE as _,
            VGPaintType::VG_PAINT_TYPE_LINEAR_GRADIENT as _);
        vgSetParameterfv(paint, VGPaintParamType::VG_PAINT_LINEAR_GRADIENT as _,
            lgcoord.len() as _, lgcoord.as_ptr());
        set_stop(paint, stops, ns);
        vgDestroyPaint(paint);
    }
}

/// Fills with a radial gradient.
pub fn fill_radial_gradient(cx: VGfloat, cy: VGfloat, fx: VGfloat, fy: VGfloat,
    radius: VGfloat, stops: &[VGfloat], ns: i32) {
    unsafe {    let paint = vgCreatePaint();
        let rgcoord = [cx, cy, fx, fy, radius];
        vgSetParameteri (paint, VGPaintParamType::VG_PAINT_TYPE as _,
            VGPaintType::VG_PAINT_TYPE_RADIAL_GRADIENT as _);
        vgSetParameterfv(paint, VGPaintParamType::VG_PAINT_RADIAL_GRADIENT as _,
            rgcoord.len() as _, rgcoord.as_ptr());
        set_stop(paint, stops, ns);
        vgDestroyPaint(paint);
    }
}

/// Limits the drawing area to specified rectangle.
pub fn clip_rect(x: VGint, y: VGint, w: VGint, h: VGint) {
    unsafe {    let coords = [x, y, w, h];
        vgSeti (VGParamType::VG_SCISSORING, VGboolean::VG_TRUE as _);
        vgSetiv(VGParamType::VG_SCISSOR_RECTS, coords.len() as _, coords.as_ptr());
    }
}

/// Stops limiting drawing area to specified rectangle.
pub fn clip_end() { unsafe { vgSeti(VGParamType::VG_SCISSORING, VGboolean::VG_FALSE as _); } }

// Shape functions

/// Creates a path for internal use.
fn new_path() -> VGPath { unsafe {
    vgCreatePath(VG_PATH_FORMAT_STANDARD, VGPathDatatype::VG_PATH_DATATYPE_F,
        1.0, 0.0, 0, 0, VGPathCapabilities::VG_PATH_CAPABILITY_APPEND_TO as _)
} }

/// Makes path data using specified segments and coordinates.
pub fn make_curve(segments: &[VGubyte], coords: &[VGfloat], flags: VGbitfield) {
    unsafe {    let path = new_path();
        vgAppendPathData(path, segments.len() as _, segments.as_ptr(), coords.as_ptr() as _);
        vgDrawPath(path, flags);
        vgDestroyPath(path);
    }
}

/// Makes a cubic bezier curve.
pub fn cbezier(sx: VGfloat, sy: VGfloat, cx: VGfloat, cy: VGfloat,
    px: VGfloat, py: VGfloat, ex: VGfloat, ey: VGfloat) {
    let segments = [
        VGPathCommand::VG_MOVE_TO_ABS as _, VGPathSegment::VG_CUBIC_TO as _ ];
    make_curve(&segments, &[sx, sy, cx, cy, px, py, ex, ey],
        VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
}

/// Makes a quadratic bezier curve.
pub fn qbezier(sx: VGfloat, sy: VGfloat, cx: VGfloat, cy: VGfloat, ex: VGfloat, ey: VGfloat) {
    let segments = [
        VGPathCommand::VG_MOVE_TO_ABS as _, VGPathSegment::VG_QUAD_TO as _ ];
    make_curve(&segments, &[sx, sy, cx, cy, ex, ey],
        VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
}

/// Interleaves arrays of x, y into a single array.
pub fn interleave(x: &[VGfloat], y: &[VGfloat], n: i32, points: &mut [VGfloat]) {
    for i in 0..(n as _) {
        points[2 * i] = x[i];
        points[2 * i + 1] = y[i];
    }
}

/// Makes either a polygon or polyline.
pub fn poly(x: &[VGfloat], y: &[VGfloat], n: VGint, flag: VGbitfield) {
    let mut points = vec![0.0f32; (n as usize) * 2];
    interleave(x, y, n, points.as_mut_slice());
    unsafe {    let path = new_path();
        vguPolygon(path, points.as_ptr(), n, VGboolean::VG_FALSE);
        vgDrawPath(path, flag);
        vgDestroyPath(path);
    }
}

/// Makes a filled polygon with vertices in x, y arrays.
pub fn polygon(x: &[VGfloat], y: &[VGfloat], n: i32) {
    poly(x, y, n, VGPaintMode::VG_FILL_PATH as _);
}

/// Makes a polyline with vertices at x, y arrays.
pub fn polyline(x: &[VGfloat], y: &[VGfloat], n: i32) {
    poly(x, y, n, VGPaintMode::VG_STROKE_PATH as _);
}

/// Makes a rectangle at the specified location and dimensions.
pub fn rect(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat) {
    unsafe {    let path = new_path();
        vguRect(path, x, y, w, h);
        vgDrawPath(path, VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
        vgDestroyPath(path);
    }
}

/// Makes a line from (x1,y1) to (x2,y2).
pub fn line(x1: VGfloat, y1: VGfloat, x2: VGfloat, y2: VGfloat) {
    unsafe {    let path = new_path();
        vguLine(path, x1, y1, x2, y2);
        vgDrawPath(path, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPath(path);
    }
}

/// Makes a rounded rectangle at the specified location and dimensions.
pub fn round_rect(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat, rw: VGfloat, rh: VGfloat) {
    unsafe {    let path = new_path();
        vguRoundRect(path, x, y, w, h, rw, rh);
        vgDrawPath(path, VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
        vgDestroyPath(path);
    }
}

/// Makes an ellipse at the specified location and dimensions.
pub fn ellipse(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat) {
    unsafe {    let path = new_path();
        vguEllipse(path, x, y, w, h);
        vgDrawPath(path, VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
        vgDestroyPath(path);
    }
}

/// Makes a circle at the specified location and dimensions.
pub fn circle(x: VGfloat, y: VGfloat, r: VGfloat) { ellipse(x, y, r, r); }

pub fn square(x: VGfloat, y: VGfloat, r: VGfloat) {    rect(x, y, r, r); }

/// Makes an elliptical arc at the specified location and dimensions.
pub fn arc(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat, sa: VGfloat, aext: VGfloat) {
    unsafe {    let path = new_path();
        vguArc(path, x, y, w, h, sa, aext, VGUArcType::VGU_ARC_OPEN);
        vgDrawPath(path, VGPaintMode::VG_FILL_PATH as u32 | VGPaintMode::VG_STROKE_PATH as u32);
        vgDestroyPath(path);
    }
}

// Outlined shapes
// Hollow shapes -because filling still happens even with a fill of 0,0,0,0
// unlike where using a strokewidth of 0 disables the stroke.
// Either this or change the original functions to require the VG_x_PATH flags

/// Makes a cubic bezier curve, stroked.
pub fn cbezier_outline(sx: VGfloat, sy: VGfloat, cx: VGfloat, cy: VGfloat,
    px: VGfloat, py: VGfloat, ex: VGfloat, ey: VGfloat) {
    let segments = [
        VGPathCommand::VG_MOVE_TO_ABS as _, VGPathSegment::VG_CUBIC_TO as _ ];
    make_curve(&segments, &[sx, sy, cx, cy, px, py, ex, ey], VGPaintMode::VG_STROKE_PATH as _);
}

/// Makes a quadratic bezier curve, outlined.
pub fn qbezier_outline(sx: VGfloat, sy: VGfloat, cx: VGfloat, cy: VGfloat,
    ex: VGfloat, ey: VGfloat) {
    let segments = [
        VGPathCommand::VG_MOVE_TO_ABS as _, VGPathSegment::VG_QUAD_TO as _ ];
    make_curve(&segments, &[sx, sy, cx, cy, ex, ey], VGPaintMode::VG_STROKE_PATH as _);
}

/// Makes a rectangle at the specified location and dimensions, outlined.
pub fn rect_outline(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat) {
    unsafe {    let path = new_path();
        vguRect(path, x, y, w, h);
        vgDrawPath(path, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPath(path);
    }
}

/// Makes a rounded rectangle at the specified location and dimensions, outlined.
pub fn roundrect_outline(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat,
    rw: VGfloat, rh: VGfloat) {
    unsafe {    let path = new_path();
        vguRoundRect(path, x, y, w, h, rw, rh);
        vgDrawPath(path, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPath(path);
    }
}

/// Makes an ellipse at the specified location and dimensions, outlined.
pub fn ellipse_outline(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat) {
    unsafe {    let path = new_path();
        vguEllipse(path, x, y, w, h);
        vgDrawPath(path, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPath(path);
    }
}

/// Makes a circle at the specified location and dimensions, outlined.
pub fn circle_outline(x: VGfloat, y: VGfloat, r: VGfloat) { ellipse_outline(x, y, r, r); }

/// Makes an elliptical arc at the specified location and dimensions, outlined.
pub fn arc_outline(x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat, sa: VGfloat, aext: VGfloat) {
    unsafe {    let path = new_path();
        vguArc(path, x, y, w, h, sa, aext, VGUArcType::VGU_ARC_OPEN);
        vgDrawPath(path, VGPaintMode::VG_STROKE_PATH as _);
        vgDestroyPath(path);
    }
}

pub struct OVGpoint { x: VGfloat, y: VGfloat }
pub struct OVGrect  { x: VGfloat, y: VGfloat, w: VGfloat, h: VGfloat }

//}

#[cfg(test)] mod tests { use super::*;
    #[test] fn setup() { // XXX:
        let commands = [ VGPathSegment::VG_MOVE_TO,  VGPathSegment::VG_CUBIC_TO,
                         VGPathSegment::VG_CUBIC_TO, VGPathSegment::VG_CUBIC_TO,
                         VGPathSegment::VG_CUBIC_TO, VGPathSegment::VG_CLOSE_PATH ];

        let coordinates = [              236.0f32, 276.0f32, // move  to
             56.0f32, 426.0f32,  56.0f32,  86.0f32, 236.0f32, 236.0f32, // cubic to
             86.0f32,  56.0f32, 426.0f32,  56.0f32, 276.0f32, 236.0f32, // cubic to
            456.0f32,  86.0f32, 456.0f32, 426.0f32, 276.0f32, 276.0f32, // cubic to
            426.0f32, 456.0f32,  86.0f32, 456.0f32, 236.0f32, 276.0f32, // cubic to
        ];

        unsafe {
            let exts = core::ffi::CStr::from_ptr(
                vgGetString(VGStringID::VG_EXTENSIONS) as _);
            if let Ok(ext) = exts.to_str() {
                println!("OpenVG extensions supported (AmanithVG):");
                ext.split_whitespace().for_each(|s| println!("  {s}"));
            }

            let path = vgCreatePath(VG_PATH_FORMAT_STANDARD,
                VGPathDatatype::VG_PATH_DATATYPE_F, 1.0f32, 0.0f32, 0, 0,
                VGPathCapabilities::VG_PATH_CAPABILITY_ALL as _);
            vgAppendPathData(path, commands.len() as _,
                commands.as_ptr() as _, coordinates.as_ptr() as _);

            vgSeti(VGParamType::VG_FILL_RULE, VGFillRule::VG_EVEN_ODD as _);
        }
    }
}

