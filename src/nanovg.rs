/****************************************************************
 * $ID: nanovg.rs  	    Sat 04 Nov 2023 15:13:31+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

use std::error::Error;
use femtovg::{renderer::OpenGl, Canvas, Path, Paint, Color};

/*
fn main() ->  Result<(), Box<dyn Error>> {
    let mut canvas = Canvas::new(get_renderer()?)?;

    let (width, height) = (640, 480);
    canvas.set_size(width, height, 1.0);
    canvas.clear_rect(0, 0, width * 4, height * 4, Color::rgbf(0.9, 0.0, 0.9));

    let mut path = Path::new();
    path.rect(0.0, 0.0, width as _, height as _);
    canvas.fill_path(&path, &Paint::linear_gradient(0.0, 0.0, width as _, 0.0,
        Color::rgba(255, 0, 0, 255), Color::rgba(0, 0, 255, 255)));
dbg!();
    canvas.flush();
dbg!();

    //let buf = canvas.screenshot()?.pixels().flat_map(|pixel|
    //    pixel.iter()).collect::<Vec<_>>();
    let buf = canvas.screenshot()?.into_contiguous_buf();
    let buf = unsafe { std::slice::from_raw_parts(buf.0.as_ptr() as *const u8,
        (width * height * 4) as _) };

    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create("target/foo.png")?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    //    png::ScaledFloat::from_scaled(45455)  // 1.0 / 2.2 scaled by 100000
    //let source_chromaticities = png::SourceChromaticities::new( // unscaled instant
    //    (0.3127, 0.3290), (0.6400, 0.3300), (0.3000, 0.6000), (0.1500, 0.0600));
    //encoder.set_source_chromaticities(source_chromaticities);
    encoder.write_header()?.write_image_data(buf)?;

    Ok(())
}

// https://github.com/Ionizing/femtovg-offscreen/blob/master/src/main.rs
// https://github.com/servo/surfman/blob/master/surfman/examples/offscreen.rs
// https://github.com/nobuyuki83/del-gl/blob/master/src/glutin/off_screen_render.rs
#[cfg(target_os = "macos")] fn get_renderer() -> Result<OpenGl, Box<dyn Error>> {
    //use glutin::{Context, ContextCurrentState, CreationError};
    use glutin::{event_loop::EventLoop, dpi::PhysicalSize};
    use glutin::{ContextBuilder, GlProfile, GlRequest};

    let ctx = ContextBuilder::new()
        .with_gl_profile(GlProfile::Core).with_gl(GlRequest::Latest)
        .build_headless(&EventLoop::new(), PhysicalSize::new(1, 1))?;
    let ctx = unsafe { ctx.make_current() }.unwrap();

    Ok(unsafe { OpenGl::new_from_function(|s| ctx.get_proc_address(s) as *const _) }?)
}

#[cfg(not(target_os = "macos"))] fn get_renderer() -> Result<OpenGl, Box<dyn Error>> {
    use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder};
    use glutin::context::{ContextApi, ContextAttributesBuilder};
    use glutin::api::egl::{device::Device, display::Display};
    use glutin::{display::GetGlDisplay, prelude::*};

    let devices = Device::query_devices()
        .expect("Failed to query devices").collect::<Vec<_>>();
    devices.iter().enumerate().for_each(|(index, device)|
        println!("Device {}: Name: {} Vendor: {}", index,
            device.name().unwrap_or("UNKNOWN"), device.vendor().unwrap_or("UNKNOWN")));

    let display = unsafe { Display::with_device(devices.first()
        .expect("No available devices"), None) }?;

    let config = unsafe { display.find_configs(
        ConfigTemplateBuilder::default().with_alpha_size(8)
            .with_surface_type(ConfigSurfaceTypes::empty()).build()) }.unwrap()
        .reduce(|config, accum| {
            if (config.supports_transparency().unwrap_or(false) &
                !accum.supports_transparency().unwrap_or(false)) ||
                config.num_samples() < accum.num_samples() { config } else { accum }
        })?;    println!("Picked a config with {} samples", config.num_samples());

    let _context = unsafe { display.create_context(&config,
        &ContextAttributesBuilder::new().build(None))
            .unwrap_or_else(|_| display.create_context(&config,
                &ContextAttributesBuilder::new()
                    .with_context_api(ContextApi::Gles(None)).build(None))
                .expect("failed to create context"))
    }.make_current_surfaceless()?;

    Ok(unsafe { OpenGl::new_from_function_cstr(|s| display.get_proc_address(s) as *const _) }?)
}
*/

use glutin::{surface::{Surface, WindowSurface}, context::PossiblyCurrentContext, prelude::*};
use winit::{event_loop::EventLoop, window::Window};

