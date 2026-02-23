use crate::app::{App, AppInstance};
use crate::scene::{FrameInput, Scene, SceneDescriptor};
use crate::scene1::Scene1;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use wgpu::RenderPass;

pub struct Demo {
    scenes: Vec<Box<dyn Scene>>,
    current_scene: usize,
}

impl Demo {
    pub fn new() -> Self {
        Demo {
            scenes: Vec::new(),
            current_scene: 0,
        }
    }
}

impl AppInstance for Demo {
    fn setup(&mut self, app: &App) {
        log!("Initializing scenes...");
        let scene1 = Box::new(Scene1::new(app));
        self.scenes.push(scene1);
    }

    fn descriptor(&self) -> &SceneDescriptor {
        self.scenes[self.current_scene].descriptor()
    }

    fn update(&mut self, input: &FrameInput) {
        if self.current_scene < self.scenes.len() {
            self.scenes[self.current_scene].update(input);
        }
    }

    fn frame(
        &mut self,
        app: &mut App,
        view: &crate::view::ViewState,
        pass: Option<&mut RenderPass<'_>>,
        is_gbuffer: bool,
    ) {
        if self.current_scene >= self.scenes.len() {
            if pass.is_none() {
                if let Some(ctx) = app
                    .canvas
                    .get_context("2d")
                    .ok()
                    .flatten()
                    .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
                {
                    let _ = ctx.set_fill_style_str("#0000ff");
                    let _ = ctx.fill_rect(0.0, 0.0, app.width as f64, app.height as f64);
                }
            }
        } else {
            self.scenes[self.current_scene].on_frame(app, view, pass, is_gbuffer);
        }
    }
}
