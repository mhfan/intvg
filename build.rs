
fn main() {     // https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html
    println!("cargo:rerun-if-changed=.git/index");
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"]).output().unwrap();
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", String::from_utf8(output.stdout).unwrap());

    //println!("cargo:rustc-env=BUILD_TIMESTAMP={}",  // XXX: not run on every build
    //    chrono::Local::now().format("%H:%M:%S%:z %Y-%m-%d"));

    //println!("cargo:rerun-if-changed=build.rs");    // XXX: prevent re-run indead
}
