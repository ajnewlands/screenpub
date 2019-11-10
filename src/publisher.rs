use uuid::Uuid;
use std::{thread, panic};
use log::{info, warn, error, debug};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use std::io::{Error, ErrorKind};
use bytes::Bytes;

use lapin::{
    Channel, Connection, ConnectionProperties, BasicProperties, Queue,
    message::DeliveryResult, types::{FieldTable, ShortString},
    ExchangeKind, options::*
};

extern crate flatbuffers;
#[allow(unused_imports)]
mod messages_generated;
use messages_generated::switchboard::*;

mod util;
use util::string_to_header;

// In order to pass state between threaded callbacks, we create a cheaply clonable structure with 
// an Arc core.
#[derive(Clone)]
pub struct Publisher {
    inner: Arc<Inner>,
}

struct Inner {
    id: String,
    chan: Arc<Channel>,
    ex: String,
    shared_q: Queue,
    session_q: Queue,
    tx: Arc<Mutex<mpsc::Sender<Bytes>>>,
}

impl Publisher {
    pub fn new( tx: mpsc::Sender<Bytes>, amqp: String, timeout: u64, exchange: &str, queue: &str) -> Result<Publisher, std::io::Error> {
        let conn = Publisher::get_connection(amqp, timeout)?;
        let chan = Arc::new(Publisher::get_channel(&conn)?);
        let _ = Publisher::create_exchange(chan.clone(), exchange)?;
        let id =  Uuid::new_v4().to_string();
        let shared_q = Publisher::create_queue(chan.clone(), queue )?;
        let session_queue_name = format!("{}.{}", queue, id);
        let session_q = Publisher::create_queue(chan.clone(), &session_queue_name )?;
        info!("Publisher created with id {}", id);

        Publisher::create_shared_bindings(chan.clone(), queue, exchange)?;

        return Ok(Publisher{ 
            inner: Arc::new(Inner{
                id,
                chan: chan.clone(),
                ex: String::from(exchange),
                shared_q,
                session_q,
                tx: Arc::new(Mutex::new(tx)),
            }),
        });
    }

