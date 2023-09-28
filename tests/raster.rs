
#[cfg(feature = "ftg")] #[test] fn ftgrays_demo() -> Result<(), Box<dyn std::error::Error>> {
    use core::{ptr::null_mut, ffi::c_void};     use intvg::ftgrays::*;

    let mut points = vec![ // FTDemo_Icon @freetype2/demo/src/ftcommon.c
        ( 4, 8).into(), ( 4,10).into(), ( 8,12).into(), ( 8,52).into(), ( 4,54).into(),
        ( 4,56).into(), (60,56).into(), (60,44).into(), (58,44).into(), (56,52).into(),
        (44,52).into(), (44,12).into(), (48,10).into(), (48, 8).into(), (32, 8).into(),
        (32,10).into(), (36,12).into(), (36,52).into(), (16,52).into(), (16,36).into(),
        (24,36).into(), (26,40).into(), (28,40).into(), (28,28).into(), (26,28).into(),
        (24,32).into(), (16,32).into(), (16,12).into(), (20,10).into(), (20, 8).into(), ];
    let mut tags = vec![FT_CURVE_TAG_ON as i8; points.len()];
    let mut contours = vec![29];

    let mut bitmap = FT_Bitmap {
        rows: 0, width: 0, pitch: 0, buffer: null_mut(), // FIXME:

        pixel_mode: FT_Pixel_Mode::FT_PIXEL_MODE_LCD as u8, // FT_PIXEL_MODE_BGRA
        // https://docs.rs/bindgen/latest/bindgen/struct.Builder.html#enums
        // Not used currently, or only used with FT_PIXEL_MODE_GRAY
        palette_mode: 0, palette: null_mut(), num_grays: 0,
    };

    #[allow(unused)]
    unsafe extern "C" fn span_func(y: i32, count: i32, // TODO: solid/linear/radial/stroke
        spans: *const FT_Span, user: *mut c_void) { // The gray span drawing callback.
        let bitmap = user as *mut FT_Bitmap;    assert!(0 < count);

        for span in core::slice::from_raw_parts(spans, count as usize) {
            for x in span.x..span.len as i16 {
                // TODO: handle pixel at (x, y) with span.coverage (alpha)
            }
        }
    }

    let outline = FT_Outline {
        n_points: points.len() as i16, points: points.as_mut_ptr(), tags: tags.as_mut_ptr(),
        n_contours: contours.len() as i16, contours: contours.as_mut_ptr(),
        flags: FT_OUTLINE_NONE, // XXX: FT_OUTLINE_IGNORE_DROPOUTS
    };  assert!(tags.len() == points.len());

    let mut params = FT_Raster_Params {
        source: &outline  as *const _ as _,
        user: &mut bitmap as *mut   _ as _, gray_spans: Some(span_func),

        // ftgrays.c: `ft_grays_raster` are typoed as `ft_gray_raster` in some comments.
        // ftimage.h: FT_Pos can be typed as `signed int` rather than `long`? // XXX:
        clip_box: FT_BBox { xMin: 0, yMin: 0, xMax: 0, yMax: 0 }, // FT_RASTER_FLAG_CLIP
        flags: FT_RASTER_FLAG_AA | FT_RASTER_FLAG_DIRECT,
        black_spans: None, bit_test: None, bit_set: None, // Unused
        target: null_mut(), // &bitmap, unused in direct mode
    };  let mut raster = null_mut();

    unsafe {    // use of extern static (ft_grays_raster/ft_standard_raster) is unsafe
        let _ = ft_grays_raster.raster_new.unwrap()(null_mut(), &mut raster);
        //if  res != 0 { panic!("raster_new failed: {}", res); }
        //ft_grays_raster.raster_reset.unwrap()(raster, null_mut(), 0);
        //ft_grays_raster.raster_set_mode.unwrap()(raster, 0, null_mut());

        let res = ft_grays_raster.raster_render.unwrap()(raster, &mut params);
        if  res != 0 { panic!("raster_render failed: {res}"); }
        //ft_grays_raster.raster_done.unwrap()(raster);
    }

    Ok(())
}

