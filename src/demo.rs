use crate::scene::Scene;
use crate::app::{App, AppInstance};
use web_sys::WebGl2RenderingContext;
use crate::scene1::Scene1;

pub struct Demo<'a> {
  scenes: Vec<Box<dyn Scene<'a>>>,
  current_scene: usize,
}

type Gl = WebGl2RenderingContext;

impl <'a> Demo<'a> {
  pub fn new() -> Self {
    //let render = |app: &App| {
      /*if app.current_frame == 0 {
        log!("Init scenes");
        let scene1 = Box::new(Scene1::new(&app));

        state.scenes.push(scene1);
        state.current_scene = 0;
      }
      else {
        state.scenes[state.current_scene].on_frame(&app);
        /*let gl = &app.context.clone();

        gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
        gl.clear_color(0.0, 0.0, 1.0, 1.0);
        gl.clear(Gl::COLOR_BUFFER_BIT);*/
      }*/
    //};

    Demo {
      scenes: Vec::new(),
      current_scene: usize::MAX,
    }
  }
}


impl <'a> AppInstance for Demo<'a>  {
  fn frame(&self, app: &App) {
    app.context.bind_framebuffer(Gl::FRAMEBUFFER, None);
    app.context.clear_color(0.0, 0.0, 1.0, 1.0);
    app.context.clear(Gl::COLOR_BUFFER_BIT);
  }
}
