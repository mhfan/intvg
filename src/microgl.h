/****************************************************************
 * $ID: micro-gl.hpp  	Mon 13 Nov 2023 13:42:03+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

#include "microgl/canvas.h"
#include "microgl/bitmaps/bitmap.h"
#include "microgl/pixel_coders/RGB565_PACKED_16.h"
#include "microgl/pixel_coders/RGB888_PACKED_32.h"
#include "microgl/pixel_coders/RGBA8888_PACKED_32.h"

#include "microgl/samplers/flat_color.h"
#include "microgl/samplers/fast_radial_gradient.h"
//#include "microgl/samplers/line_linear_gradient.h"
//#include "microgl/samplers/axial_linear_gradient.h"
#include "microgl/samplers/angular_linear_gradient.h"
//#include "microgl/samplers/linear_gradient_2_colors.h"

using namespace microtess;

using number = float;
//using number = Q<15, microgl::ints::int32_t>;

using path_t = path<number>;
//using path_t = path<number, std::vector>;

using canvas_t = canvas<bitmap<RGBA8888_PACKED_32>>;
//using canvas_t = canvas<bitmap<RGB565_PACKED_16>>;
//using canvas_t = canvas<bitmap<RGB888_PACKED_32>>;

using vertex = vec2<number>;

struct stencil {
    static const bool antialias = true;
    using BlendMode = blendmode::Normal;
    using PorterDuff = porterduff::FastSourceOverOnOpaque;

    //const Sampler sampler;
    matrix_3x3<number> transform;
    tess_quality quality;   // fill_rule::even_odd
    fill_rule rule;         // tess_quality::prettier_with_extra_vertices
    unsigned char opacity;
};

struct stroker {
    number width;
    stroke_cap cap;         // stroke_cap::round
    stroke_line_join join;  // stroke_line_join::round
    unsigned miter_limit;   // 4
    //Iterable dash_array;
    int dash_offset;        // 0
};

extern "C" {

path_t* path_new();
void path_del(path_t* path);
void path_close(path_t* path);
void path_clear(path_t* path);

void path_moveto (path_t* path, const vertex* end);
void path_lineto (path_t* path, const vertex* end);
void path_quadto (path_t* path, const vertex* cp0, const vertex* end);
void path_cubicto(path_t* path, const vertex* cp1, const vertex* cp2, const vertex* end);
void path_ellipse(path_t* path, const vertex* center, const vertex* radius,
        const number rotation, const vertex* angle, bool acw);
//const vertex* path_lastpoint(path_t* path); // private member

canvas_t* canvas_new(unsigned width, unsigned height);
void canvas_clear(canvas_t* cvs, const color_t* color);
void canvas_fill  (canvas_t* cvs, path_t* path, const stencil* sten);
void canvas_stroke(canvas_t* cvs, path_t* path, const stencil* sten, const stroker* pen);
void canvas_del(canvas_t* cvs);

}

