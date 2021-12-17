use crate::scene::Scene;
use crate::app::{App, AppInstance};
use web_sys::WebGl2RenderingContext;
use crate::scene1::Scene1;

pub struct Demo {
	scenes: Vec<Box<dyn Scene>>,
	current_scene: usize,
}

type Gl = WebGl2RenderingContext;

impl Demo {
	pub fn new() -> Self {
		Demo {
			scenes: Vec::new(),
			current_scene: usize::MAX,
		}
	}
}


impl AppInstance for Demo {
	fn setup(&mut self, app: &App) {
		log!("Initializing scenes...");
		let scene1 = Box::new(Scene1::new(&app));
		self.scenes.push(scene1);
	}


	fn frame(&mut self, app: &App) {
		if self.current_scene >= self.scenes.len() {
			app.context.bind_framebuffer(Gl::FRAMEBUFFER, None);
			app.context.clear_color(0.0, 0.0, 1.0, 1.0);
			app.context.clear(Gl::COLOR_BUFFER_BIT);
		}
		else {
            // self.scenes[self.current_scene].on_frame(app);
		}
	}
}
