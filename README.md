
# Another Rust implementation for [TinyVG](https://tinyvg.tech/) image format

A library/tool to load/save binary .tvg file, rendering to .png file powered by [tiny-skia](https://github.com/RazrFalcon/tiny-skia), parsing from .svg file powered by [usvg](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg), with least external dependencies, simple and concise Rust code.

TinyVG textual representation (.tvgt) is only for debugging purposes which can be easily achieved with ```#derivei[(Debug)]``` and ```println!("{:?}, commands)``` or ```dbg!(commands)``` in Rust.

