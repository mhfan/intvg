
![Build status](https://github.com/mhfan/intvg/actions/workflows/rust-ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/mhfan/intvg/graph/badge.svg)](https://codecov.io/gh/mhfan/intvg)
[![Crates.io](https://img.shields.io/crates/v/intvg.svg)](https://crates.io/crates/intvg)
[![dependency status](https://deps.rs/repo/github/mhfan/intvg/status.svg)](https://deps.rs/repo/github/mhfan/intvg)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

# Another [TinyVG](https://tinyvg.tech/) image parser & renderer in Rust

A library/tool to load/save binary .tvg file, rendering to .png file powered by [tiny-skia](https://github.com/RazrFalcon/tiny-skia), parsing from .svg file powered by [usvg](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg), with least external dependencies, simple and concise Rust code.

TinyVG textual representation (.tvgt) is only for debugging/development purposes which can be easily achieved with `#[derive(Debug)]` and `println!("{:?}, commands)` or `dbg!(commands)` in Rust.

## Binding [GPAC/EVG](https://github.com/gpac/gpac/tree/master/src/evg) 2D rendering engine

Build controlled by feature `"evg"`.

Built-in seems the only one Rust binding for GPAC/EVG.

## Binding [Blend2D](https://github.com/blend2d/blend2d) rendering engine

Build controlled by feature `"b2d"`.

A built-in Rust binding for Blend2D, simpler than outdated [blend2d-rs](https://github.com/Veykril/blend2d-rs).

## Rendering performance comparison

|  2D Engine |   Timing | Performance | Note             |
|        --: |      --: |         --: | :--              |
|   Blend2D: |  1.94 ms |   515.5 fps | float/double     |
| tiny_skia: |  5.10 ms |   195.9 fps |                  |
|  GPAC/EVG: |  7.96 ms |   125.6 fps | floating point   |
|  GPAC/EVG: | 18.07 ms |    55.3 fps | fixed-point math |

Test rendering [Ghostscript_Tiger.svg](https://commons.wikimedia.org/wiki/File:Ghostscript_Tiger.svg) on _MacBook Pro_ with _Apple M1 Pro_.

* <https://www.nan.fyi/svg-paths>

