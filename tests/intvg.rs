
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};
    use std::{fs::{self, File}, io::{BufReader, BufWriter}};

    /* fs::remove_file("target/foo.tvg").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.svg target/foo.tvg"), Some(1_000))?;   ptys.exp_eof()?;
    fs::remove_file("target/foo.png").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.tvg target/foo.png"), Some(1_000))?;   ptys.exp_eof()?; */

    assert!(TVGImage::load_data(&mut BufReader::new(File::open("data/tiger.svg")?))
        .is_err_and(|e| { eprintln!("{e}\n{e:?}"); true }));    // coverage TVGError

    for entry in fs::read_dir("data")? { let entry = entry?;
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) { continue }
        let path = entry.path();    println!("Test {}:", path.display());

        //if  path.as_os_str() != "data/tiger.tvg" { continue }   // to test specific file
        let ext = path.extension().unwrap();
        let tvg = if ext == "tvg" {
            TVGImage::load_data(&mut BufReader::new(File::open(&path)?))?
        } else if ext == "svg" { TVGImage::from_usvg(&fs::read(&path)?)?
        } else { continue }; // "data/*.png"

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let tvgf = format!("target/{}.tvg", stem);
        tvg.save_data(&mut BufWriter::new(File::create(&tvgf)?))?;

        let img = tvg.render(1.0)?;
        #[cfg(feature = "evg")] intvg::render_evg::Render::render(&tvg, 1.0)?;
        #[cfg(feature = "b2d")] intvg::render_b2d::Render::render(&tvg, 1.0)?;
        img.save_png(format!("target/{}.png", stem))?;

        TVGImage::load_data(&mut BufReader::new(File::open(&tvgf)?)).inspect_err(
            |_| eprintln!("Fail to load `{}'", &tvgf))?;
    }   Ok(())
}

