use crossterm::event::{Event, EventStream, MouseEventKind};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{self, Duration, MissedTickBehavior};

use crate::app::AppEvent;

pub async fn read_crossterm_events(tx: mpsc::UnboundedSender<AppEvent>) {
    let mut reader = EventStream::new();
    let mut ticker = time::interval(Duration::from_millis(120));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
            maybe_event = reader.next() => {
                let Some(Ok(event)) = maybe_event else {
                    break;
                };
                let app_event = match event {
                    Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => {
                        AppEvent::Key(key)
                    }
                    Event::Resize(w, h) => AppEvent::Resize(w, h),
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::ScrollUp => AppEvent::MouseScroll { delta: -3, row: mouse.row },
                        MouseEventKind::ScrollDown => AppEvent::MouseScroll { delta: 3, row: mouse.row },
                        _ => continue,
                    },
                    _ => continue,
                };
                if tx.send(app_event).is_err() {
                    break;
                }
            }
        }
    }
}
