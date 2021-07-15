use web_sys::WebGl2RenderingContext;

use crate::app::App;

type Gl = WebGl2RenderingContext;

pub trait Scene {
    fn on_frame(&self, gl: &Gl, app: &App);
}
