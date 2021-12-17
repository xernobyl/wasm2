use crate::app::App;

pub trait Scene {
    fn on_frame(&mut self, app: &App);
}
