use ahash::AHashMap;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use pancurses::{curs_set, endwin, initscr, noecho};
use tokio::runtime::Handle;
use tokio::sync::oneshot;

use crate::terminal_input;
use crate::terminal_input::{TerminalKey, TerminalPoll};
use crate::pure::message_gutter::MessageGutterState;
use crate::ScreenTitle;

use super::element_store::ElementStore;
use super::types::UiEvent;
use super::ui_session::{self, UiSession};
use super::RuntimeUi;

/// Owns a current-thread Tokio runtime on a dedicated background thread.
///
/// The UI main thread blocks on `mpsc` and never drives the runtime directly, so
/// keyboard listening and user-spawned async work run on this thread via `block_on`.
pub(super) struct AsyncRuntimeDriver {
    handle: Handle,
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl AsyncRuntimeDriver {
    pub fn start(signal_tx: ui_session::UiSignalSender) -> Self {
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let thread = thread::Builder::new()
            .name("tersse-async".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create async runtime");
                ready_tx
                    .send(runtime.handle().clone())
                    .expect("async runtime ready signal");
                runtime.block_on(run_async_driver(signal_tx, shutdown_rx));
            })
            .expect("Failed to spawn async driver thread");

        let handle = ready_rx
            .recv()
            .expect("async runtime ready signal missing");
        Self {
            handle,
            shutdown_tx: Some(shutdown_tx),
            thread: Some(thread),
        }
    }

    pub fn handle(&self) -> Handle {
        self.handle.clone()
    }
}

impl Drop for AsyncRuntimeDriver {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

async fn run_async_driver(
    signal_tx: ui_session::UiSignalSender,
    mut shutdown: oneshot::Receiver<()>,
) {
    let mut stream = terminal_input::terminal_event_stream();
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            event = terminal_input::read_terminal_event(&mut stream) => {
                match event {
                    Ok(Some(event)) => {
                        if signal_tx.send(ui_session::UiSignal::Terminal(event)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        let _ = signal_tx.send(ui_session::UiSignal::TerminalError);
                        break;
                    }
                }
            }
        }
    }
}

impl RuntimeUi {
    pub fn new() -> Self {
        let _ = terminal_input::enter_raw_mode();
        let win = initscr();
        noecho();
        let _ = curs_set(0);
        pancurses::start_color();
        pancurses::use_default_colors();

        let ui_queue = ui_session::new_ui_queue();
        let (ui_signal_tx, ui_signal_rx) = ui_session::new_ui_signal_channel();
        let async_driver = AsyncRuntimeDriver::start(ui_signal_tx.clone());

        let mut ui = Self {
            win,
            title: None,
            elements: ElementStore::new(),
            focused_position: 0,
            pair_cache: AHashMap::new(),
            next_pair_id: 1,
            next_element_id: 0,
            cached_heights: AHashMap::new(),
            text_input_layout_cache: AHashMap::new(),
            resize_debounce_until: None,
            redraw_debounce_until: None,
            last_terminal_yx: None,
            screen_scroll: 0,
            ui_queue,
            ui_signal_tx,
            ui_signal_rx,
            async_driver: Some(async_driver),
            has_rendered_first_frame: false,
            ui_queue_redraw_pending: false,
            ui_queue_redraw_plan: crate::pure::ui_redraw::ElementRedrawPlan::default(),
            draining_ui_queue: false,
            sync_layout_redraw_pending: false,
            message_gutter: MessageGutterState::default(),
            message_gutter_expires_at: None,
        };
        let _ = ui.reload_screen_after_resize();
        ui
    }

    pub fn set_title(&mut self, title: ScreenTitle) {
        self.title = Some(title);
    }

    pub fn clear_title(&mut self) {
        self.title = None;
    }

    /// Returns a cloneable handle for queueing UI updates from other threads.
    pub fn ui_session(&self) -> UiSession {
        UiSession::new(Arc::clone(&self.ui_queue), self.ui_signal_tx.clone())
    }

    /// Returns a cloneable handle to the shared Tokio runtime.
    ///
    /// Use this to spawn async background tasks without creating a separate runtime.
    pub fn runtime(&self) -> super::UiRuntime {
        let handle = self
            .async_driver
            .as_ref()
            .expect("async runtime missing")
            .handle();
        super::UiRuntime::new(handle)
    }

