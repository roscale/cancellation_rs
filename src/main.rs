use std::{thread::sleep, time::Duration};

use crate::{
    channel::{Receiver, Sender, channel},
    jthread::JThread,
    stop_source::{StopSource, StopToken},
};

mod channel;
mod jthread;
mod stop_source;

fn main() {
    // Manual cancel + join with return value.
    {
        let printer = Printer::new();

        printer.print("page 1".to_string());
        printer.print("page 2".to_string());

        let pages_printed = printer.finish();
        println!("Done. Pages printed: {pages_printed}\n");
    }

    // Cancel + join when dropped.
    {
        {
            Printer::new().print("page 1".to_string());
        }
        println!("Done. Dropped.\n");
    }

    // Extenal cancel
    {
        let stop_source = StopSource::new();

        let printer = Printer::new_with_stop_source(stop_source.clone());
        printer.print("page 1".to_string());

        stop_source.request_stop();

        println!("Manually cancelled");
    }
}

struct Printer {
    tx: Sender<String>,
    thread: JThread<usize>,
}

impl Printer {
    pub fn new() -> Self {
        let (tx, rx) = channel::<String>();

        Self {
            tx,
            thread: jthread::spawn(|stop_token| printer(rx, stop_token)),
        }
    }

    pub fn new_with_stop_source(stop_source: StopSource) -> Self {
        let (tx, rx) = channel::<String>();

        Self {
            tx,
            thread: jthread::spawn_with_stop_source(
                |stop_token| printer(rx, stop_token),
                stop_source,
            ),
        }
    }

    pub fn print(&self, s: String) {
        self.tx.send(s);
    }

    pub fn finish(mut self) -> usize {
        self.thread.request_stop();
        self.thread.join()
    }
}

fn printer(rx: Receiver<String>, stop_token: StopToken) -> usize {
    let mut pages_printed = 0;

    loop {
        let Some(page) = rx.receive(&stop_token) else {
            break;
        };

        println!("Printing {page}...");
        sleep(Duration::from_secs(1));
        pages_printed += 1;
    }

    pages_printed
}
