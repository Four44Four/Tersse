use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tokio::task::AbortHandle;
use tersse::{ElementId, RuntimeUi};

pub fn arm(
    ui: &Rc<RefCell<RuntimeUi>>,
    timer: &mut Option<AbortHandle>,
    id: ElementId,
    after: Duration,
    on_expire: impl FnOnce() + 'static,
) {
    timer.take().inspect(|t| t.abort());
    let ui = Rc::clone(ui);
    *timer = Some(
        tokio::task::spawn_local(async move {
            tokio::time::sleep(after).await;
            on_expire();
            let _ = ui.borrow_mut().remove_and_reflow(id);
        })
        .abort_handle(),
    );
}
