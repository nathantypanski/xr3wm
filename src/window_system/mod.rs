use errors::*;

mod x;

pub use self::x::XWindowSystem;

pub trait WindowSystem {
    fn initialize() -> Result<Self> where Self: Sized;
    fn run(&self) -> Result<()>;
    fn stop(&self);
}
