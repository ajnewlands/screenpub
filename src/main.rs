use log::{info, error};
use std::thread;
use std::thread::JoinHandle;

mod publisher;

fn main() -> std::io::Result<()> {
    match std::env::var_os("RUST_LOG") {
        Some(_) => (),
        None => std::env::set_var("RUST_LOG", "info,lapin=info,tokio_reactor=info,actix_server=info,actix_web=info"),
    };
    env_logger::init();

    let amqp = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672".into());
    let timeout: u64 = 5000;
    let mut threads = Vec::<JoinHandle<_>>::new();
    for thread in 0..5 {
        let thread_name = format!("publisher-{}", (thread+1));
        let amqp = amqp.clone();
        let timeout = timeout.clone();

        let t = thread::Builder::new().name( thread_name ).spawn( move|| {
            info!("started {}", thread::current().name().unwrap_or_else(||"anonymous thread") );
            let publisher = publisher::Publisher::new(amqp, timeout, "switchboard", "publisher").unwrap();
            match publisher.consume_shared() {
                Err(e) => error!("Consumer error: {:?}", e),
                _ => (),
            };
        });

        match t {
            Ok(thr) => threads.push(thr),
            Err(e) => error!("Failed to launch thread: {:?}", e),
        };
    }

    for thread in threads {
        thread.join();
    }

    Ok(())
}