fn main() ->  Result<(), Box<dyn Error>> { // XXX: femtovg
    use winit::event::{Event, WindowEvent};
    let event_loop = EventLoop::new()?;
    let (window, surface, glctx,
        mut canvas) = create_window(&event_loop, 1000., 600.)?;

    /* #[cfg(target_arch = "wasm32")] let (window, mut canvas) = {
        use winit::platform::web::WindowBuilderExtWebSys;
        use wasm_bindgen::JsCast;

        //  XXX: HTML5/canvas API
        let canvas = web_sys::window().unwrap()
            .document().unwrap().get_element_by_id("canvas").unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let window = WindowBuilder::new()
            .with_canvas(Some(canvas)).build(&event_loop).unwrap();
        let canvas = Canvas::new(OpenGl::new_from_html_canvas(&canvas)
            .expect("Cannot create renderer")).expect("Cannot create canvas");

        (window, canvas)
    }; */

    render(&window, &surface, &glctx, &mut canvas);
    event_loop.run(|event, target| match event {
            Event::WindowEvent { window_id: _, event } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Destroyed => target.exit(),
                _ => () //println!("{:?}", event)
            },
            //Event::RedrawRequested(_) => render(&window, &surface, &glctx, &mut canvas),
            Event::LoopExiting => target.exit(),
            _ => () //println!("{:?}", event)
    })?;    //loop {}

    Ok(())
}

fn render<T: femtovg::Renderer>(_window: &Window, surface: &Surface<WindowSurface>,
    glctx: &PossiblyCurrentContext, canvas: &mut Canvas<T>) { // TODO:
    //let size = window.inner_size();
    let (width, height) = (canvas.width(), canvas.height());

    canvas.clear_rect(0, 0, width, height, Color::black());
    //canvas.clear_rect(30, 30, 30, 30, Color::rgbf(1., 0., 0.)); // Make small red rectangle

    let mut path = Path::new();
    path.rect(0.0, 0.0, width as _, height as _);
    canvas.fill_path(&path, &Paint::linear_gradient(0.0, 0.0, width as _, 0.0,
        Color::rgba(255, 0, 0, 255), Color::rgba(0, 0, 255, 255)));

    canvas.flush(); // Tell renderer to execute all drawing commands
    surface.swap_buffers(glctx).expect("Could not swap buffers"); // Display what just rendered
}

//  https://github.com/rust-windowing/glutin/blob/master/glutin_examples/examples/egl_device.rs

#[allow(clippy::type_complexity)]
fn create_window(event_loop: &EventLoop<()>, width: f32, height: f32)
    -> Result<(Window, Surface<WindowSurface>,
        PossiblyCurrentContext, Canvas<OpenGl>), Box<dyn Error>> {
    use glutin::{config::ConfigTemplateBuilder, surface::SurfaceAttributesBuilder,
        context::{ContextApi, ContextAttributesBuilder}, display::GetGlDisplay};
    use winit::{window::WindowBuilder, dpi::PhysicalSize};
    use raw_window_handle::HasRawWindowHandle;
    use glutin_winit::DisplayBuilder;
    use std::num::NonZeroU32;

    let (window, gl_config) = DisplayBuilder::new()
        .with_window_builder(Some(WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(true).with_title("Femtovg")))
        .build(event_loop, ConfigTemplateBuilder::new().with_alpha_size(8),
            |configs|
                // Find the config with the maximum number of samples,
                // so our triangle will be smooth.
                configs.reduce(|config, accum| {
                    if (config.supports_transparency().unwrap_or(false) &
                        !accum.supports_transparency().unwrap_or(false)) ||
                        config.num_samples() < accum.num_samples() { config } else { accum }
                }).unwrap())?;

    let window = window.unwrap();
    let size = window.inner_size();
    let raw_window_handle = window.raw_window_handle();
    let gl_display = gl_config.display();

    let surf_attr =
        SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(raw_window_handle, NonZeroU32::new(size. width).unwrap(),
                                      NonZeroU32::new(size.height).unwrap());
    let surface = unsafe {
        gl_display.create_window_surface(&gl_config, &surf_attr)? };

    let glctx = Some(unsafe {
        gl_display.create_context(&gl_config,
            &ContextAttributesBuilder::new()
                .build(Some(raw_window_handle)))
            .unwrap_or_else(|_| gl_display.create_context(&gl_config,
                &ContextAttributesBuilder::new()
                    .with_context_api(ContextApi::Gles(None))
                    .build(Some(raw_window_handle))).expect("Failed to create context"))
    }).take().unwrap().make_current(&surface)?;

    let mut canvas = Canvas::new(unsafe {
        OpenGl::new_from_function_cstr(|s|
            gl_display.get_proc_address(s) as *const _) }?)?;
    canvas.set_size(size.width, size.height, window.scale_factor() as _);

    Ok((window, surface, glctx, canvas))
}

