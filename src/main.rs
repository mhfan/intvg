
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render};
    use usvg::TreeParsing;

    let uopt = usvg::Options::default();
    let path = env::args().skip(1).next()
        .ok_or("path not specified").unwrap_or("examples/files/tiger.svg".to_owned());
    let _tree = usvg::Tree::from_data(&fs::read(path)?, &uopt)?;

    let path = env::args().skip(1).next()
        .ok_or("path not specified").unwrap_or("examples/files/tiger.tvg".to_owned());
    let mut tvg = TVGImage::new();
    tvg.load(&mut BufReader::new(File::open(path)?))?;
    tvg.render()?.save_png("target/foo.png")?;

    Ok(())
}

//pub fn convert(usvg::Tree) -> TVGImage { }
//pub fn convert(TVGImage) -> usvg::Tree { }

