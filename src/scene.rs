use web_sys::WebGl2RenderingContext;

use crate::app::App;

type Gl = WebGl2RenderingContext;

pub trait Scene {
    // fn init(&mut self);
    fn on_frame(&self, app: &App);
}
