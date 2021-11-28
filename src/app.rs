use crate::fullscreen_buffers::{self, ScreenBuffers};
use crate::half_cube::{self, HalfCube};
use crate::shaders::{setup_shaders, Programs};
use serde::Serialize;
use std::panic;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

type Gl = WebGl2RenderingContext;

pub struct App {
    pub context: Rc<Gl>,
    pub programs: [Option<web_sys::WebGlProgram>; Programs::NPrograms as usize],
    pub current_frame: u32,
    pub delta_time: f64,
    pub current_timestamp: f64,
    pub width: u32,
    pub height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
    pub fullscreen_buffers: ScreenBuffers,
    pub cube: HalfCube,
    new_width: u32, // set this whenever there are resizes
    new_height: u32,
}

impl App {
    pub fn init(setup: &'static dyn FnMut(&App), frame: &'static dyn FnMut(&App)) {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct WebGlOptions {
            alpha: bool,
            desynchronized: bool,
            antialias: bool,
            depth: bool,
            fail_if_major_performance_caveat: bool,
            power_preference: &'static str,
            premultiplied_alpha: bool,
            preserve_drawing_buffer: bool,
            stencil: bool,
        }

        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        #[allow(unused_must_use)]
        {
            canvas.style().set_property("position", "fixed");
            canvas.style().set_property("left", "0");
            canvas.style().set_property("top", "0");
            canvas.style().set_property("width", "100%");
            canvas.style().set_property("height", "100%");
            document.body().unwrap().append_child(&canvas);
        }

        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;
        let aspect_ratio: f32;

        if width != 0 && height != 0 {
            canvas.set_width(width);
            canvas.set_height(height);

            aspect_ratio = width as f32 / height as f32;
        } else {
            aspect_ratio = 1.0;
        }

        log!("Created canvas ({}, {})...", width, height);

        let context_options = JsValue::from_serde(&WebGlOptions {
            alpha: false,
            desynchronized: true,
            antialias: false,
            depth: false,
            fail_if_major_performance_caveat: false, // true
            power_preference: "high-performance",
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
            stencil: false,
        })
        .unwrap();

        let context = canvas
            .get_context_with_context_options("webgl2", &context_options)
            .unwrap()
            .unwrap()
            .dyn_into::<Gl>()
            .unwrap();

        log!("Created context...");

        #[allow(unused_must_use)]
        {
            //context.get_extension("EXT_float_blend"); // blend on 32 bit components, shouldn't be needed but keep here just in case
            //context.get_extension("EXT_texture_filter_anisotropic"); // find how to use this with wasm :S
            context.get_extension("EXT_color_buffer_float"); // enable a bunch of types
            context.get_extension("OES_texture_float_linear"); // enable linear filtering on floating textures
        }

        #[cfg(debug_assertions)]
        {
            log!("Enabling debug extensions.");
            #[allow(unused_must_use)]
            {
                context.get_extension("WEBGL_debug_shaders");
            }
        }

        // unused stuff
        // context.get_extension("OVR_multiview2"); // for VR stuff, keep here for future reference
        // context.get_extension("EXT_texture_compression_bptc");
        // context.get_extension("EXT_texture_compression_rgtc");
        // context.get_extension("WEBGL_compressed_texture_s3tc");
        // context.get_extension("WEBGL_compressed_texture_s3tc_srgb");

        let fullscreen_buffers =
            fullscreen_buffers::ScreenBuffers::init(&context, &(width as i32), &(height as i32))
                .unwrap();
        let screen = web_sys::window().unwrap().screen().unwrap();

        let rc_context = Rc::new(context);

        let mut app = App {
            context: rc_context.clone(),
            cube: half_cube::HalfCube::new(rc_context.clone()),
            programs: Default::default(),
            current_frame: 0,
            current_timestamp: 0.0,
            delta_time: 0.0,
            aspect_ratio,
            width: 0,
            height: 0,
            new_width: width,
            new_height: height,
            max_width: screen.width().ok().unwrap() as u32,
            max_height: screen.height().ok().unwrap() as u32,
            fullscreen_buffers,
        };

        log!("setup_shaders()");
        setup_shaders(rc_context, &mut app.programs).expect("Shader error");

        let app_rc0 = Rc::new(RefCell::new(app));

        let app_rc = app_rc0.clone();
        let closure = Closure::wrap(Box::new(move || {
            let width = canvas.client_width() as u32;
            let height = canvas.client_height() as u32;

            if width != 0 && height != 0 && canvas.width() != width && canvas.height() != height {
                canvas.set_width(width);
                canvas.set_height(height);
                let mut app = app_rc.borrow_mut();
                app.new_width = width;
                app.new_height = height;
            }
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_onresize(Option::Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let app_rc = app_rc0.clone();
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        setup(&app_rc0.borrow());

        let closure = Closure::wrap(Box::new(move |timestamp| {
            #[allow(unused_must_use)]
            {
                web_sys::window().unwrap().request_animation_frame(
                    (f.borrow().as_ref().unwrap() as &Closure<dyn FnMut(f64)>)
                        .as_ref()
                        .unchecked_ref(),
                );
            }

            let mut app = app_rc.borrow_mut();
            app.delta_time = timestamp - app.current_timestamp;
            app.current_timestamp = timestamp;

            let _resized = if app.new_width > 0 {
                if app.max_height < app.new_height {
                    app.max_height = app.new_height;
                }

                if app.max_width < app.new_width {
                    app.max_width = app.new_width;
                }

                app.width = app.new_width;
                app.height = app.new_height;
                app.aspect_ratio = app.width as f32 / app.height as f32;
                app.new_width = 0;

                log!("Resize: {} {}, {}", app.width, app.height, app.aspect_ratio);
                log!("Max size: {} {}", app.max_width, app.max_height);

                true
            } else {
                false
            };

            frame(&app);

            app.current_frame += 1;
        }) as Box<dyn FnMut(f64)>);

        *g.borrow_mut() = Some(closure);

        log!("Starting render loop...");
        #[allow(unused_must_use)]
        {
            web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }
}
