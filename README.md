
![Build status](https://github.com/mhfan/intvg/actions/workflows/rust-ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/mhfan/intvg/graph/badge.svg)](https://codecov.io/gh/mhfan/intvg)
[![Crates.io](https://img.shields.io/crates/v/intvg.svg)](https://crates.io/crates/intvg)
[![dependency status](https://deps.rs/repo/github/mhfan/intvg/status.svg)](https://deps.rs/repo/github/mhfan/intvg)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

# Another [TinyVG](https://tinyvg.tech/) image parser & renderer in Rust

A library/tool to load/save binary .tvg file, rendering to .png file powered by [tiny-skia](https://github.com/RazrFalcon/tiny-skia), parsing from .svg file powered by [usvg](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg), with least external dependencies, simple and concise Rust code.

TinyVG textual representation (.tvgt) is only for debugging/development purposes which can be easily achieved with `#[derive(Debug)]` and `println!("{:?}, commands)` or `dbg!(commands)` in Rust.

## Binding [Blend2D](https://github.com/blend2d/blend2d) rendering engine

Build controlled by feature `"b2d"`.

```bash
    (cd 3rdparty && ./layout.sh)    # just run once

    cargo r -F b2d -- <path-to-svg/tvg> [<path-to-tvg/png>]
```

A built-in Rust binding for Blend2D, simpler than outdated [blend2d-rs](https://github.com/Veykril/blend2d-rs).

## Binding [GPAC/EVG](https://github.com/gpac/gpac/tree/master/src/evg) 2D rendering engine

Build controlled by feature `"evg"`.

```bash
    (cd 3rdparty && ./layout.sh)    # just run once

    cargo r -F evg -- <path-to-svg/tvg> [<path-to-tvg/png>]
```

Built-in seems the only one Rust binding in public for GPAC/EVG.

## A SVG Renderer/Viewer powered by [Femtovg](https://github.com/Femtovg/Femtovg)

Build controlled by feature `"nvg"`.

```bash
    cargo r -F nvg --bin nanovg -- <path-to-svg>
```

Refer to the SVG example in Femotovg, updated, rewritten and optimized the code, added Drag & Drop support.

## Another SVG/TinyVG Viewer based on [HTML5/Web 2D Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API)

Run the following command in the [wcnvs](wcnvs/README.md) sub-crate/project to start the Dioxus dev server:

```bash
dx serve --hot-reload
```

- Open the browser to <http://localhost:8080>

## Rendering performance comparison

|  2D Engine |   Timing | Performance | Note             |
|        --: |      --: |         --: | :--              |
|   Femtovg: |  1.65 ms |   606.8 fps | max performance  |
|   Blend2D: |  1.94 ms |   515.5 fps | float/double     |
| CanvasAPI: |  2.00 ms |   500.0 fps | bad T accuracy?  |
| tiny_skia: |  5.10 ms |   195.9 fps |                  |
|  GPAC/EVG: |  7.96 ms |   125.6 fps | floating point   |
|  GPAC/EVG: | 18.07 ms |    55.3 fps | fixed-point math |

Test rendering [Ghostscript_Tiger.svg](https://commons.wikimedia.org/wiki/File:Ghostscript_Tiger.svg) on _MacBook Pro_ with _Apple M1 Pro_.

- <https://www.nan.fyi/svg-paths>
