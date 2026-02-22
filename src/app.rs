use crate::camera::Camera;
use crate::gpu::{GbufferSet, GpuContext};
use crate::half_cube::HalfCube;
use crate::stereo_camera::StereoCamera;
use crate::view::ViewState;
use std::f32::consts::PI;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wgpu::RenderPass;

const JITTER_SIZE: usize = 8;

// -----------------------------------------------------------------------------
// Debug: enable render features one by one (set to true to enable).
//
// Suggested order to isolate issues:
//  1. All false except leave ENABLE_GBUFFER_PATH = false → clear only (simple path).
//  2. ENABLE_WAREHOUSE = true → warehouse raymarch.
//  3. ENABLE_SCENE = true → cubes / scene.
//  4. ENABLE_GBUFFER_PATH = true → use gbuffer pass (warehouse_gbuffer + scene), then present.
//  5. ENABLE_TAA = true → TAA resolve (writes to resolve texture).
//  6. ENABLE_POST = true → brightness + blur + screen composite.
// -----------------------------------------------------------------------------
const ENABLE_WAREHOUSE: bool = true;
const ENABLE_SCENE: bool = true;
/// false = draw directly to swap (warehouse + scene), no TAA/post. Set true for full pipeline.
const ENABLE_GBUFFER_PATH: bool = true;
const ENABLE_TAA: bool = true;
const ENABLE_POST: bool = true;
// -----------------------------------------------------------------------------

/// Framebuffer scale for viewport dimensions. 1.0 = full res. Used when WebGPU is in place.
#[allow(dead_code)]
const FRAME_BUFFER_SCALE: f32 = 1.0;

