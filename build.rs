
use std::{env, path::{Path, PathBuf}};

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
    println!("cargo:rerun-if-changed={}", PathBuf::from(".git/index").display());

    //std::process::Command::new(PathBuf::from("3rdparty/layout.sh")).status()?;
    #[allow(unused)] let path = PathBuf::from("target/bindings");
    std::fs::create_dir_all(&path)?;    // env::var("OUT_DIR")?

    #[cfg(feature = "b2d")] binding_b2d(&path)?;
    #[cfg(feature = "evg")] binding_evg(&path)?;
    #[cfg(feature = "ftg")] binding_ftg(&path)?;
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
        .generate()?.write_to_file(path.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "evg")] fn binding_evg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
        .generate()?.write_to_file(path.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "b2d")] fn binding_b2d(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut b2d_src = PathBuf::from("3rdparty/blend2d/blend2d");
    let mut jit_src = PathBuf::from("3rdparty/asmjit/asmjit");
    if !b2d_src.exists() { b2d_src.set_file_name("src"); }
    if !jit_src.exists() { jit_src.set_file_name("src"); }
    #[allow(unused_mut)] let mut bgen = bindgen::builder();
    let module = "blend2d"; // "blend2d_bindings";

    let mut cc = cc::Build::new();
    #[cfg(feature = "b2d_sfp")] { //println!("cargo:rustc-cfg=feature=\"b2d_sfp\"");
        #[cfg(target_arch = "aarch64")] cc.flag("-mno-outline-atomics");
        cc.define("BLEND2D_NO_DFP",  None).flag("-fsingle-precision-constant")
          .define("BL_BUILD_NO_TLS", None).flag("-Wno-uninitialized").compiler("g++-13");
        bgen = bgen.clang_arg("-DBLEND2D_NO_DFP");
    }   // refer to blend2d/CMakeLists.txt

    let compiler = cc.get_compiler();
    if  compiler.is_like_msvc() {
        cc  .flag("-MP").flag("-GR-").flag("-GF").flag("-W4")
            .flag("-Zc:__cplusplus").flag("-Zc:inline").flag("-GS-")//.flag("-GS")
            .flag("-Zc:strictStrings").flag("-Zc:threadSafeInit-").flag("-Oi")
            .flag_if_supported("-Zc:arm64-aliased-neon-types-");

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
            #[cfg(target_arch = "x86")]    cc.flag("-arch:SSE2");
            cc.define("BL_BUILD_OPT_AVX512", None);

            if compiler.is_like_clang() {   // XXX:
                cc  .flag("-clang:-fno-rtti").flag("-clang:-fno-math-errno")
                    .flag("-clang:-fno-trapping-math");

                let simd_flag = "-arch:AVX512";
                cc.is_flag_supported(simd_flag).unwrap_or(false).then(|| cc.flag(simd_flag)
                        .flag("-mpopcnt").flag("-mpclmul").flag("-mbmi").flag("-mbmi2")
                        .define("BL_TARGET_OPT_POPCNT", None).define("BL_TARGET_OPT_BMI2", None));

                is_x86_feature_detected!("avx2").then(|| cc.flag("-arch:AVX2")
                        .flag_if_supported("-mfma")
                        .flag("-mpopcnt").flag("-mpclmul").flag("-mbmi").flag("-mbmi2")
                        .define("BL_TARGET_OPT_POPCNT", None).define("BL_TARGET_OPT_BMI2", None));

                is_x86_feature_detected!("avx").then(||
                    cc.flag("-arch:AVX").flag("-mpopcnt").flag("-mpclmul"));
                is_x86_feature_detected!("sse4.2").then(||
                    cc.flag("-msse4.2") .flag("-mpopcnt").flag("-mpclmul"));
                is_x86_feature_detected!("sse4.1").then(|| cc.flag("-msse4.1"));
                is_x86_feature_detected!("ssse3") .then(|| cc.flag("-mssse3"));
                is_x86_feature_detected!("sse3")  .then(|| cc.flag("-msse3"));
            } else {
                is_x86_feature_detected!("sse3")  .then(|| cc.define("__SSE3__",   None));
                is_x86_feature_detected!("ssse3") .then(|| cc.define("__SSSE3__",  None));
                is_x86_feature_detected!("sse4.1").then(|| cc.define("__SSE4_1__", None));
                is_x86_feature_detected!("sse4.2").then(|| cc.define("__SSE4_2__", None));
                is_x86_feature_detected!("avx")   .then(|| cc.flag("-arch:AVX"));
                is_x86_feature_detected!("avx2")  .then(|| cc.flag("-arch:AVX2"));
                cc.flag_if_supported("-arch:AVX512");
            }
        }
    } else /*if compiler.is_like_gnu() || compiler.is_like_clang() */{
        cc//.flag("-Wall").flags("-Wextra").flag("-Wonversion") // ignore all warning flags
            .flag("-fno-exceptions").flag("-fno-rtti").flag("-fvisibility=hidden")
            .flag("-fno-math-errno").flag("-fno-threadsafe-statics")
            .flag("-fmerge-all-constants").flag_if_supported("-ftree-vectorize")
            .flag_if_supported("-mllvm")  .flag_if_supported("--disable-loop-idiom-all");

        if cfg!(not(target_vendor = "apple")) { cc.flag("-fno-semantic-interposition"); }
        if env::var("CARGO_CFG_TARGET_OS")?.as_str() == "ios" { //cfg!(target_os = "ios")
            cc  .flag("-fno-trapping-math").flag("-fno-finite-math-only")
                .flag_if_supported("-fno-enforce-eh-specs");
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
            // XXX: https://doc.rust-lang.org/std/arch/index.html
            // https://doc.rust-lang.org/reference/conditional-compilation.html
            cc.define("BL_BUILD_OPT_AVX512", None);     let mut avx512 = "";
            is_x86_feature_detected!("avx512f") .then(|| avx512 = "avx512f");
            is_x86_feature_detected!("avx512bw").then(|| avx512 = "avx512bw");
            is_x86_feature_detected!("avx512dq").then(|| avx512 = "avx512dq");
            is_x86_feature_detected!("avx512cd").then(|| avx512 = "avx512cd");
            is_x86_feature_detected!("avx512vl").then(|| avx512 = "avx512vl");
            if !avx512.is_empty() { cc.flag(format!("-m{avx512}"))
                    .flag("-mpopcnt").flag("-mpclmul").flag("-mbmi").flag("-mbmi2")
                    .define("BL_TARGET_OPT_POPCNT", None).define("BL_TARGET_OPT_BMI2", None);
            }

            //is_x86_feature_detected!("avxifma").then(|| cc.flag("-mfma")); // AKA avx2fma?
            is_x86_feature_detected!("avx2").then(|| cc.flag("-mavx2").flag_if_supported("-mfma")
                    .flag("-mpopcnt").flag("-mpclmul").flag("-mbmi").flag("-mbmi2")
                    .define("BL_TARGET_OPT_POPCNT", None).define("BL_TARGET_OPT_BMI2", None));

            is_x86_feature_detected!("avx").then(||
                cc.flag("-mavx")   .flag("-mpopcnt").flag("-mpclmul"));
            is_x86_feature_detected!("sse4.2").then(||
                cc.flag("-msse4.2").flag("-mpopcnt").flag("-mpclmul"));
            is_x86_feature_detected!("sse4.1").then(|| cc.flag("-msse4.1"));
            is_x86_feature_detected!("ssse3") .then(|| cc.flag("-mssse3"));
            is_x86_feature_detected!("sse3")  .then(|| cc.flag("-msse3"));

            #[cfg(target_arch = "x86")] is_x86_feature_detected!("sse2").then(|| {
                if  cc.get_compiler().is_like_gnu() {  cc.flag("-mfpmath=sse"); }
                    cc.flag("-msse2");
            });
        }

        if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            let simd_flag = "-mfpu=neon-vfpv4";
            cc.is_flag_supported(simd_flag)
                .is_ok_and(|bl| bl).then(|| cc.flag(simd_flag)
                .define("BL_TARGET_OPT_ASIMD", None).define("BL_BUILD_OPT_ASIMD", None));
        }

        if cfg!(target_arch = "aarch64") { let simd_flag = "-march=armv8-a+aes+crc";
            cc.is_flag_supported(simd_flag)
                .is_ok_and(|bl| bl).then(|| cc.flag(simd_flag)
                .define("BL_TARGET_OPT_ASIMD_CRYPTO", None)
                .define("BL_BUILD_OPT_ASIMD_CRYPTO", None));
        }
    }

    cc.cpp(true).flag("-std=c++17").define("ASMJIT_EMBED", None)
        .define("ASMJIT_NO_STDCXX", None).define("ASMJIT_NO_FOREIGN", None)
        //.define("ASMJIT_ABI_NAMESPACE=abi_bl", None)
        .files(glob::glob(&format!("{}/**/*.cpp",
            jit_src.display()))?.filter_map(Result::ok))
        .files(glob::glob(&format!("{}/**/*.cpp",
            b2d_src.display()))?.filter_map(|f|
                f.ok().filter(|f| !f.to_string_lossy().contains("_test"))))
        .include(b2d_src.parent().unwrap()).include(jit_src.parent().unwrap())
        .include(&b2d_src).include(jit_src).opt_level(3).define("NDEBUG", None).compile(module);
    //println!("cargo:rustc-link-lib=rt");  // https://blend2d.com/doc/build-instructions.html

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
        .generate()?.write_to_file(path.join(module).with_extension("rs"))?;

    Ok(())
}

#[cfg(feature = "ovg")] fn binding_ovg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut ovg_dir = PathBuf::from("3rdparty/amanithvg/include");

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
        .generate()?.write_to_file(path.join(module).with_extension("rs"))?;

    Ok(())
}

