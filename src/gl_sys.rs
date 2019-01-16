// gl_sys.rs


pub trait GLSys {
	fn new() -> Result<Self, &'static str> where Self: Sized;
	fn start_loop(&self);
}
