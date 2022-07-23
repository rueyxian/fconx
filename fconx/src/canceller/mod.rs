///
#[derive(Debug)]
pub struct Canceller {
    is_cancel: std::sync::Arc<parking_lot::Mutex<bool>>,
    cancel_send: parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

///
impl Canceller {
    ///
    pub(crate) fn new() -> std::sync::Arc<Canceller> {
        let (cancel_send, cancel_recv) = tokio::sync::oneshot::channel::<()>();
        let cancel_send = parking_lot::Mutex::new(Some(cancel_send));
        let is_cancel = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let is_cancel_clone = std::sync::Arc::clone(&is_cancel);

        tokio::spawn(async move {
            let _ = cancel_recv.await;
            *is_cancel_clone.lock() = true;
        });

        let canceller = Canceller {
            is_cancel,
            cancel_send,
        };

        std::sync::Arc::new(canceller)
    }

    ///
    pub fn arc_clone(self: &std::sync::Arc<Canceller>) -> std::sync::Arc<Canceller> {
        std::sync::Arc::clone(&self)
    }

    ///
    pub(crate) fn is_cancel(self: &std::sync::Arc<Canceller>) -> bool {
        *self.is_cancel.lock()
    }

    ///
    pub fn cancel(self: std::sync::Arc<Canceller>) {
        self.cancel_send.lock().take().unwrap().send(()).unwrap();
    }
}
