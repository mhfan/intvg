
//#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    tracing::info!("starting app of web canvas");
    launch(app);
}

//  https://github.com/nathan-barry/rust-site/blob/main/src/projects/game_of_life.rs
use wasm_bindgen::{JsCast/*, prelude::*, closure::Closure*/};
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, Path2d};

fn app() -> Element {
    let canvas_demo = |_| {
        let window = web_sys::window().unwrap();
        let canvas = window.document().unwrap().get_element_by_id("canvas").unwrap()
            .dyn_into::<HtmlCanvasElement>().unwrap();
        let ctx2d = canvas.get_context("2d").unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>().unwrap();

        let scale = window.device_pixel_ratio();
        canvas.set_height((canvas.client_height() as f64 * scale) as _);
        canvas.set_width ((canvas.client_width()  as f64 * scale) as _);
        let _ = ctx2d.scale(scale, scale);  // XXX: matter for retina rendering

        ctx2d.set_fill_style  (&"rgba(0, 255, 0, 255)".into());
        ctx2d.set_stroke_style(&"rgba(0, 0, 255, 255)".into());

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
    };

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

    rsx! { style { dangerous_inner_html: format_args!("{}", // XXX:
            //"html { background-color: #15191D; color: #DCDCDC; }
            // body { font-family: Courier, Monospace; text-align: center; height: 100vh; }"
            " body { display: flex; justify-content: center; align-items: center; height: 96vh; }
             #main { display: flex;  flex-direction: column;
                     justify-content: space-evenly; width: 60%; height: 100%; }
            #canvas { width: 100%; height: 90%; }"
        )}

        canvas { id: "canvas", onmounted: canvas_demo }
        input { r#type: "file", accept: ".tvg, .svg", id: "picker",
            onchange: move |evt| async move { if let Some(feng) = &evt.files() {
                let canvas = web_sys::window().unwrap().document().unwrap()
                    .get_element_by_id("canvas").unwrap()
                    .dyn_into::<HtmlCanvasElement>().unwrap();
                let ctx2d = canvas.get_context("2d").unwrap().unwrap()
                    .dyn_into::<CanvasRenderingContext2d>().unwrap();

                let data = feng.read_file(&feng.files()[0]).await.unwrap();
                use {intvg::tinyvg::TVGBuf, instant::Instant};

                let tvg = if evt.value().ends_with(".svg") {
                    let mut fontdb = usvg::fontdb::Database::new(); fontdb.load_system_fonts();
                    let tree = usvg::Tree::from_data(&data,
                        &usvg::Options::default(), &fontdb).unwrap();

                    let now = Instant::now();
                    wcnvs::render::render_svg(&tree, &ctx2d, canvas.width(), canvas.height());
                    draw_perf(&ctx2d, 1. / (Instant::now() - now).as_secs_f32());

                    return //intvg::convert::Convert::from_usvg(&data).unwrap()
                } else { TVGBuf::load_data(&mut std::io::Cursor::new(&data)).unwrap() };
                tracing::info!("picked file: {}, {:?} with {} bytes\nTinyVG: {:?}",
                    evt.value(), feng.files(), data.len(), tvg.header);

                let now = Instant::now();
                wcnvs::render::render_tvg(&tvg, &ctx2d, canvas.width(), canvas.height());
                draw_perf(&ctx2d, 1. / (Instant::now() - now).as_secs_f32());
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

