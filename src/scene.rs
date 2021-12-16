use web_sys::WebGl2RenderingContext;
use crate::app::App;
type Gl = WebGl2RenderingContext;

pub trait Scene<'a> {
    fn on_frame(&mut self, app: &'a App);
}
