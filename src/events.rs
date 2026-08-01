use crossterm::event::{self, Event};
use tokio::sync::mpsc;

pub fn spawn_input(tx: mpsc::Sender<Event>) {
    std::thread::spawn(move || loop {
        match event::read() {
            Ok(ev) => {
                if tx.blocking_send(ev).is_err() {
                    break;
                }
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    });
}
