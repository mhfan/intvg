
use std::{env, path::{Path, PathBuf}, process::Command};

//  https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");    // XXX: prevent re-run indead
    // By default, cargo always re-run the build script if any file within the package
    // is changed, and no any rerun-if instruction is emitted.
    //println!("cargo:rerun-if-changed=src");
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}",
        chrono::Local::now().format("%H:%M:%S%z %Y-%m-%d"));

    let output = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output()?;
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", String::from_utf8(output.stdout)?);
    println!("cargo:rerun-if-changed={}", PathBuf::from(".git/index").display());

    let status = Command::new(PathBuf::from("3rdparty/layout.sh")).status()?;
    if !status.success() { return Err("3rdparty/layout.sh failed".into()) }
    #[allow(unused)] let odir = PathBuf::from("target/bindings");
    std::fs::create_dir_all(&odir)?;    // env::var("OUT_DIR")?

    #[cfg(feature = "b2d")] binding_b2d(&odir)?;
    #[cfg(feature = "evg")] binding_evg(&odir)?;
    #[cfg(feature = "ftg")] binding_ftg(&odir)?;
    #[cfg(feature = "ovg")] binding_ovg(&odir)?;
    #[cfg(feature = "ugl")] binding_ugl(&odir)?;

    Ok(())
}

#[cfg(feature = "ftg")] fn binding_ftg(odir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug)] struct DoctestComment;
    impl bindgen::callbacks::ParseCallbacks for DoctestComment {
        fn process_comment(&self, comment: &str) -> Option<String> {
            Some(format!("```c,ignore\n{comment}\n```"))  // FIXME:
        }
    }

    let (ftg_dir, module) = (PathBuf::from("3rdparty/ftg"), "ftgrays");
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
        .generate()?.write_to_file(odir.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "evg")] fn binding_evg(odir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (evg_dir, module) = (PathBuf::from("3rdparty/evg"), "gpac_evg");
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
        .generate()?.write_to_file(odir.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "b2d")] fn configure_b2d(build: &mut cc::Build, b2d_src: &Path, jit_src: &Path,
    base_flags: &[&str], build_defines: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    build.cpp(true).flag("-std=c++17").define("NDEBUG", None)
        .define("ASMJIT_STATIC", None).define("ASMJIT_NO_STDCXX", None)
        .define("ASMJIT_NO_FOREIGN", None).define("ASMJIT_ABI_NAMESPACE", "abi_bl")
        .include(b2d_src.parent().unwrap()).include(jit_src.parent().unwrap())
        .include(b2d_src).include(jit_src).opt_level(3);
    for flag in base_flags { build.flag(flag); }
    for define in build_defines { build.define(define, None); }

    let compiler = build.get_compiler();
    if  compiler.is_like_msvc() {
        build.flag("-MP").flag("-GR-").flag("-GF").flag("-W4")
            .flag("-Zc:__cplusplus").flag("-Zc:inline").flag("-GS-")
            .flag("-Zc:strictStrings").flag("-Zc:threadSafeInit-").flag("-Oi")
            .flag_if_supported("-Zc:arm64-aliased-neon-types-");
        if compiler.is_like_clang() {
            build.flag("-clang:-fno-rtti").flag("-clang:-fno-math-errno")
                 .flag("-clang:-fno-trapping-math");
        }
    } else {
        #[cfg(feature = "b2d_sfp")] if compiler.is_like_gnu() {
            //build.compiler("g++");  // XXX: required
            build.define("BLEND2D_NO_DFP", None).flag("-fsingle-precision-constant");
        }
        #[cfg(not(feature = "b2d_sfp"))]
        build.flags(["-Wall", "-Wextra", "-Wconversion", "-Wdouble-promotion"]);
        for flag in ["-Wduplicated-cond", "-Wduplicated-branches", "-Wlogical-op",
            "-Wlogical-not-parentheses", "-Wrestrict", "-Wbidi-chars=any"] {
            build.flag_if_supported(flag);
        }
        build.flag("-fno-exceptions").flag("-fno-rtti").flag("-fvisibility=hidden")
            .flag("-fno-math-errno").flag("-fno-threadsafe-statics")
            .flag("-fmerge-all-constants").flag_if_supported("-ftree-vectorize")
            .flag_if_supported("-mllvm").flag_if_supported("--disable-loop-idiom-all");
        if env::var("CARGO_CFG_TARGET_VENDOR")? != "apple" {
            build.flag_if_supported("-fno-semantic-interposition");
        }
        if env::var("CARGO_CFG_TARGET_OS")? != "ios" {
            build.flag("-fno-trapping-math").flag("-fno-finite-math-only")
                .flag_if_supported("-fno-enforce-eh-specs");
        }
    }   Ok(())
}