pub trait AppInstance {
    fn setup(&mut self, app: &App);
    /// Called once per view (once for mono, twice for stereo left/right).
    /// When `pass` is `Some`, WebGPU is active: warehouse is already drawn; the implementor may record more draws (e.g. cubes) into the same pass.
    /// When `is_gbuffer` is true, the pass is the G-buffer pass (2 color + depth); use [HalfCube::draw_instanced_gbuffer].
    fn frame(&mut self, app: &mut App, view: &ViewState, pass: Option<&mut RenderPass<'_>>, is_gbuffer: bool);
}

pub struct App {
    /// Canvas element; used for 2d fallback clear and WebGPU surface.
    pub canvas: Rc<web_sys::HtmlCanvasElement>,
    /// WebGPU context; set after async init (request_adapter/request_device).
    pub gpu: Option<GpuContext>,
    pub current_frame: u32,
    pub delta_time: f64,
    pub current_timestamp: f64,
    pub width: u32,
    pub height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
    pub cube: HalfCube,
    pub camera: Camera,
    pub stereo_camera: StereoCamera,
    pub use_stereo: bool,
    pub jitter_pattern: Vec<f32>,
    new_width: u32,
    new_height: u32,
}

impl App {
    pub fn init(mut app_instance: Box<dyn AppInstance>) {
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

        log!("Canvas created; WebGPU init starting.");

        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;
        let aspect_ratio: f32 = if width != 0 && height != 0 {
            canvas.set_width(width);
            canvas.set_height(height);
            width as f32 / height as f32
        } else {
            1.0
        };

        let screen = web_sys::window().unwrap().screen().unwrap();
        let canvas_rc = Rc::new(canvas);
        let canvas_rc_for_resize = canvas_rc.clone();
        let fovy = 2.0 * ((PI / 4.0).tan() / (1.0 + aspect_ratio * aspect_ratio).sqrt()).atan();
        let gpu_rc = Rc::new(RefCell::new(None::<GpuContext>));
        let gpu_rc_for_loop = gpu_rc.clone();
        let gbuffer_rc = Rc::new(RefCell::new(None::<GbufferSet>));
        let gbuffer_rc_for_loop = gbuffer_rc.clone();
        let mut app = App {
            canvas: canvas_rc.clone(),
            gpu: None,
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
            cube: HalfCube::new(),
            camera: Camera::new(PI / 2.0, aspect_ratio, 0.1, f32::INFINITY),
            stereo_camera: StereoCamera::new(fovy, aspect_ratio, 0.1, f32::INFINITY),
            use_stereo: false,
            jitter_pattern: crate::utils::halton_sequence_2d(JITTER_SIZE, 2, 3),
        };
        app.stereo_camera.set_eye_distance(0.08);
        app.stereo_camera.set_convergence_distance(2.0);

        app_instance.as_mut().setup(&app);

        let app_rc0 = Rc::new(RefCell::new(app));
        let pending_resize = Rc::new(RefCell::new(None::<(u32, u32)>));
        let pending_resize_for_resize = pending_resize.clone();
        let pending_stereo_toggle = Rc::new(RefCell::new(false));
        let closure = Closure::wrap(Box::new(move || {
            let width = canvas_rc_for_resize.client_width() as u32;
            let height = canvas_rc_for_resize.client_height() as u32;
            if width != 0 && height != 0 && canvas_rc_for_resize.width() != width && canvas_rc_for_resize.height() != height {
                canvas_rc_for_resize.set_width(width);
                canvas_rc_for_resize.set_height(height);
                *pending_resize_for_resize.borrow_mut() = Some((width, height));
            }
        }) as Box<dyn FnMut()>);
        web_sys::window()
            .unwrap()
            .set_onresize(Option::Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let closure = Closure::wrap(Box::new(move || {
            log!("KEY DOWN!");
        }) as Box<dyn FnMut()>);
        #[allow(unused_must_use)]
        {
            document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        }
        closure.forget();

        let closure = Closure::wrap(Box::new(move || {
            log!("KEY UP!");
        }) as Box<dyn FnMut()>);
        #[allow(unused_must_use)]
        {
            document.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref());
        }
        closure.forget();

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let app_instance_rc = Rc::new(RefCell::new(app_instance));
        let app_rc_for_loop = app_rc0.clone();
        let pending_resize_for_loop = pending_resize.clone();
        let pending_stereo_toggle_for_loop = pending_stereo_toggle.clone();
        let pending_init = Rc::new(RefCell::new(None::<(HalfCube, GpuContext)>));
        let pending_init_for_loop = pending_init.clone();

        // Async WebGPU init. Never borrows app_rc; stores (cube, gpu) in pending_init for the rAF loop to apply.
        {
            let pending_init_async = pending_init.clone();
            let canvas_for_gpu = canvas_rc.as_ref().clone();
            spawn_local(async move {
                if let Some(gpu) = crate::gpu::init_gpu(canvas_for_gpu).await {
                    let cube = crate::half_cube::HalfCube::init_from_gpu(
                        &gpu.device,
                        &gpu.queue,
                        gpu.surface_format,
                    );
                    *pending_init_async.borrow_mut() = Some((cube, gpu));
                    log!("WebGPU initialized.");
                }
            });
        }

        let closure = Closure::wrap(Box::new(move |timestamp| {
            #[allow(unused_must_use)]
            {
                web_sys::window().unwrap().request_animation_frame(
                    (f.borrow().as_ref().unwrap() as &Closure<dyn FnMut(f64)>)
                        .as_ref()
                        .unchecked_ref(),
                );
            }

            let Ok(mut app) = app_rc_for_loop.try_borrow_mut() else {
                return;
            };
            if let Some((cube, gpu)) = pending_init_for_loop.borrow_mut().take() {
                app.cube = cube;
                *gpu_rc_for_loop.borrow_mut() = Some(gpu);
            }
            if let Some((w, h)) = pending_resize_for_loop.borrow_mut().take() {
                app.new_width = w;
                app.new_height = h;
            }
            if pending_stereo_toggle_for_loop.replace(false) {
                app.use_stereo = !app.use_stereo;
                log!("Stereo: {}", app.use_stereo);
            }
            app.delta_time = timestamp - app.current_timestamp;
            app.current_timestamp = timestamp;

            let resized = if app.new_width > 0 && app.new_height > 0 {
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
                app.new_height = 0;
                log!("Resize: {}x{}", app.width, app.height);
                true
            } else {
                false
            };
            let _ = resized;

            let aspect_ratio = app.aspect_ratio;
            let fb_w = app.width as f32;
            let fb_h = app.height as f32;
            let frame = app.current_frame as usize;
            let ts = app.current_timestamp;
            let jitter_x = app.jitter_pattern[(frame % JITTER_SIZE) * 2] / fb_w;
            let jitter_y = app.jitter_pattern[(frame % JITTER_SIZE) * 2 + 1] / fb_h;
            app.camera.set_aspect(aspect_ratio);
            app.camera.set_jitter(jitter_x, jitter_y);
            let cam_pos = glam::Vec3::new(-10.0, 1.7, -10.0);
            let look_at = glam::Vec3::new(0.0, 1.7, 0.0);
            app.camera.look_at(cam_pos, look_at, glam::Vec3::NEG_Y);
            app.camera.update();

            if app.use_stereo {
                let ar = app.aspect_ratio;
                app.stereo_camera.look_at(cam_pos, look_at, glam::Vec3::NEG_Y);
                app.stereo_camera.set_aspect(ar);
                app.stereo_camera.update();
            }

            let fb_width = (app.width as f32 / FRAME_BUFFER_SCALE).ceil().max(1.0) as i32;
            let fb_height = (app.height as f32 / FRAME_BUFFER_SCALE).ceil().max(1.0) as i32;
            let views: Vec<ViewState> = if app.use_stereo {
                app.stereo_camera
                    .to_view_states((fb_width, fb_height))
            } else {
                let vp = (0, 0, fb_width, fb_height);
                vec![app.camera.to_view_state(vp)]
            };

            let time_s = (app.current_timestamp / 1000.0) as f32;
            let mut app_instance = app_instance_rc.borrow_mut();
            let width = app.width;
            let height = app.height;

            if let Some(gpu) = gpu_rc_for_loop.borrow_mut().as_mut() {
                gpu.configure_surface(width, height);
                let Ok(frame_tex) = gpu.surface.get_current_texture() else {
                    app.current_frame += 1;
                    return;
                };
                let swap_view = frame_tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                if app.current_frame == 0 {
                    log!(
                        "[Frame 0] Drawing with GPU (views: {}, warehouse: {}, scene: {}, gbuffer: {}, taa: {}, post: {}).",
                        views.len(),
                        ENABLE_WAREHOUSE,
                        ENABLE_SCENE,
                        ENABLE_GBUFFER_PATH,
                        ENABLE_TAA,
                        ENABLE_POST
                    );
                }
                let use_gbuffer_path = views.len() == 1 && ENABLE_GBUFFER_PATH;
                if use_gbuffer_path {
                    let history_index = gpu.taa_history_index();
                    gpu.ensure_gbuffer(width, height, &gbuffer_rc_for_loop);
                    let gbuffer_guard = gbuffer_rc_for_loop.borrow();
                    let gbuffer = gbuffer_guard.as_ref().unwrap();
                    let color_view = gbuffer.color_view();
                    let velocity_view = gbuffer.velocity_view();
                    let depth_view = gbuffer.depth_view();
                    {
                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("gbuffer"),
                            color_attachments: &[
                                Some(wgpu::RenderPassColorAttachment {
                                    view: &color_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                }),
                                Some(wgpu::RenderPassColorAttachment {
                                    view: &velocity_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                }),
                            ],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &depth_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(0.0),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            multiview_mask: None,
                            occlusion_query_set: None,
                        });
                        let view_state = &views[0];
                        let (vx, vy, vw, vh) = view_state.viewport;
                        pass.set_viewport(vx as f32, vy as f32, vw as f32, vh as f32, 0.0, 1.0);
                        if ENABLE_WAREHOUSE {
                            gpu.draw_warehouse_gbuffer(
                                &mut pass,
                                view_state,
                                time_s,
                                width,
                                height,
                            );
                        }
                        if ENABLE_SCENE {
                            app_instance.frame(&mut *app, view_state, Some(&mut pass), true);
                        }
                    }
                    if ENABLE_TAA {
                        gpu.run_taa_pass(&mut encoder, gbuffer, history_index);
                    }
                    let resolve_view = gbuffer.resolve_view();
                    let bloom_view = gbuffer.bloom_view();
                    let blur_view = gbuffer.blur_view();
                    let bw = gbuffer.bloom_width();
                    let bh = gbuffer.bloom_height();
                    if ENABLE_POST && ENABLE_TAA {
                        gpu.run_brightness_pass(&mut encoder, &resolve_view, &bloom_view, bw, bh);
                        gpu.run_blur_pass(&mut encoder, &bloom_view, &blur_view, bw, bh);
                        gpu.run_screen_pass(&mut encoder, &resolve_view, &blur_view, &swap_view);
                    } else {
                        let source = if ENABLE_TAA { &resolve_view } else { &color_view };
                        gpu.run_present_pass(&mut encoder, source, &swap_view);
                    }
                } else {
                    let depth_view = gpu.main_depth_view();
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &swap_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.02,
                                    g: 0.02,
                                    b: 0.08,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: depth_view.map(|v| wgpu::RenderPassDepthStencilAttachment {
                            view: v,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        multiview_mask: None,
                        occlusion_query_set: None,
                    });
                    for view_state in &views {
                        let (vx, vy, vw, vh) = view_state.viewport;
                        pass.set_viewport(
                            vx as f32,
                            vy as f32,
                            vw as f32,
                            vh as f32,
                            0.0,
                            1.0,
                        );
                        // Always run fullscreen pass: raymarch when ENABLE_WAREHOUSE, else solid gray so cubes show on top.
                        gpu.draw_warehouse(
                            &mut pass,
                            view_state,
                            time_s,
                            width,
                            height,
                            !ENABLE_WAREHOUSE,
                        );
                        if ENABLE_SCENE {
                            app_instance.frame(&mut *app, view_state, Some(&mut pass), false);
                        }
                    }
                }
                gpu.queue.submit(Some(encoder.finish()));
                frame_tex.present();
            } else {
                for view in &views {
                    app_instance.frame(&mut *app, view, None, false);
                }
            }

            app.current_frame += 1;
        }) as Box<dyn FnMut(f64)>);

