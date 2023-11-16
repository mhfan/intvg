/****************************************************************
 * $ID: micro-gl.cpp  	Tue 14 Nov 2023 14:10:21+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

//  https://micro-gl.github.io/docs/microgl/concepts/math

#include "microgl.h"

//#include "micro-alloc/dynamic_memory.h"
//#include "micro-alloc/linear_memory.h"
//#include "micro-alloc/stack_memory.h"
//#include "micro-alloc/pool_memory.h"
//#include "micro-alloc/std_memory.h"

//unsigned char memory[5000];
//dynamic_memory<> alloc{memory, sizeof(memory)};
//linear_memory<> alloc{memory, sizeof(memory)};
//stack_memory<> alloc{memory, sizeof(memory)};
//pool_memory<> alloc{memory, sizeof(memory), 256, 8, true};
//static std_memory alloc;

#include <new> // required by `Placement new`

path_t* path_new() { return new path_t(); } // new (path) path_t();
void path_del(path_t *path) { delete path; }
void path_close(path_t *path) { path->closePath(); }
void path_clear(path_t *path) { path->clear(); }

void path_moveto (path_t *path, const vertex *end) { path->moveTo(*end); }
void path_lineto (path_t *path, const vertex *end) { path->lineTo(*end); }
void path_quadto (path_t *path, const vertex *cp0, const vertex *end) {
    path->quadraticCurveTo(*cp0, *end);
}
void path_cubicto(path_t *path, const vertex *cp1, const vertex *cp2, const vertex *end) {
    path->cubicBezierCurveTo(*cp1, *cp2, *end);
}
void path_ellipse(path_t *path, const vertex *center, const vertex *radius,
    const number rotation, const vertex* angle, bool acw) {
    path->ellipse(*center, radius->x, radius->y, rotation, angle->x, angle->y, acw);
}
//const vertex* path_lastpoint(path_t *path) { return &path->lastPointOfCurrentSubPath(); }

void canvas_del(canvas_t *cvs) { delete cvs; }
canvas_t* canvas_new(unsigned width, unsigned height) { return new canvas_t(width, height); }
void canvas_clear(canvas_t *cvs, const color_t *color) { cvs->clear(*color); }

using namespace sampling;
#include <initializer_list>

void canvas_fill(canvas_t *cvs, path_t *path, const stencil *sten) {
    cvs->drawPathFill  <stencil::BlendMode, stencil::PorterDuff, stencil::antialias>(
        flat_color<> {{255,0,255,255}}, // FIXME:
        sten->transform, *path, sten->rule, sten->quality, sten->opacity);
}

void canvas_stroke(canvas_t *cvs, path_t *path, const stencil *sten, const stroker *pen) {
    //fast_radial_gradient<number, 4, canvas_t::rgba, precision::high> radial{0.5, 0.5, 0.5};
    //angular_linear_gradient<number, 4, canvas_t::rgba, precision::high> gradient{0};
    cvs->drawPathStroke<stencil::BlendMode, stencil::PorterDuff, stencil::antialias>(
        flat_color<> {{255,0,255,255}},
        sten->transform, *path, pen->width, pen->cap, pen->join, pen->miter_limit,
        std::initializer_list<int>{ 50, 50 },
        pen->dash_offset, sten->opacity);
}

