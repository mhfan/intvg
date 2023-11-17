
use std::{env, path::Path};

//  https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");    // XXX: prevent re-run indead
    // By default, cargo always re-run the build script if any file within the package
    // is changed, and no any rerun-if instruction is emitted.
    //println!("cargo:rerun-if-changed=src");
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}",
        chrono::Local::now().format("%H:%M:%S%z %Y-%m-%d"));

    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"]).output()?;
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", String::from_utf8(output.stdout)?);
    println!("cargo:rerun-if-changed={}", Path::new(".git").join("index").display());

    //std::process::Command::new(Path::new("3rdparty").join("layout.sh")).status()?;
    #[allow(unused)] let path = std::path::PathBuf::from(env::var("OUT_DIR")?);
    #[cfg(feature = "ftg")] binding_ftg(&path)?;
    #[cfg(feature = "evg")] binding_evg(&path)?;
    #[cfg(feature = "b2d")] binding_b2d(&path)?;
    #[cfg(feature = "ovg")] binding_ovg(&path)?;
    #[cfg(feature = "ugl")] binding_ugl(&path)?;

    Ok(())
}

#[cfg(feature = "ftg")] fn binding_ftg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug)] struct DoctestComment;
    impl bindgen::callbacks::ParseCallbacks for DoctestComment {
        fn process_comment(&self, comment: &str) -> Option<String> {
            Some(format!("```c,ignore\n{comment}\n```"))  // FIXME:
        }
    }

    let (ftg_dir, module) = (Path::new("3rdparty").join("ftg"), "ftgrays");
    cc::Build::new().flag("-std=c17").flag("-pedantic").define("STANDALONE_", None)
        .define("FALL_THROUGH", "((void)0)").file(ftg_dir.join("ftgrays.c"))
        .files(glob::glob(&format!("{}/stroke/*.c", ftg_dir.display()))?
            .filter_map(Result::ok)).file(ftg_dir.join("ftraster.c"))
        .flag("-Wno-unused").flag("-Wno-implicit-fallthrough")
        .opt_level(3).define("NDEBUG", None).compile(module);

    // The bindgen::Builder is the main entry point to bindgen,
    // and lets you build up options for the resulting bindings.
    bindgen::builder().header(ftg_dir.join("ftgrays.h").to_string_lossy())
        //.header(ftg_dir.join("ftimage.h").to_string_lossy())
        .clang_args(["-DSTANDALONE_", "-DFT_BEGIN_HEADER=", "-DFT_END_HEADER=",
            "-DFT_STATIC_BYTE_CAST(type,var)=(type)(unsigned char)(var)",
        ]).allowlist_item("FT_OUTLINE_.*|FT_RASTER_FLAG_.*|FT_CURVE_TAG.*")
        .allowlist_var("ft_grays_raster").allowlist_type("FT_Outline|FT_Pixel_Mode")
        .allowlist_var("ft_standard_raster").merge_extern_blocks(true)
        .layout_tests(false).derive_copy(false).derive_debug(false)
        //.default_visibility(bindgen::FieldVisibilityKind::PublicCrate)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .parse_callbacks(Box::new(DoctestComment)).generate_comments(false) // XXX:
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(path.join(format!("{module}.rs")))?;

    Ok(())
}

#[cfg(feature = "evg")] fn binding_evg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (evg_dir, module) = (Path::new("3rdparty").join("evg"), "gpac_evg");
    #[allow(unused_mut)] let mut bgen = bindgen::builder();

    let mut cc = cc::Build::new();
    #[cfg(feature = "evg_fixed")] {
        cc.define("GPAC_FIXED_POINT", None);
        bgen = bgen.clang_arg("-DGPAC_FIXED_POINT");
    }

    cc.flag("-std=c17").flag("-Wno-pointer-sign").define("GPAC_DISABLE_LOG", None)
        .flag("-Wno-unused-parameter").define("GPAC_DISABLE_THREADS", None)
        .flag("-Wno-implicit-fallthrough").flag("-Wno-unused")
        .files(glob::glob(&format!("{}/*.c",
            evg_dir.display()))?.filter_map(Result::ok))
        .include(&evg_dir).opt_level(3).define("NDEBUG", None).compile(module);

    bgen.header(evg_dir.join("gpac").join("evg.h").to_string_lossy())
        .clang_args(["-DGPAC_DISABLE_THREADS", &format!("-I{}", evg_dir.display()) ])
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        //.default_visibility(bindgen::FieldVisibilityKind::PublicCrate)
        .allowlist_function("gf_evg_s.*").allowlist_function("gf_path_.*")
        .merge_extern_blocks(true).new_type_alias("Fixed")//.allowlist_item("GF_LINE_.*")
        .layout_tests(false).derive_copy(false).derive_debug(false)
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(path.join(format!("{module}.rs")))?;

    Ok(())
}

