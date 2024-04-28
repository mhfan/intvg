
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg_attr(coverage_nightly, coverage(off))] //#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};

    eprintln!(r"{} v{}-g{}, {}, {} 🦀", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),
        env!("BUILD_GIT_HASH"), env!("BUILD_TIMESTAMP"), env!("CARGO_PKG_AUTHORS"));
        //build_time::build_time_local!("%H:%M:%S%:z %Y-%m-%d"),    //option_env!("ENV_VAR_NAME");

    let (cnt, mut args) = (env::args().count(), env::args());
    if   cnt < 2 { println!("Usage: {} <path-to-svg/tvg> [<path-to-tvg/png>] [-R <b2d|evg>]\n",
            args.next().unwrap());  return Ok(())
    }   // all unwrap are safe

    let mut path = args.nth(1).unwrap();
    let tvg = if path.ends_with(".tvg") {
        TVGImage::load_data(&mut BufReader::new(File::open(&path)?))?
    } else if path.ends_with(".svg") { TVGImage::from_svgd(&fs::read(&path)?)?
    } else { return Err("only support .svg & .tvg files".into()) };

    if 2 < cnt { path = args.next().unwrap(); } else {
        path.replace_range(path.rfind('.').unwrap().., ".png");
    }

    if  path.ends_with(".tvg") {
        tvg.save_data(&mut BufWriter::new(fs::OpenOptions::new()
            .write(true).create_new(true).open(path)?))?;
    } else if path.ends_with(".png") {
        if std::path::Path::new(&path).exists() { return Err("output file exists".into()) }

        let tnow = std::time::Instant::now();
        if args.next().is_some_and(|arg| arg == "-R") {
            let eng = args.next().expect("need to specify a renderer!");

            #[cfg(feature = "b2d")] if eng == "b2d" {
                intvg::render_b2d::Render::render(&tvg, 1.0)?.save_png(path)?; return Ok(())
            }
            #[cfg(feature = "evg")] if eng == "evg" {
                intvg::render_evg::Render::render(&tvg, 1.0)?.save_png(path)?; return Ok(())
            }   eprintln!("Not support `{eng}' rendering!");
        } else { tvg.render(1.0)?.save_png(path)?; }
        eprintln!("Rendering performance: {:.2} fps", 1.0 / tnow.elapsed().as_secs_f32());
    } else { return Err("unknown output file extension".into()) }   Ok(())
}

