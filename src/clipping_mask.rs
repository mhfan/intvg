//! Scoped path clipping and alpha masks for the Blend2D backend.
//!
//! Blend2D only keeps rectangular clips in `BLContext`. Arbitrary clips are
//! therefore rendered into bounded offscreen layers and combined with
//! Porter-Duff `DST_IN`.

use super::*;

impl BLContext {
    /// Renders `content` through an alpha mask produced by `mask`.
    ///
    /// `bounds` is the affected rectangle in target-device pixels. Both
    /// callbacks retain this context's current transform, but start with fresh
    /// paint state and transparent pixels. The composed result is then drawn
    /// through this context's current clip, alpha, and composition operator.
    ///
    /// # Limitations
    ///
    /// This API **does not support luminance masks**; it supports alpha masks
    /// only. The mask is rendered into `A8`, so RGB color and luminance are
    /// discarded. For example, opaque black produces a fully opaque mask here,
    /// whereas SVG's default luminance-mask semantics make it transparent.
    ///
    /// A non-positive extent is a successful no-op. Keeping `bounds` tight
    /// limits temporary storage to one `PRGB32` and one `A8` image of `w * h`
    /// pixels.
    pub fn render_mask_in<M, C>(&mut self, bounds: BLRectI, mask: M, content: C) ->
        Result<(), BLErr> where
            M: FnOnce(&mut BLContext) -> Result<(), BLErr>,
            C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        let target = self.get_target_size();
        if bounds.w <= 0 || bounds.h <= 0 { return Ok(()); }
        let (max_x, max_y) = (target.width() as i32, target.height() as i32);
        let (x0, y0) = (bounds.x.clamp(0, max_x), bounds.y.clamp(0, max_y));
        let (x1, y1) = (bounds.x.saturating_add(bounds.w) .clamp(0, max_x),
                        bounds.y.saturating_add(bounds.h) .clamp(0, max_y));
        if x0 >= x1 || y0 >= y1 { return Ok(()); }
        let bounds = BLRectI { x: x0, y: y0, w: (x1 - x0), h: (y1 - y0) };

        let mut trfm = self.final_transform();
        trfm.post_translate((-bounds.x as f64, -bounds.y as f64).into());
        let content = render_layer(&bounds, &trfm,
            BLFormat::BL_FORMAT_PRGB32, content)?;
        let mask = render_layer(&bounds, &trfm, BLFormat::BL_FORMAT_A8, mask)?;

        let mut masked = BLContext::from_image(content)?;
        masked.reset_transform(None);
        masked.set_comp_op(BLCompOp::BL_COMP_OP_DST_IN);
        masked.blit_image_d(BLPoint::new(), &mask, &local_area(&bounds))?;
        self.blit_layer(&masked.end()?, &bounds)
    }

    /// Renders `content` through the filled area of `path`, allocating only its
    /// device-space intersection with the target image.
    ///
    /// This is the path equivalent of a scoped clip: Blend2D has no persistent
    /// path-clip state, so callers provide the rendering operation as a
    /// closure. The context's current fill rule is used for the clip path.
    pub fn clip_to_path<C>(&mut self, path: &BLPath, content: C) ->
        Result<(), BLErr> where C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        let Some(bounds) = self.path_device_bounds(path)? else { return Ok(()); };
        self.clip_to_path_in(bounds, path, content)
    }

    /// Equivalent to [`BLContext::clip_to_path`], but uses caller-provided
    /// target-device bounds to avoid measuring and transforming `path`.
    pub fn clip_to_path_in<C>(&mut self, bounds: BLRectI, path: &BLPath, content: C) ->
        Result<(), BLErr> where C: FnOnce(&mut BLContext) -> Result<(), BLErr>, {
        // SAFETY: `self.0` is a live context and the getter only reads its state.
        let fill_rule = unsafe { bl_context_get_fill_rule(&self.0) };
        self.render_mask_in(bounds, |mask| {
            mask.set_fill_rule(fill_rule);
            mask.fill_geometry_rgba32(path, BLRgba32::new(255, 255, 255, 255))
        }, content)
    }

    fn path_device_bounds(&self, path: &BLPath) -> Result<Option<BLRectI>, BLErr> {
        if path.get_size() == 0 { return Ok(None); }

        let mut device_path = BLPath::new();
        device_path.add_transformed_path(path, &self.final_transform())?;
        let bbox = device_path.get_bounding_box()?;
        let target = self.get_target_size();
        if ![bbox.x0, bbox.y0, bbox.x1, bbox.y1].iter().all(|v| v.is_finite()) {
            return Ok(None);
        }
        let (max_x, max_y) = (target.width() as f64, target.height() as f64);
        let (x0, y0) = (bbox.x0.floor().clamp(0.0, max_x),
                        bbox.y0.floor().clamp(0.0, max_y));
        let (x1, y1) = (bbox.x1.ceil() .clamp(0.0, max_x),
                        bbox.y1.ceil() .clamp(0.0, max_y));
        Ok((x0 < x1 && y0 < y1).then_some(BLRectI {
            x: x0 as _, y: y0 as _, w: (x1 - x0) as _, h: (y1 - y0) as _
        }))
    }

    fn blit_layer(&mut self, image: &BLImage, bounds: &BLRectI) -> Result<(), BLErr> {
        let transform = self.user_transform();
        self.reset_transform(None);
        let area = local_area(bounds);
        let result = self.blit_image_d((bounds.x as f64,
                                        bounds.y as f64).into(), image, &area);
        self.reset_transform(Some(&transform));     result
    }
}