#[cfg(feature = "b2d")] fn binding_b2d(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let b2d_src = Path::new("3rdparty").join("blend2d").join("src");
    let jit_src = Path::new("3rdparty").join("asmjit") .join("src");
    #[allow(unused_mut)] let mut bgen = bindgen::builder();
    let module = "blend2d"; // "blend2d_bindings";

    let mut cc = cc::Build::new();
    #[cfg(feature = "b2d_sfp")] { //println!("cargo:rustc-cfg=feature=\"b2d_sfp\"");
        #[cfg(target_arch = "aarch64")] cc.flag("-mno-outline-atomics");
        cc.define("BLEND2D_NO_DFP",  None).flag("-fsingle-precision-constant")
          .define("BL_BUILD_NO_TLS", None).flag("-Wno-uninitialized").compiler("g++-13");
        bgen = bgen.clang_arg("-DBLEND2D_NO_DFP");
    }   blend2d_simd(&mut cc);

    cc.cpp(true).flag("-std=c++17").define("ASMJIT_EMBED", None)
        .define("ASMJIT_NO_STDCXX", None).define("ASMJIT_NO_FOREIGN", None)
        .files(glob::glob(&format!("{}/**/*.cpp",
            jit_src.display()))?.filter_map(Result::ok)).flag("-Wno-uninitialized")
        .files(glob::glob(&format!("{}/**/*.cpp",
            b2d_src.display()))?.filter_map(|f| f.ok().filter(|f|
                f.as_os_str().to_str().is_some_and(|f| !f.contains("_test")))))
        .flag("-fvisibility=hidden").flag("-fno-exceptions").flag("-fno-math-errno")
        .flag("-fmerge-all-constants").flag("-ftree-vectorize").flag("-fno-rtti")
        .flag("-fno-threadsafe-statics").include(&b2d_src).include(jit_src)
        .opt_level(3).define("NDEBUG", None).compile(module);
    //println!("cargo:rustc-link-lib=rt");  // https://blend2d.com/doc/build-instructions.html

    bgen.header(b2d_src.join("blend2d.h").to_string_lossy()).blocklist_item("*Virt")
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .default_non_copy_union_style(bindgen::NonCopyUnionStyle::ManuallyDrop)
        .default_visibility(bindgen::FieldVisibilityKind::PublicCrate)
        .derive_copy(false).derive_debug(false).merge_extern_blocks(true)
        .allowlist_function("bl.*").allowlist_type("BL.*").layout_tests(false)
        //.clang_args(["-x", "c++", "-std=c++17", &format!("-I{b2d_src}")])
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(path.join(format!("{module}.rs")))?;

    fn blend2d_simd(cc: &mut cc::Build) {
        let compiler = cc.get_compiler();
        if  compiler.is_like_msvc() { // refer to blend2d/CMakeLists.txt
            let simd_flag = "-arch:AVX";
            cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl)
                .then(|| cc.flag(simd_flag).define("BL_BUILD_OPT_AVX", None));

            let simd_flag = "-arch:AVX512";
            cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl)
                .then(|| cc.flag(simd_flag).define("BL_BUILD_OPT_AVX512", None));

            let simd_flag = "-arch:AVX2";
            if  cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl) {
                cc.flag(simd_flag).define("BL_BUILD_OPT_AVX2", None)
                .define("__SSE3__", None).define("__SSSE3__", None)
                .define("__SSE4_1__", None).define("__SSE4_2__", None);

                // 64-bit MSVC compiler doesn't like -arch:SSE[2] as it's implicit.
                env::var("TARGET").is_ok_and(|v|
                    !v.contains("x86_64")).then(|| cc.flag("-arch:SSE2"));
            }
        } else { // XXX: https://doc.rust-lang.org/std/arch/index.html
            let simd_flag = "-mavx";
            cc.is_flag_supported(simd_flag).is_ok_and(|yes| yes)
                .then(|| cc.flag(simd_flag).define("BL_BUILD_OPT_AVX", None));

            let simd_flag = "-mavx512bw";
            if  cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl) {
                cc.flag(simd_flag).define("BL_BUILD_OPT_AVX512", None)
                    .flag("-mavx512dq").flag("-mavx512cd").flag("-mavx512vl");
            }
            let simd_flag = "-mavx2";
            if  cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl) {
                cc.flag(simd_flag).define("BL_BUILD_OPT_AVX2", None).flag("-msse2")
                    .flag("-msse3").flag("-mssse3").flag("-msse4.1").flag("-msse4.2");
            } else {
                let simd_flag = "-mfpu=neon-vfpv4";
                cc.is_flag_supported(simd_flag).is_ok_and(|bl| bl)
                    .then(|| cc.flag(simd_flag).define("BL_BUILD_OPT_ASIMD", None));
            }

            //if compiler.is_like_gnu() { cc.flag("-fno-semantic-interposition"); }
        }
    }

    Ok(())
}

