// gl_sys.rs


pub trait GLSys<'a> {
	fn new() -> Result<Self, &'a str> where Self: Sized;
	fn start_loop(&self);
}