fn render_layer(bounds: &BLRectI, transform: &BLMatrix2D, format: BLFormat,
    render: impl FnOnce(&mut BLContext) -> Result<(), BLErr>) -> Result<BLImage, BLErr> {
    let mut layer = BLContext::new(bounds.w as _, bounds.h as _, format)?;
    layer.clear_all()?;
    layer.reset_transform(Some(transform));
    render(&mut layer)?;
    layer.end()
}

fn local_area(bounds: &BLRectI) -> BLRectI {
    BLRectI { x: 0, y: 0, w: bounds.w, h: bounds.h }
}

#[cfg(test)] mod tests { use super::*;

    fn alpha(image: &BLImage, x: usize, y: usize) -> u8 {
        image.pixels().unwrap()[y * image.stride() as usize + x * 4 + 3]
    }

    #[test] fn clips_content_to_path() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.add_rect(&(2.0, 1.0, 3.0, 4.0).into());

        let mut ctx = BLContext::new(8, 8, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_all()?;
        ctx.clip_to_path(&path, |layer|
            layer.fill_all_rgba32(BLRgba32::new(255, 0, 0, 255)))?;
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 3, 2), 255);
        assert_eq!(alpha(&image, 0, 0), 0);
        assert_eq!(alpha(&image, 6, 6), 0);
        Ok(())
    }

    #[test] fn mask_and_content_share_the_parent_transform() -> Result<(), BLErr> {
        let mask_rect: BLRect = (0.0, 0.0, 2.0, 4.0).into();
        let mut ctx = BLContext::new(8, 4, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_all()?;
        ctx.translate((2.0, 0.0).into());
        let transform = ctx.user_transform().get_values();
        ctx.render_mask_in((0, 0, 8, 4).into(),
            |mask| mask.fill_geometry_rgba32(
                &mask_rect, BLRgba32::new(255, 255, 255, 255)),
            |content| content.fill_all_rgba32(BLRgba32::new(0, 255, 0, 255)))?;
        assert_eq!(ctx.user_transform().get_values(), transform);
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 2, 1), 255);
        assert_eq!(alpha(&image, 0, 1), 0);
        assert_eq!(alpha(&image, 4, 1), 0);
        Ok(())
    }

    #[test] fn automatic_bounds_follow_the_final_transform() -> Result<(), BLErr> {
        let mut path = BLPath::new();
        path.add_rect(&(0.0, 0.0, 2.0, 3.0).into());

        let mut ctx = BLContext::new(10, 10, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clear_all()?;
        ctx.translate((7.0, 2.0).into());
        ctx.rotate(core::f64::consts::FRAC_PI_2, None);
        ctx.clip_to_path(&path, |layer|
            layer.fill_all_rgba32(BLRgba32::new(0, 0, 255, 255)))?;
        let image = ctx.end()?;

        assert_eq!(alpha(&image, 5, 3), 255);
        assert_eq!(alpha(&image, 7, 3), 0);
        assert_eq!(alpha(&image, 3, 3), 0);
        Ok(())
    }

    #[test] fn empty_and_offscreen_paths_are_noops() -> Result<(), BLErr> {
        let mut called = false;
        let mut ctx = BLContext::new(4, 4, BLFormat::BL_FORMAT_PRGB32)?;
        ctx.clip_to_path(&BLPath::new(), |_| { called = true; Ok(()) })?;

        let mut path = BLPath::new();
        path.add_rect(&(10.0, 10.0, 2.0, 2.0).into());
        ctx.clip_to_path(&path, |_| { called = true; Ok(()) })?;

        assert!(!called);
        Ok(())
    }
}
