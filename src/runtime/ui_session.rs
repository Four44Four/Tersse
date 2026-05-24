use std::sync::{Arc, Mutex};

use super::RuntimeUi;

type UiWork = Box<dyn FnOnce(&mut RuntimeUi) + Send>;
pub(crate) type UiQueue = Arc<Mutex<Vec<UiWork>>>;

pub(crate) fn new_ui_queue() -> UiQueue {
    Arc::new(Mutex::new(Vec::new()))
}

/// Handle for queueing work to run synchronously on the UI thread.
#[derive(Clone)]
pub struct UiSession {
    queue: UiQueue,
}

impl UiSession {
    pub(crate) fn new(queue: UiQueue) -> Self {
        Self { queue }
    }

    /// Queues `work` to run on the UI thread during the next event-loop frame.
    ///
    /// The UI is redrawn immediately after `work` finishes. This method is safe
    /// to call from any thread or async runtime.
    pub fn queue_update(&self, work: impl FnOnce(&mut RuntimeUi) + Send + 'static) {
        self.queue.lock().unwrap().push(Box::new(work));
    }
}

pub(crate) fn ui_queue_has_pending(queue: &UiQueue) -> bool {
    !queue.lock().unwrap().is_empty()
}

impl RuntimeUi {
    pub(crate) fn drain_ui_queue(&mut self) {
        let works: Vec<UiWork> = {
            let mut pending = self.ui_queue.lock().unwrap();
            std::mem::take(&mut *pending)
        };
        for work in works {
            work(self);
            let _ = self.tick_resize_debounce();
            if !self.is_resize_debounce_active() {
                self.draw();
            }
        }
    }

    pub(crate) fn ui_queue_has_pending(&self) -> bool {
        ui_queue_has_pending(&self.ui_queue)
    }
}
