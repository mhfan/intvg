
pub mod tinyvg;
pub mod render;
pub mod convert;

#[cfg(feature = "b2d")] pub mod blend2d;
#[cfg(feature = "evg")] pub mod gpac_evg;
#[cfg(feature = "evg")] pub mod render_evg;
#[cfg(feature = "b2d")] pub mod render_b2d;

/*  TODO: https://github.com/sammycage/plutovg
    https://github.com/kiba/SDL_svg/blob/master/ftgrays.c
    https://github.com/Samsung/rlottie/tree/master/src/vector/freetype
    https://gitlab.freedesktop.org/freetype/freetype/-/blob/master/src/smooth/ftgrays.c
    https://gitlab.freedesktop.org/freetype/freetype/-/blob/master/src/raster/ftraster.c */

#[cfg(feature = "ftg")] pub mod ftgrays  {  pub use ftg_ffi::*; // XXX:
    #[allow(non_snake_case)] #[allow(non_camel_case_types)] //#[allow(non_upper_case_globals)]
    mod ftg_ffi { include!(concat!(env!("OUT_DIR"), "/ftgrays.rs")); } // ftg_bindings.rs

    impl From<(FT_Pos, FT_Pos)> for FT_Vector {
        fn from(v: (FT_Pos, FT_Pos)) -> Self { Self { x: v.0, y: v.1 } }
    }
}

