use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::terminal_input::TerminalPoll;

use super::RuntimeUi;

type UiWork = Box<dyn FnOnce(&mut RuntimeUi) + Send>;
pub(crate) type UiQueue = Arc<Mutex<Vec<UiWork>>>;
pub(crate) type UiSignalSender = mpsc::Sender<UiSignal>;
pub(crate) type UiSignalReceiver = mpsc::Receiver<UiSignal>;

pub(crate) enum UiSignal {
    QueueUpdated,
    Terminal(TerminalPoll),
    TerminalError,
}

pub(crate) fn new_ui_queue() -> UiQueue {
    Arc::new(Mutex::new(Vec::new()))
}

pub(crate) fn new_ui_signal_channel() -> (UiSignalSender, UiSignalReceiver) {
    mpsc::channel()
}

/// Handle for queueing work to run synchronously on the UI thread.
#[derive(Clone)]
pub struct UiSession {
    queue: UiQueue,
    signal_tx: UiSignalSender,
}

impl UiSession {
    pub(crate) fn new(queue: UiQueue, signal_tx: UiSignalSender) -> Self {
        Self { queue, signal_tx }
    }

    /// Queues `work` to run on the UI thread during the next event-loop frame.
    ///
    /// The UI is redrawn after `work` finishes, respecting the queue update redraw
    /// debounce interval. This method is safe to call from any thread or async runtime.
    pub fn queue_update(&self, work: impl FnOnce(&mut RuntimeUi) + Send + 'static) {
        self.queue.lock().unwrap().push(Box::new(work));
        let _ = self.signal_tx.send(UiSignal::QueueUpdated);
    }

    /// Displays `message` in the message gutter overlay.
    ///
    /// If a message is already visible, the gutter updates in place and shows the
    /// multi-message indicator. Safe to call from any thread or async runtime.
    pub fn send_message(&self, message: impl Into<String> + Send + 'static) {
        let message = message.into();
        self.queue_update(move |ui| {
            ui.apply_gutter_message(message);
        });
    }
}

pub(crate) fn ui_queue_has_pending(queue: &UiQueue) -> bool {
    !queue.lock().unwrap().is_empty()
}

impl RuntimeUi {
    pub(crate) fn drain_ui_queue(&mut self) -> bool {
        let works: Vec<UiWork> = {
            let mut pending = self.ui_queue.lock().unwrap();
            std::mem::take(&mut *pending)
        };
        if works.is_empty() {
            return false;
        }
        self.draining_ui_queue = true;
        for work in works {
            work(self);
            let _ = self.tick_resize_debounce();
            if self.ui_queue_redraw_pending {
                self.request_draw();
            }
        }
        self.draining_ui_queue = false;
        true
    }

    pub(crate) fn ui_queue_has_pending(&self) -> bool {
        ui_queue_has_pending(&self.ui_queue)
    }
}
