use crate::app::App;

pub trait Scene {
  fn on_frame(&self, app: &App);
}