#[cfg(feature = "b2d")] fn binding_b2d(odir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (target_arch, module) = (env::var("CARGO_CFG_TARGET_ARCH")?, "blend2d");
    let mut b2d_src = PathBuf::from("3rdparty/blend2d/blend2d");
    let mut jit_src = PathBuf::from("3rdparty/asmjit/asmjit");
    if !b2d_src.exists() { b2d_src.set_file_name("src"); }
    if !jit_src.exists() { jit_src.set_file_name("src"); }

    let compiler = cc::Build::new().get_compiler();
    let is_x86 = target_arch == "x86" || target_arch == "x86_64";
    let is_arm = target_arch == "arm" || target_arch == "aarch64";
    let (mut base_flags, mut groups) = (Vec::new(), Vec::<B2dGroup>::new());
    // https://doc.rust-lang.org/reference/conditional-compilation.html
    // https://doc.rust-lang.org/std/arch/index.html

    struct B2dGroup {
        name: &'static str,
          flags: &'static [&'static str],
        defines: &'static [&'static str],
        files: Vec<PathBuf>, enabled: bool,
    }

    impl B2dGroup {
        fn new(name: &'static str, flags: &'static [&'static str],
            defines: &'static [&'static str]) -> Self {
            Self { name, flags, defines, files: Vec::new(), enabled: false }
        }
    }

    const B2D_OPT_NAMES: [&str; 11] = [
        "sse2", "sse3", "ssse3", "sse4_1", "sse4_2",
        "avx", "avx2", "avx2fma", "avx512",
        "asimd", "asimd_crypto"
    ];

    if is_x86 {
        let is_msvc = compiler.is_like_msvc();
        if target_arch == "x86" {
            if is_msvc { base_flags.push("-arch:SSE2"); } else {
                if compiler.is_like_gnu() { base_flags.push("-mfpmath=sse"); }
                base_flags.push("-msse2");
            }
        }

        if is_msvc && compiler.is_like_clang() {
            groups.extend([
                B2dGroup::new("sse2", &[], &[]),
                B2dGroup::new("sse3",   &["-msse3"],   &[]),
                B2dGroup::new("ssse3",  &["-mssse3"],  &[]),
                B2dGroup::new("sse4_1", &["-msse4.1"], &[]),
                B2dGroup::new("sse4_2", &["-msse4.2", "-mpopcnt", "-mpclmul"], &[]),
                B2dGroup::new("avx",   &["-arch:AVX", "-mpopcnt", "-mpclmul"], &[]),
                B2dGroup::new("avx2",
                    &["-arch:AVX2",   "-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
                B2dGroup::new("avx2fma",
                    &["-arch:AVX2",   "-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2", "-mfma"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2", "BL_TARGET_OPT_FMA"]),
                B2dGroup::new("avx512",
                    &["-arch:AVX512", "-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
            ]);
        } else if is_msvc {
            groups.extend([
                B2dGroup::new("sse2",   &[], &[]),
                B2dGroup::new("sse3",   &[], &["__SSE3__"]),
                B2dGroup::new("ssse3",  &[], &["__SSSE3__"]),
                B2dGroup::new("sse4_1", &[], &["__SSE4_1__"]),
                B2dGroup::new("sse4_2", &[], &["__SSE4_2__"]),
                B2dGroup::new("avx",    &["-arch:AVX"], &[]),
                B2dGroup::new("avx2",   &["-arch:AVX2"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
                B2dGroup::new("avx2fma",&["-arch:AVX2"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2", "BL_TARGET_OPT_FMA"]),
                B2dGroup::new("avx512", &["-arch:AVX512"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
            ]);
        } else {
            groups.extend([
                B2dGroup::new("sse2", &[], &[]),
                B2dGroup::new("sse3",   &["-msse3"], &[]),
                B2dGroup::new("ssse3",  &["-mssse3"], &[]),
                B2dGroup::new("sse4_1", &["-msse4.1"], &[]),
                B2dGroup::new("sse4_2", &["-mpopcnt", "-mpclmul", "-msse4.2"], &[]),
                B2dGroup::new("avx",    &["-mpopcnt", "-mpclmul", "-mavx"], &[]),
                B2dGroup::new("avx2",   &["-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2", "-mavx2"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
                B2dGroup::new("avx2fma",
                    &["-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2", "-mavx2", "-mfma"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2", "BL_TARGET_OPT_FMA"]),
                B2dGroup::new("avx512",
                    &["-mpopcnt", "-mpclmul", "-mbmi", "-mbmi2",
                      "-mavx512f", "-mavx512bw", "-mavx512dq", "-mavx512cd", "-mavx512vl"],
                    &["BL_TARGET_OPT_POPCNT", "BL_TARGET_OPT_BMI2"]),
            ]);
        }
    } else if is_arm {
        let asimd_flags: &'static [&'static str] =
            if target_arch == "arm" { &["-mfpu=neon-vfpv4"] } else { &[] };
        groups.push(B2dGroup::new("asimd", asimd_flags, &["BL_TARGET_OPT_ASIMD"]));
        if target_arch == "aarch64" {
            groups.push(B2dGroup::new("asimd_crypto", &["-march=armv8-a+aes+crc+crypto"],
                &["BL_TARGET_OPT_ASIMD_CRYPTO"]));
        }
    }

    let mut baseline = Vec::new();
    for path in glob::glob(&format!("{}/**/*.cpp", b2d_src.display()))?
        .filter_map(Result::ok).filter(|path|
            !path.file_stem().and_then(|v| v.to_str())
                .is_some_and(|stem| stem.contains("_test"))) {
        let stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or_default();
        let opt_name = B2D_OPT_NAMES.iter().find(|name| stem.ends_with(&format!("_{name}")));
        match opt_name.and_then(|name| groups.iter_mut().find(|group| group.name == *name)) {
            Some(group) => group.files.push(path),
            None if opt_name.is_none() => baseline.push(path),
            None => {}
        }
    }

    let mut lower_groups_available = true;
    for group in &mut groups {
        let mut check = cc::Build::new();   check.cpp(true);
        group.enabled = lower_groups_available && group.flags.iter().all(|flag|
            check.is_flag_supported(flag).unwrap_or(false));
        if group.name != "avx2fma" { lower_groups_available = group.enabled; }
    }

    let mut build_defines = Vec::new();
    let enabled = |name| groups.iter().any(|group| group.name == name && group.enabled);
    if is_x86 {
        for (name, define) in [
            ("avx512", "BL_BUILD_OPT_AVX512"), ("avx2",   "BL_BUILD_OPT_AVX2"),
            ("sse4_2", "BL_BUILD_OPT_SSE4_2"), ("avx",    "BL_BUILD_OPT_AVX"),
            ("sse4_1", "BL_BUILD_OPT_SSE4_1"), ("ssse3",  "BL_BUILD_OPT_SSSE3"),
            ("sse3",   "BL_BUILD_OPT_SSE3"),   ("sse2",   "BL_BUILD_OPT_SSE2")] {
            if enabled(name) { build_defines.push(define); break; }
        }
    } else {
        if enabled("asimd") { build_defines.push("BL_BUILD_OPT_ASIMD"); }
        if enabled("asimd_crypto") { build_defines.push("BL_BUILD_OPT_ASIMD_CRYPTO"); }
    }

    let mut base = cc::Build::new();
    configure_b2d(&mut base, &b2d_src, &jit_src, &base_flags, &build_defines)?;
    base.files(baseline).files(glob::glob(&format!("{}/**/*.cpp",
        jit_src.display()))?.filter_map(Result::ok)).compile(module);

    for group in groups.iter().filter(|group| group.enabled && !group.files.is_empty()) {
        let mut build = cc::Build::new();   build.flags(group.flags);
        configure_b2d(&mut build, &b2d_src, &jit_src, &base_flags, &build_defines)?;
        for &define in group.defines { build.define(define, None); }
        build.files(&group.files).compile(&format!("blend2d_{}", group.name));
    }
    //println!("cargo:rustc-link-lib=rt");  // https://blend2d.com/doc/build-instructions.html

    #[allow(unused_mut)]  let mut bgen = bindgen::builder();
    #[cfg(feature = "b2d_sfp")] if compiler.is_like_gnu() {
        bgen = bgen.clang_arg("-DBLEND2D_NO_DFP");
    }
    bgen.header(b2d_src.join("blend2d.h").to_string_lossy())
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .default_non_copy_union_style(bindgen::NonCopyUnionStyle::ManuallyDrop)
        .default_visibility(bindgen::FieldVisibilityKind::PublicCrate)
        .derive_copy(false).derive_debug(false).merge_extern_blocks(true)
        //.derive_hash(false).derive_partialeq(false).derive_eq(false) // XXX: not work for enum?
        .allowlist_function("bl.*").allowlist_type("BL.*").layout_tests(false)
        .clang_args(["-x", "c++", "-std=c++17",
            &format!("-I{}", b2d_src.parent().unwrap().display())])
        //.blocklist_item("*(Virt|Impl)") // XXX: can not be blocked
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(odir.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "ovg")] fn binding_ovg(odir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut ovg_dir = PathBuf::from("3rdparty/amanithvg/include");

    // XXX: need to set environment variable before `cargo r/t`:
    // DYLD_FALLBACK_LIBRARY_PATH=$PWD/3rdparty/amanithvg/lib/macosx/ub/sre/standalone

    bindgen::builder().clang_arg(format!("-I{}", ovg_dir.display()))
        .header(ovg_dir.join("VG").join("vgext.h").to_string_lossy())
        .derive_copy(false).derive_debug(false).merge_extern_blocks(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
        .allowlist_function("vg.*").allowlist_type("VG.*").layout_tests(false)
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?.write_to_file(odir.join("openvg.rs"))?;

    ovg_dir.pop(); ovg_dir.push("lib"); ovg_dir.push(env::consts::OS); //ovg_dir.push("ub");
    ovg_dir.push(env::consts::ARCH); ovg_dir.push("sre"); ovg_dir.push("standalone");
    println!("cargo:rustc-link-search=native={}", ovg_dir.display());
    println!("cargo:rustc-link-lib=dylib=AmanithVG");

    Ok(())
}

#[cfg(feature = "ugl")] fn binding_ugl(odir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (ugl_inc, module) = (PathBuf::from("3rdparty/micro-gl/include"), "microgl");

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
        .generate()?.write_to_file(odir.join(module).with_extension("rs"))?;

    Ok(())
}
