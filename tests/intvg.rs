
#[test] fn tinyvg() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs::{self, File}, io::{BufReader, BufWriter}};
    use intvg::tinyvg::TVGImage;

    //let mut ptys = rexpect::spawn(concat!(env!("CARGO_BIN_EXE_intvg"),
    //    ""), Some(1_000))?;     ptys.exp_eof()?;

    let mut tvg = TVGImage::new();
    assert!(tvg.load(&mut BufReader::new(File::open("examples/files/tiger.svg")?))
        .is_err_and(|e| { println!("{e}\n{e:?}"); true }));     // coverage TVGError

    fs::read_dir("examples/files")?
        .try_for_each(|entry| -> intvg::tinyvg::Result<()> {
        let entry = entry?;     let path = entry.path();
        if  entry.file_type().is_ok_and(|ft| ft.is_file()) &&
            path.extension().is_some_and(|ext| ext == "tvg") {
            let mut tvg = TVGImage::new();
            tvg.load(&mut BufReader::new(File::open(&path)?))?;
            tvg.save(&mut BufWriter::new(File::create("target/foo.tvg")?))?;
            println!("{}: ", path.display());   tvg.digest(true);
            // XXX: binary compare and reload?
        }   Ok(())
    })?;    Ok(())
}