    /// Due to apparent deficiencies in Lapin, this won't return early when a connection is rejected.
    /// From experimentation, this seems to only be an issue on Windows (it will return immediately on Linux)
    fn get_connection(amqp: String, timeout: u64) -> Result<Connection, std::io::Error> {
        let (sender, receiver) = mpsc::channel();
        {
            let _t = thread::spawn(move ||{
                let conn = Connection::connect(&amqp, ConnectionProperties::default());
                match conn.wait() {
                    Ok(conn) => sender.send(conn).unwrap(),
                    Err(_) => drop(sender),
                };
            });
        }

        return match receiver.recv_timeout(Duration::from_millis(timeout)) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e)), 
        };
    }

    /// Get a channel for the connection
    fn get_channel(conn: &Connection) -> Result<Channel, std::io::Error> {
        return match conn.create_channel().wait() {
            Ok(ch) => match ch.basic_qos( 30, BasicQosOptions::default()).wait() {
                    Ok(_) => Ok(ch),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e)), 
            },
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e)),
        };
    }

    /// Declare the named exchange, creating it if it doesn exist already.
    fn create_exchange(chan: Arc<Channel>, exchange: &str) -> Result<&str, std::io::Error> {
        let opts = ExchangeDeclareOptions{ passive:false, durable: false, auto_delete: true, internal:false, nowait:false };
        return match chan.exchange_declare(exchange , ExchangeKind::Headers, opts, FieldTable::default()).wait() {
            Ok(_) => Ok(exchange),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e)),
        };
    }

    /// Declare the named queue (creating it if it doesn't exist)
    fn create_queue(chan: Arc<Channel>, queue: &str) -> Result<Queue, std::io::Error> {
        let opts = QueueDeclareOptions{ passive:false, durable:false, exclusive:false, auto_delete:true, nowait:false};
        return match chan.queue_declare(queue, opts, FieldTable::default()).wait() {
            Ok(q) => Ok(q),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e)),
        };
    }

    fn create_shared_bindings(chan: Arc<Channel>, queue: &str, exchange: &str) -> Result<(), std::io::Error> {
        let bindings = vec![ ("type", "ViewStart") ];
        Publisher::create_type_binding(chan.clone(), queue, exchange, bindings)?;

        Ok(())
    }

    fn get_view_update(&self, session: &str) -> bytes::Bytes {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let update = ViewUpdate::create(&mut builder, &ViewUpdateArgs{
            ..Default::default()
        });
        let ses = builder.create_string(session);

        let message = Msg::create(&mut builder, &MsgArgs{
            content_type: Content::ViewUpdate,
            session: Some(ses),
            content: Some(update.as_union_value()),
        });
        builder.finish(message, None);

        return bytes::Bytes::from(builder.finished_data());
    }

    fn dispatch_message(&self, message: bytes::Bytes, args: Vec<(&str, &str)>) -> Result<(), Error> {
        let mut headers = FieldTable::default();

        for (name, val) in args.iter(){
            debug!("adding header {}, value {}", name, val);
            headers.insert(ShortString::from(*name), string_to_header(val));
        }

        let props = BasicProperties::default().with_headers(headers);

        return self.inner.chan.basic_publish(&self.inner.ex, "", BasicPublishOptions::default(), message.to_vec(), props).wait()
            .map_err(|e| Error::new(ErrorKind::Other, e));
    }

    fn create_type_binding(chan: Arc<Channel>, queue: &str, exchange: &str, bindings: Vec<(&str, &str)> ) -> Result<(), std::io::Error> {
        let mut fields = FieldTable::default();

        for (name, val) in bindings.iter(){
            fields.insert(ShortString::from(*name), string_to_header(val));
        }

        return chan.queue_bind(queue, exchange, "", QueueBindOptions::default(), fields)
            .wait()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Unable to bind to queue {}: {:?}", queue, e)));
    }

    //fn create_session_bindings(chan: Arc<Channel>, queue: &str, exchange: &str, session: &str) -> Result<(), std::io::Error> {
    fn create_session_bindings(&self, session: &str) -> Result<(), std::io::Error> {
        let session_queue_name = format!("publisher.{}", self.inner.id);
        Publisher::create_type_binding(self.inner.chan.clone(), &session_queue_name.clone(), &self.inner.ex, vec![ ("session", session), ("type","ViewAck")] )?;
        Publisher::create_type_binding(self.inner.chan.clone(), &session_queue_name.clone(), &self.inner.ex, vec![ ("session", session), ("type", "ViewEnd")] )?;

        Ok(())
    }

    fn handle_message( &self, message: Msg, rabbit_headers: &FieldTable ) {
        let session =  match &(rabbit_headers.inner())["session"]  {
            lapin::types::AMQPValue::LongString(s) => s.as_str(),
            _ => "",
        };
        let dest_id =  match &(rabbit_headers.inner())["sender_id"]  {
            lapin::types::AMQPValue::LongString(s) => s.as_str(),
            _ => "",
        };

        debug!("got a message of type {:?} for session {}", message.content_type(), session);
        match message.content_type() {
            Content::ViewStart => {
                match self.create_session_bindings(&session) {
                    Ok(_) => {
                        debug!("Created bindings for session {}", session);
                        self.stop_consumer("shared");
                        let update = self.get_view_update(session);
                        self.dispatch_message(update, vec![ ("type", "ViewUpdate"), ("sender_id", &self.inner.id), ("session",session), ("dest_id", dest_id) ]);
                    },
                    Err(e) => error!("Unable to create session bindings for session {}: {:?}", session, e),
                };
            },
            Content::ViewAck => {
                match self.create_session_bindings(&session) {
                    Ok(_) => {
                        let update = self.get_view_update(session);
                        self.dispatch_message(update, vec![ ("type", "ViewUpdate"), ("sender_id", &self.inner.id), ("session",session), ("dest_id", dest_id) ]);
                    },
                    Err(e) => error!("Unable to create session bindings for session {}: {:?}", session, e),
                };
            },
            x => warn!("unhandled message type {:?}", x),
        };
    }

    pub fn stop_consumer(&self, tag: &str) -> Result<(), std::io::Error> {
        self.inner.chan.basic_cancel(tag, BasicCancelOptions::default()).wait()
            .map_err(|e| Error::new(ErrorKind::ConnectionReset, e))?;
        Ok(())
    }

    // Attach consumers to both the shared queue and the session-specific queue.
    pub fn consume(&self) -> Result<(), std::io::Error> {
        let exchange = self.inner.ex.clone();
        let chan = self.inner.chan.clone();

        // Preconfigure delegate to handle session messages.
        let session_self = self.clone(); 
        let session_opts = BasicConsumeOptions{ no_local: true, no_ack: false, exclusive: true, nowait: false };
        let session_consumer = chan.basic_consume(&self.inner.session_q, "session", session_opts, FieldTable::default()).wait()
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;

        session_consumer.set_delegate( Box::new( move | delivery: DeliveryResult | {
            match delivery {
                Ok(Some(delivery)) => {
                    // reading the flatbuffer will panic if it is invalid; catch_unwind will
                    // prevent the program from summarily aborting.
                    match delivery.properties.headers().as_ref() {
                        None => error!("Received message has no headers"), // should be impossible, given our bindings
                        Some(headers) => match panic::catch_unwind(|| get_root_as_msg(&delivery.data)) {
                            Ok(msg) => session_self.handle_message(msg, headers),
                            Err(_) => error!("Dropping invalid message"),
                        }
                    }
                    chan.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).wait().expect("ACK failed")
                },
                Ok(None) => warn!("Session consumer cancelled"),
                Err(e) => error!("Consumer error {}", e),
            };
        }));
        

        // Then start listening for messages on the shared queue.
        let shared_self = self.clone(); 
        let shared_opts = BasicConsumeOptions{ no_local: true, no_ack: false, exclusive: false, nowait: false };
        let shared_consumer = self.inner.chan.basic_consume(&self.inner.shared_q, "shared", shared_opts, FieldTable::default()).wait()
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;
        let chan = self.inner.chan.clone();

        shared_consumer.set_delegate(Box::new(move | delivery: DeliveryResult |{ 
            match delivery {
                Ok(Some(delivery)) => {
                    // reading the flatbuffer will panic if it is invalid; catch_unwind will
                    // prevent the program from summarily aborting.
                    match delivery.properties.headers().as_ref() {
                        None => error!("Received message has no headers"), // should be impossible, given our bindings
                        Some(headers) => match panic::catch_unwind(|| get_root_as_msg(&delivery.data)) {
                            Ok(msg) => shared_self.handle_message(msg, headers),
                            Err(_) => error!("Dropping invalid message"),
                        }
                    }
                    chan.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).wait().expect("ACK failed")
                }, 
                Ok(None) => info!("Shared consumer cancelled for {}", shared_self.inner.id), // Consumer cancelled
                Err(e) => error!("Shared consumer error {}", e),
            };
        }));

        Ok(())
    }
}
