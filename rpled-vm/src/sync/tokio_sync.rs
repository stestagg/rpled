use core::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

pub struct AsyncSignal {
    flag: AtomicBool,
    flag_changed: Notify,
}

impl Default for AsyncSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncSignal {
    pub const fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
            flag_changed: Notify::const_new(),
        }
    }
}

impl super::Signal for AsyncSignal {
    fn signal(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.flag_changed.notify_waiters();
    }

    fn reset(&self) {
        self.flag.store(false, Ordering::SeqCst);
        self.flag_changed.notify_waiters();
    }

    async fn wait_signal(&self) {
        loop {
            let notified = self.flag_changed.notified();
            if self.flag.load(Ordering::SeqCst) {
                return;
            }
            notified.await;
        }
    }

    async fn wait_reset(&self) {
        loop {
            let notified = self.flag_changed.notified();

            if !self.flag.load(Ordering::SeqCst) {
                return;
            }
            notified.await;
        }
    }

    fn is_signaled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

pub struct TokioSync;

impl super::Sync for TokioSync {
    type Signal = AsyncSignal;

    fn create_signal() -> Self::Signal {
        AsyncSignal::new()
    }

    fn delay(us: u16) -> impl Future<Output = ()> {
        tokio::time::sleep(core::time::Duration::from_micros(us as u64))
    }
}
