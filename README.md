
[![codecov](https://codecov.io/gh/mhfan/intvg/graph/badge.svg)](https://codecov.io/gh/mhfan/intvg)
[![Crates.io](https://img.shields.io/crates/v/intvg.svg)](https://crates.io/crates/intvg)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

# Another Rust implementation for [TinyVG](https://tinyvg.tech/) image format

A library/tool to load/save binary .tvg file, rendering to .png file powered by [tiny-skia](https://github.com/RazrFalcon/tiny-skia), parsing from .svg file powered by [usvg](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg), with least external dependencies, simple and concise Rust code.

TinyVG textual representation (.tvgt) is only for debugging purposes which can be easily achieved with ```#derivei[(Debug)]``` and ```println!("{:?}, commands)``` or ```dbg!(commands)``` in Rust.
