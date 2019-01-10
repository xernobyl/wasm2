// gl_sys.rs


pub trait GLSys<'a> {
	fn new() -> Result<Self, &'a str>;
	fn start_loop(&self);
}
