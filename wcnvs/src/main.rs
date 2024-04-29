
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
                let data = feng.read_file(&feng.files()[0]).await.unwrap();

                use intvg::{convert::Convert, tinyvg::TVGBuf};
                let tvg = if evt.value().ends_with(".svg") { TVGBuf::from_usvg(&data).unwrap()
                } else { TVGBuf::load_data(&mut std::io::Cursor::new(&data)).unwrap() };
                tracing::info!("picked file: {}, {:?} with {} bytes\nTinyVG: {:?}",
                    evt.value(), feng.files(), data.len(), tvg.header);

                let canvas = web_sys::window().unwrap().document().unwrap()
                    .get_element_by_id("canvas").unwrap()
                    .dyn_into::<HtmlCanvasElement>().unwrap();
                let ctx2d = canvas.get_context("2d").unwrap().unwrap()
                    .dyn_into::<CanvasRenderingContext2d>().unwrap();
                ctx2d.reset();

                //let bounding = canvas.get_bounding_client_rect();
                tracing::info!("canvas size: {} x {}; client: {} x {}", canvas.width(),
                    canvas.height(), canvas.client_width(), canvas.client_height());
                ctx2d.clear_rect(0.0, 0.0, canvas.width() as _, canvas.height() as _);

                let scale = (canvas.width()  as f32  / tvg.header.width  as f32)
                        .min(canvas.height() as f32  / tvg.header.height as f32);
                let _ = ctx2d.translate(
                    ((canvas.width()  as f32 - scale * tvg.header.width  as f32) / 2.) as _,
                    ((canvas.height() as f32 - scale * tvg.header.height as f32) / 2.) as _);

                wcnvs::tinyvg::render(&tvg, scale, &ctx2d).unwrap();
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

