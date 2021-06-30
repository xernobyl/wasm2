use crate::app::App;

pub trait Scene {
  fn new() -> Self where Self: Sized;
  fn init(&mut self);
  fn on_frame(&self, app: &App);
}
