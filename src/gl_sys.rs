// gl_sys.rs


pub trait GLSys {
	fn new() -> Result<Self, String> where Self: Sized;
	fn start_loop(self);
}
