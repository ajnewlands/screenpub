use log::{info, error};
use std::thread;
use std::sync::{Arc, Mutex, Condvar, mpsc};
use bytes::Bytes;

mod publisher;

fn main() -> std::io::Result<()> {
    match std::env::var_os("RUST_LOG") {
        Some(_) => (),
        None => std::env::set_var("RUST_LOG", "info,lapin=info,tokio_reactor=info,actix_server=info,actix_web=info"),
    };
    env_logger::init();

    let amqp = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let timeout: u64 = 5000;
    let (tx, rx): (mpsc::Sender<Bytes>, mpsc::Receiver<Bytes>) = mpsc::channel();

    for thread in 0..5 {
        let tx = tx.clone();
        let thread_name = format!("publisher-{}", (thread+1));
        let amqp = amqp.clone();
        let timeout = timeout.clone();

        let t = thread::Builder::new().name( thread_name ).spawn( move|| {
            info!("started {}", thread::current().name().unwrap_or_else(||"anonymous thread") );

            match publisher::Publisher::new(tx, amqp, timeout, "switchboard", "publisher") {
                Ok(publisher) => publisher.consume(),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            };
        });

        match t {
            Ok(_) => (),
            Err(e) => error!("Failed to launch thread: {:?}", e),
        };
    }
    drop (tx); // drop the original sender.

    // Accept messages from any remaining children; exit when all supervised threads are gone.
    loop {
        match rx.recv() {
            Ok(msg) => info!("Supervisor received channel message.."),
            Err(_) => break,
        };
    }

    Ok(())
}
