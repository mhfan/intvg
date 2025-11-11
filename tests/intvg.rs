
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};
    use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::PathBuf};
    let idir = PathBuf::from("target/images");
    let tigr = PathBuf::from("data/tiger.svg");
    fs::create_dir_all(&idir)?;

    /* let bexe = std::env::var("CARGO_BIN_EXE_intvg")?;
    for fname in ["foo.tvg", "foo.png"] {
        let mut cmd = std::process::Command::new(&bexe);
        let tigr = if fname.ends_with("png") {
            &tigr.with_extension("tvg") } else { &tigr };
        let outp = idir.join(fname);    fs::remove_file(&outp)?;
        cmd.args([tigr.to_str().unwrap(), outp.to_str().unwrap()]);
        rexpect::session::spawn_command(cmd, Some(1_000))?.exp_eof()?;
    } */

    assert!(TVGImage::load_data(&mut BufReader::new(File::open(tigr)?))
        .is_err_and(|e| { eprintln!("{e}\n{e:?}"); true }));    // coverage TVGError

    for entry in fs::read_dir("data")? { let entry = entry?;
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) { continue }
        let path = entry.path();    println!("Test {}:", path.display());

        //if  path.as_os_str() != "data/tiger.tvg" { continue }   // to test specific file
        let Some(ext) = path.extension() else { continue };
        let tvg = if ext == "tvg" {
            TVGImage::load_data(&mut BufReader::new(File::open(&path)?))?
        } else if ext == "svg" { TVGImage::from_usvg(&fs::read(&path)?)?
        } else { continue }; // "data/*.png"

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap();
        let outp = idir.join(stem).with_extension("tvg");
        tvg.save_data(&mut BufWriter::new(File::create(&outp)?))?;

        let img = tvg.render(1.0)?;
        #[cfg(feature = "evg")] intvg::render_evg::Render::render(&tvg, 1.0)?;
        #[cfg(feature = "b2d")] intvg::render_b2d::Render::render(&tvg, 1.0)?;
        img.save_png(outp.with_extension("png"))?;

        TVGImage::load_data(&mut BufReader::new(File::open(&outp)?)).inspect_err(
            |_| eprintln!("Fail to load `{}'", outp.display()))?;
    }   Ok(())
}

