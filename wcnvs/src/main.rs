
//#![allow(non_snake_case)]
use dioxus::{prelude::*, web::WebEventExt};

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    tracing::info!("starting app of web canvas");
    launch(app);
}

//  https://github.com/nathan-barry/rust-site/blob/main/src/projects/game_of_life.rs
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, Path2d};
use wasm_bindgen::{JsCast/*, prelude::*, closure::Closure*/};

fn app() -> Element {
    fn draw_canvas(ctx2d: &CanvasRenderingContext2d, data: &[u8], file: &str, cw: u32, ch: u32) {
        use {intvg::tinyvg::TVGBuf, instant::Instant};

        let tvg = if file.ends_with(".svg") {
            let mut usvg_opts = usvg::Options::default();
            usvg_opts.fontdb_mut().load_system_fonts();
            let tree = usvg::Tree::from_data(data, &usvg_opts).unwrap();

            let now = Instant::now();
            wcnvs::render::render_svg(&tree, ctx2d, cw, ch);
            draw_perf(ctx2d, 1. / now.elapsed().as_secs_f32());

            return //intvg::convert::Convert::from_usvg(&data).unwrap()
        } else { TVGBuf::load_data(&mut std::io::Cursor::new(data)).unwrap() };
        //tracing::info!("picked file: {} with {} bytes\nTinyVG: {:?}",
        //    file, data.len(), tvg.header);  // evt.value()

        let now = Instant::now();
        wcnvs::render::render_tvg(&tvg, ctx2d, cw, ch);
        draw_perf(ctx2d, 1. / now.elapsed().as_secs_f32());
    }

    fn draw_perf(ctx2d: &CanvasRenderingContext2d, fps: f32) {
        ctx2d.save();   let _ = ctx2d.reset_transform();
        let _ = ctx2d.translate(3., 3.);
        ctx2d.set_fill_style(&"#00000080".into());
        ctx2d.fill_rect(0., 0., 100., 20.);

        ctx2d.set_text_align("right");
        ctx2d.set_text_baseline("top");
        ctx2d.set_font("14px sans-serif");
        ctx2d.set_fill_style(&"#f0f0f0f0".into());
        let _ = ctx2d.fill_text(&format!("{:.2} FPS", fps), 100. - 10., 2.);
        ctx2d.restore();
    }

    fn draw_smile(ctx2d: &CanvasRenderingContext2d) {
        ctx2d.set_fill_style  (&"#00ff00ff".into());
        ctx2d.set_stroke_style(&"#0000ffff".into());
        use std::f64::consts::{PI, TAU};

        if true {   ctx2d.begin_path();
            ctx2d.arc(75.0, 75.0, 50.0, 0.0, TAU).unwrap();    // Draw the outer circle.

            ctx2d.move_to(110.0, 75.0); // Draw the mouth.
            ctx2d.arc(75.0, 75.0, 35.0, 0.0,  PI).unwrap();

            ctx2d.move_to( 65.0, 65.0); // Draw the  left eye.
            ctx2d.arc(60.0, 65.0,  5.0, 0.0, TAU).unwrap();

            ctx2d.move_to( 95.0, 65.0); // Draw the right eye.
            ctx2d.arc(90.0, 65.0,  5.0, 0.0, TAU).unwrap();

            ctx2d.stroke();
        } else {    let path = Path2d::new().unwrap();
            path.arc(75.0, 75.0, 50.0, 0.0, TAU).unwrap();    // Draw the outer circle.

            path.move_to(110.0, 75.0); // Draw the mouth.
            path.arc(75.0, 75.0, 35.0, 0.0,  PI).unwrap();

            path.move_to( 65.0, 65.0); // Draw the  left eye.
            path.arc(60.0, 65.0,  5.0, 0.0, TAU).unwrap();

            path.move_to( 95.0, 65.0); // Draw the right eye.
            path.arc(90.0, 65.0,  5.0, 0.0, TAU).unwrap();

            ctx2d.stroke_with_path(&path);
        }
    }

    let mut file_data: Signal<Option<(Vec<u8>, String)>> = use_signal(|| None);
    let init_canvas = move |evt: Event<MountedData>| {
        let canvas: HtmlCanvasElement = evt.web_event().clone().dyn_into().unwrap();
        //let canvas: HtmlCanvasElement = web_sys::window().unwrap().document().unwrap()
        //    .get_element_by_id("canvas").unwrap().dyn_into().unwrap();

        let closure = wasm_bindgen::closure::
            Closure::<dyn FnMut(_)>::new(move |entries: js_sys::Array| {
            for entry in entries.iter() {
                let entry: web_sys::ResizeObserverEntry = entry.dyn_into().unwrap();
                if  entry.target().get_attribute("id")
                    .is_some_and(|id| id != "canvas") { return }
                //let bbox = entry.content_rect();

                let canvas: HtmlCanvasElement = entry.target().dyn_into().unwrap();
                let ctx2d: CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().dyn_into().unwrap();

                // XXX: this is matter for retina rendering
                let scale = web_sys::window().unwrap().device_pixel_ratio();
                canvas.set_width ((canvas.client_width()  as f64 * scale) as _);
                canvas.set_height((canvas.client_height() as f64 * scale) as _);

                let fdata = file_data.peek_unchecked();
                if  fdata.is_none() {   let _ = ctx2d.scale(scale, scale);
                    draw_smile(&ctx2d); return
                }

                let fdata = fdata.as_ref().unwrap();
                draw_canvas(&ctx2d, &fdata.0, &fdata.1, canvas.width(), canvas.height());
            }
        }); // Closure::wrap(Box::new() as Box<dyn FnMut(_)>);

        //window.set_onresize(Some(closure.as_ref().unchecked_ref()));  closure.forget();
        web_sys::ResizeObserver::new(closure.into_js_value()
            .unchecked_ref()).unwrap().observe(&canvas);
    };

    rsx! { style { dangerous_inner_html: format_args!("{}", // XXX:
            //"html { background-color: #15191D; color: #DCDCDC; }
            // body { font-family: Courier, Monospace; text-align: center; height: 100vh; }"
            " body { display: flex; justify-content: center; align-items: center; height: 96vh; }
             #main { display: flex;  flex-direction: column;
                     justify-content: space-evenly; width: 60%; height: 100%; }
            #canvas { width: 100%; height: 90%; }"
        ) }

        canvas { id: "canvas", onmounted: init_canvas }
        input { r#type: "file", accept: ".tvg, .svg", id: "picker",
            onchange: move |evt| async move { if let Some(feng) = &evt.files() {
                let canvas: HtmlCanvasElement = web_sys::window().unwrap().document().unwrap()
                    .get_element_by_id("canvas").unwrap().dyn_into().unwrap();
                let ctx2d = canvas.get_context("2d").unwrap().unwrap().dyn_into().unwrap();

                let file = feng.files()[0].clone(); //evt.value();
                let data = feng.read_file(&file).await.unwrap();
                draw_canvas(&ctx2d, &data, &file, canvas.width(), canvas.height());
                file_data.set(Some((data, file)));
            } }
        }
    }
}

/* fn app() -> Element { rsx!(Router::<Route> {}) }

#[derive(Clone, Routable, Debug, PartialEq)] enum Route {
    #[route("/blog/:id")] Blog { id: i32 },
    #[route("/")] Home {},
}

#[component] fn Blog(id: i32) -> Element {
    rsx!(Link { to: Route::Home {}, "Go to counter" } "Blog post {id}")
}

#[component] fn Home() -> Element {
    let mut count = use_signal(cx, || 0);

    rsx! { Link { to: Route::Blog { id:  count()  }, "Go to blog" }
        div { h1 { "High-Five counter: {count}" }
            button { onclick: move |_|  count += 1, "Up high!" }
            button { onclick: move |_|  count -= 1, "Down low!" }
        }
    }
} */

