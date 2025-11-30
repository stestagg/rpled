
pub trait Sync {
    type Signal: Signal;
    type Delay: Delay;
}

pub trait Signal {
    fn signal(&self);
    fn reset(&self);

    fn wait_signal(&self) -> impl Future<Output = ()>;
    fn wait_reset(&self) -> impl Future<Output = ()>;

    fn is_signaled(&self) -> bool;
}

pub trait Delay {
    fn delay_us(&self, us: u32) -> impl Future<Output = ()>;
}

#[cfg(feature = "embassy")]
impl Signal for embassy_sync::signal::Signal<()> {
    fn signal(&self) {
        self.signal(());
    }

    fn reset(&self) {
        self.reset();
    }

    fn wait(&self) -> impl Future<Output = ()> + '_ {
        self.wait()
    }
}

#[cfg(feature = "embassy")]
impl Delay for embassy_time::Delay {
    fn delay_us(&self, us: u32) -> impl Future<Output = ()> {
        embassy_time::Delay::new(core::time::Duration::from_micros(us as u64))
    }
}

#[cfg(target_has_atomic)]
impl Signal for core::sync::atomic::AtomicBool {
    fn signal(&self) {
        self.store(true, core::sync::atomic::Ordering::SeqCst);
    }

    fn reset(&self) {
        self.store(false, core::sync::atomic::Ordering::SeqCst);
    }

    fn wait_signal(&self) -> impl Future<Output = ()> {
        async move {
            while !self.load(core::sync::atomic::Ordering::SeqCst) {
                core::hint::spin_loop();
            }
        }
    }
    fn wait_reset(&self) -> impl Future<Output = ()> {
        async move {
            while self.load(core::sync::atomic::Ordering::SeqCst) {
                core::hint::spin_loop();
            }
        }
    }

    fn is_signaled(&self) -> bool {
        self.load(core::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(feature = "tokio")]
struct TokioSleep{}

#[cfg(feature = "tokio")]
impl Delay for tokio::time::Interval {
    fn delay_us(&self, us: u32) -> impl Future<Output = ()> {
        tokio::time::sleep(core::time::Duration::from_micros(us as u64))
    }
}