    /// Runs the UI event loop until the user quits.
    pub fn run(&mut self) {
        while self.run_frame() {}
    }

    /// Advances one frame. Returns `false` when the user quits.
    pub fn step(&mut self) -> bool {
        self.run_frame()
    }

    fn run_frame(&mut self) -> bool {
        if !self.has_rendered_first_frame {
            let _ = self.tick_resize_debounce();
            if !self.is_resize_debounce_active() {
                self.draw();
            }
            self.has_rendered_first_frame = true;
        }

        let quit = if self.ui_queue_has_pending() {
            matches!(self.process_signal(ui_session::UiSignal::QueueUpdated), UiEvent::Quit)
        } else if let Some(signal) = self.wait_for_signal() {
            matches!(self.process_signal(signal), UiEvent::Quit)
        } else {
            matches!(self.process_signal(ui_session::UiSignal::QueueUpdated), UiEvent::Quit)
        };
        !quit
    }

    fn wait_for_signal(&self) -> Option<ui_session::UiSignal> {
        if let Some(until) = self.next_debounce_deadline() {
            let now = Instant::now();
            if now >= until {
                return Some(ui_session::UiSignal::QueueUpdated);
            }
            let timeout = until.saturating_duration_since(now);
            self.ui_signal_rx.recv_timeout(timeout).ok()
        } else {
            self.ui_signal_rx.recv().ok()
        }
    }

    fn process_signal(&mut self, signal: ui_session::UiSignal) -> UiEvent {
        match signal {
            ui_session::UiSignal::QueueUpdated => {
                self.drain_ui_queue();
                self.tick_message_gutter_expiry();
                self.finish_non_keyboard_redraw();
                UiEvent::None
            }
            ui_session::UiSignal::Terminal(event) => self.handle_terminal_poll(event),
            ui_session::UiSignal::TerminalError => UiEvent::Quit,
        }
    }

    fn handle_terminal_poll(&mut self, event: TerminalPoll) -> UiEvent {
        let keyboard_input =
            matches!(&event, TerminalPoll::Paste(_) | TerminalPoll::Key(_));
        if keyboard_input {
            self.flush_pending_queue_redraw_for_keyboard();
        }
        match event {
            TerminalPoll::Resized { .. } => {
                self.note_terminal_resize();
                self.drain_ui_queue();
                let _ = self.flush_pending_redraw();
                UiEvent::None
            }
            TerminalPoll::Paste(paste) => {
                let current = self.current_focused_id();
                let _ = self.handle_text_input_paste(&paste);
                self.redraw_keyboard_current_element(current);
                self.finish_terminal_input_redraw(false);
                UiEvent::None
            }
            TerminalPoll::Key(key) => {
                let previous = self.current_focused_id();
                let (ui_event, full_immediate) = self.handle_key(key);
                if matches!(ui_event, UiEvent::Quit) {
                    return ui_event;
                }
                let current = self.current_focused_id();
                if !full_immediate {
                    self.redraw_keyboard_focused_elements(previous, current);
                }
                self.finish_terminal_input_redraw(full_immediate);
                ui_event
            }
        }
    }

    fn handle_key(&mut self, key: TerminalKey) -> (UiEvent, bool) {
        if self.handle_screen_scroll(key) {
            return (UiEvent::None, true);
        }

        if self.handle_display_scroll(key) {
            return (UiEvent::None, false);
        }

        if self.handle_text_input_editing(key) {
            return (UiEvent::None, false);
        }

        match key {
            TerminalKey::Quit | TerminalKey::Escape => (UiEvent::Quit, false),
            TerminalKey::Up | TerminalKey::Left { .. } => {
                self.focus_prev();
                (UiEvent::None, false)
            }
            TerminalKey::Down | TerminalKey::Right { .. } => {
                self.focus_next();
                (UiEvent::None, false)
            }
            TerminalKey::Enter | TerminalKey::Space => (self.activate_button_on_focus(), false),
            _ => (UiEvent::None, false),
        }
    }
}

impl Drop for RuntimeUi {
    fn drop(&mut self) {
        let _ = self.async_driver.take();
        let _ = curs_set(1);
        endwin();
        let _ = terminal_input::leave_raw_mode();
    }
}
