
![Build status](https://github.com/mhfan/intvg/actions/workflows/rust-ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/mhfan/intvg/graph/badge.svg)](https://codecov.io/gh/mhfan/intvg)
[![Crates.io](https://img.shields.io/crates/v/intvg.svg)](https://crates.io/crates/intvg)
[![dependency status](https://deps.rs/repo/github/mhfan/intvg/status.svg)](https://deps.rs/repo/github/mhfan/intvg)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

# Another Rust implementation for [TinyVG](https://tinyvg.tech/) image format

A library/tool to load/save binary .tvg file, rendering to .png file powered by [tiny-skia](https://github.com/RazrFalcon/tiny-skia), parsing from .svg file powered by [usvg](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg), with least external dependencies, simple and concise Rust code.

TinyVG textual representation (.tvgt) is only for debugging purposes which can be easily achieved with ```#derivei[(Debug)]``` and ```println!("{:?}, commands)``` or ```dbg!(commands)``` in Rust.

## Binding [GPAC/EVG 2D rendering engine](https://github.com/gpac/gpac/tree/master/src/evg) (with Fixed-point math)

## Binding [Blend2D rendering engine](https://github.com/blend2d/blend2d) (with single precision)

