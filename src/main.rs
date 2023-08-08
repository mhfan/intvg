
#![cfg_attr(coverage_nightly, feature(no_coverage))]

#[cfg_attr(coverage_nightly, no_coverage)] //#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};

    eprintln!(r"{} v{}-g{}, {}, {} 🦀", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),
        env!("BUILD_GIT_HASH"), env!("BUILD_TIMESTAMP"), env!("CARGO_PKG_AUTHORS"));
        //build_time::build_time_local!("%H:%M:%S%:z %Y-%m-%d"),    //option_env!("ENV_VAR_NAME");

    let (cnt, mut args) = (env::args().count(), env::args());
    if   cnt < 2 { println!("Usage: {} <path-to-svg/tvg> [<path-to-tvg/png>]\n",
            args.next().unwrap());  return Ok(())
    }   // all unwrap are safe

    let mut path = args.nth(1).unwrap();
    let mut tvg = if path.ends_with(".svg") {
        use usvg::{TreeParsing, TreeTextToPath};
        let mut tree = usvg::Tree::from_data(&fs::read(&path)?, &usvg::Options::default())?;
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();     tree.convert_text(&fontdb);
        TVGImage::from_usvg(&tree)
    } else if path.ends_with(".tvg") {
        let mut tvg = TVGImage::new();
        tvg.load(&mut BufReader::new(File::open(&path)?))?;     tvg
    } else { return Err("only .svg & .tvg file is supported".into()) };

    if 2 < cnt { path = args.next().unwrap(); } else {
        path.replace_range(path.rfind('.').unwrap().., ".png");
    }

    if  path.ends_with(".tvg") {
        tvg.save(&mut BufWriter::new(fs::OpenOptions::new()
            .write(true).create_new(true).open(path)?))?;
    } else if path.ends_with(".png") {
        if std::path::Path::new(&path).exists() { return Err("output file exists".into()) }
        tvg.render(1.0)?.save_png(path)?;
    } else { return Err("unknown output file extension".into()) }   Ok(())
}

