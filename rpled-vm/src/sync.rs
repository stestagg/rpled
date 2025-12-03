pub trait Sync {
    type Signal: Signal;

    fn create_signal() -> Self::Signal;
    fn delay(us: u16) -> impl Future<Output = ()>;
}

pub trait Signal {
    fn signal(&self);
    fn reset(&self);

    fn wait_signal(&self) -> impl Future<Output = ()>;
    fn wait_reset(&self) -> impl Future<Output = ()>;

    fn is_signaled(&self) -> bool;
}

#[cfg(feature = "tokio")]
pub mod tokio_sync;

#[cfg(feature = "tokio")]
pub use self::tokio_sync::TokioSync;
