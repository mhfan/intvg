
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};
    use std::{fs::{self, File}, io::{BufReader, BufWriter}};

    use usvg::{TreeParsing, TreeTextToPath};
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    /* fs::remove_file("target/foo.tvg").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.svg target/foo.tvg"), Some(1_000))?;   ptys.exp_eof()?;
    fs::remove_file("target/foo.png").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.tvg target/foo.png"), Some(1_000))?;   ptys.exp_eof()?; */

    assert!(TVGImage::new().load(&mut BufReader::new(File::open("data/tiger.svg")?))
        .is_err_and(|e| { eprintln!("{e}\n{e:?}"); true }));    // coverage TVGError

    let mut time_render = 0f32;
    for entry in fs::read_dir("data")? {    let entry = entry?;
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) { continue }
        let path = entry.path();

        //if  path.as_os_str() != "data/tiger.tvg" { continue }   // to test specific file
        let ext = path.extension().unwrap();
        let mut tvg = if ext == "tvg" {
            let mut tvg = TVGImage::new();
            tvg.load(&mut BufReader::new(File::open(&path)?))?; tvg
        } else if ext == "svg" {
            let mut tree = usvg::Tree::from_data(&fs::read(&path)?,
                &usvg::Options::default())?;    tree.convert_text(&fontdb);
            TVGImage::from_usvg(&tree)
        } else { continue }; // "data/*.png"

        let stem = path.file_stem().unwrap().to_str().unwrap();
        tvg.save(&mut BufWriter::new(File::create(format!("target/{}.tvg", stem))?))?;
        let tnow = std::time::Instant::now();
        let img = tvg.render(1.0)?;
        #[cfg(feature = "evg")] intvg::render_evg::Render::render(&tvg, 1.0)?;
        #[cfg(feature = "b2d")] intvg::render_b2d::Render::render(&tvg, 1.0)?;
        let timing = tnow.elapsed().as_secs_f32();  time_render += timing;
        println!("{}: rendering {:.2} fps", path.display(), 1.0 / timing);
        img.save_png(format!("target/{}.png", stem))?;
    }   println!("All rendering: {:.3} s", time_render);

    Ok(())
}