        *g.borrow_mut() = Some(closure);

        let vr_supported = js_sys::eval("navigator.xr !== undefined")
            .unwrap()
            .as_bool()
            .unwrap();
        log!("WebXR support: {}", vr_supported);

        if vr_supported {
            Self::add_vr_button(pending_stereo_toggle.clone());
        } else {
            log!("WebXR not available (navigator.xr is undefined).");
        }

        log!("Starting render loop...");
        #[allow(unused_must_use)]
        {
            web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }

    fn add_vr_button(pending_stereo_toggle: Rc<RefCell<bool>>) {
        let document = web_sys::window().unwrap().document().unwrap();
        let button = document
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlDivElement>()
            .unwrap();

        #[allow(unused_must_use)]
        {
            button.set_inner_text("🤓");
            button.style().set_property("position", "fixed");
            button.style().set_property("right", "0");
            button.style().set_property("bottom", "0");
            button.style().set_property("font-size", "10em");
            button.style().set_property("cursor", "pointer");
            button.style().set_property("user-select", "none");
            button.style().set_property("transform", "rotate(45deg)");
            button
                .style()
                .set_property("text-shadow", "#f00 -0.05em 0.05em 0.1em, #0ff 0.05em -0.05em 0.1em");
            document.body().unwrap().append_child(&button);
        }

        let closure = Closure::wrap(Box::new(move || {
            *pending_stereo_toggle.borrow_mut() = true;
        }) as Box<dyn FnMut()>);
        #[allow(unused_must_use)]
        {
            button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        }
        closure.forget();
    }
}