#[cfg(feature = "ovg")] fn binding_ovg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut ovg_dir = Path::new("3rdparty").join("amanithvg").join("include");

    //env::var("CARGO_MANIFEST_DIR").unwrap()
    // XXX: need to set environment variable before `cargo r/t`:
    // DYLD_FALLBACK_LIBRARY_PATH=$PWD/3rdparty/amanithvg/lib/macosx/ub/sre/standalone

    bindgen::builder().clang_arg(format!("-I{}", ovg_dir.display()))
        .header(ovg_dir.join("VG").join("vgext.h").to_string_lossy())
        .derive_copy(false).derive_debug(false).merge_extern_blocks(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .allowlist_function("vg.*").allowlist_type("VG.*").layout_tests(false)
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(path.join("openvg.rs"))?;

    ovg_dir.pop(); ovg_dir.push("lib"); ovg_dir.push(env::consts::OS); //ovg_dir.push("ub");
    ovg_dir.push(env::consts::ARCH); ovg_dir.push("sre"); ovg_dir.push("standalone");
    println!("cargo:rustc-link-search=native={}", ovg_dir.display());
    println!("cargo:rustc-link-lib=dylib=AmanithVG");

    Ok(())
}

#[cfg(feature = "ugl")] fn binding_ugl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ugl_inc = Path::new("3rdparty").join("micro-gl").join("include");
    let module = "microgl";

    let mut ugl_cpp = Path::new("src").join(module).with_extension("cpp");
    cc::Build::new().cpp(true).flag("-std=c++17").file(&ugl_cpp)
        .flag("-Wno-unused-parameter").flag("-Wno-unused").flag("-Wno-sign-compare")
        .flag("-Wno-deprecated-copy").flag("-Wno-uninitialized")
        .flag("-Wno-reorder").flag("-Wno-misleading-indentation")
        .include(&ugl_inc).opt_level(3).define("NDEBUG", None).compile(module);
    println!("cargo:rerun-if-changed={}", ugl_cpp.display());

    ugl_cpp.set_extension("h");
    bindgen::builder().header(ugl_cpp.to_string_lossy()).opaque_type("(canvas|path)_t")
        .clang_args(["-x", "c++", "-std=c++17", &format!("-I{}", ugl_inc.display())])
        .derive_copy(false).derive_debug(false).merge_extern_blocks(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .allowlist_function("(canvas|path).*").layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(path.join(format!("{module}.rs")))?;

    Ok(())
}

