
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};
    use usvg::TreeParsing;

    let path = env::args().skip(1).next()
        .ok_or("path not specified").unwrap_or("examples/files/tiger.svg".to_owned());
    let tree = usvg::Tree::from_data(&fs::read(path)?, &usvg::Options::default())?;
    let mut tvg = TVGImage::from_usvg(&tree);
    tvg.save(&mut BufWriter::new(File::create("target/foo.tvg")?))?;

    let path = env::args().skip(1).next()
        .ok_or("path not specified").unwrap_or("examples/files/tiger.tvg".to_owned());
    let mut tvg = TVGImage::new();
    tvg.load(&mut BufReader::new(File::open(path)?))?;
    tvg.render(1.0)?.save_png("target/foo.png")?;

    Ok(())
}

//pub fn convert(usvg::Tree) -> TVGImage { }
//pub fn convert(TVGImage) -> usvg::Tree { }

