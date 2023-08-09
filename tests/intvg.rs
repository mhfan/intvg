
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};

    use usvg::{TreeParsing, TreeTextToPath};
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    /* std::fs::remove_file("target/foo.tvg").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.svg target/foo.tvg"), Some(1_000))?;   ptys.exp_eof()?;
    std::fs::remove_file("target/foo.png").unwrap();
    let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
        "  data/tiger.tvg target/foo.png"), Some(1_000))?;   ptys.exp_eof()?; */

    let mut tvg = TVGImage::new();
    assert!(tvg.load(&mut BufReader::new(File::open("data/tiger.svg")?))
        .is_err_and(|e| { eprintln!("{e}\n{e:?}"); true }));    // coverage TVGError

    for entry in fs::read_dir("data")? {
        let entry = entry?;
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) { continue }
        let path = entry.path();

        //if  path.as_os_str() != "data/tiger.tvg" { continue }       // to test specific file
        if let Some(ext) = path.extension() {
            let tvg = if ext == "tvg" { println!("{}: ", path.display());
                let mut tvg = TVGImage::new();
                tvg.load(&mut BufReader::new(File::open(&path)?))?;
                tvg.save(&mut BufWriter::new(File::create("target/foo.tvg")?))?;    tvg
                // XXX: binary compare and reload?
            } else if ext == "svg" { println!("{}: ", path.display());
                let mut tree = usvg::Tree::from_data(&fs::read(&path)?,
                    &usvg::Options::default())?;
                tree.convert_text(&fontdb);

                let mut tvg = TVGImage::from_usvg(&tree);
                tvg.save(&mut BufWriter::new(File::create("target/foo.tvg")?))?;    tvg
            } else { continue };
            tvg.render(1.0)?.save_png("target/foo.png")?;
        }
    };    Ok(())
}

