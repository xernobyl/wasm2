use crate::scene::Scene;
use crate::app::App;
use web_sys::WebGl2RenderingContext;
use crate::scene1::Scene1;

pub struct Demo {
  scenes: Vec<Box<dyn Scene>>,
  current_scene: usize,
}

type Gl = WebGl2RenderingContext;

impl Demo {
  pub fn run() {
    let mut state = Demo {
      scenes: Vec::new(),
      current_scene: usize::MAX,
    };

    let setup = |app: &App| {
      log!("Init scenes");
      let scene1 = Box::new(Scene1::new(&app));

      state.scenes.push(scene1);
      state.current_scene = 0;
    };

    let render = |app: &App| {
      state.scenes[state.current_scene].on_frame(&app);
      /*let gl = &app.context.clone();

      gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
      gl.clear_color(0.0, 0.0, 1.0, 1.0);
      gl.clear(Gl::COLOR_BUFFER_BIT);*/
    };

    App::init(&setup, &render);
  }
}
