
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};

    //let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
    //    ""), Some(1_000))?;     ptys.exp_eof()?;

    let mut tvg = TVGImage::new();
    assert!(tvg.load(&mut BufReader::new(File::open("data/tiger.svg")?))
        .is_err_and(|e| { eprintln!("{e}\n{e:?}"); true }));    // coverage TVGError

    for entry in fs::read_dir("data")? {
        let entry = entry?;
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) { continue }
        let path = entry.path();

        //if  path.as_os_str() != "data/tiger.tvg" { continue }       // to test specific file
        if let Some(ext) = path.extension() {
            if ext == "tvg" {   println!("{}: ", path.display());
                let mut tvg = TVGImage::new();
                tvg.load(&mut BufReader::new(File::open(&path)?))?;
                tvg.render(1.0)?.save_png("target/foo.png")?;
                // XXX: binary compare and reload?
            } else
            if ext == "svg" {   println!("{}: ", path.display());
                use usvg::TreeParsing;
                let tree = usvg::Tree::from_data(&fs::read(&path)?,
                    &usvg::Options::default())?;
                let mut tvg = TVGImage::from_usvg(&tree);
                tvg.save(&mut BufWriter::new(File::create("target/foo.tvg")?))?;
                tvg.render(1.0)?.save_png("target/foo.png")?;
            }
        }
    };    Ok(())
}

