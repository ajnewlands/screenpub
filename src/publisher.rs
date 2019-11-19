use uuid::Uuid;
use std::{thread, panic};
use log::{info, warn, error, debug};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::time::Duration;
use std::io::{Error, ErrorKind};
use bytes::Bytes;

use mtpng::encoder::{Encoder, Options};
use mtpng::{ColorType, CompressionLevel, Header};

use amiquip::{Connection, Publish, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, ExchangeType, ExchangeDeclareOptions, FieldTable, AmqpValue, AmqpProperties, Channel, Queue};

extern crate flatbuffers;
#[allow(unused_imports)]
mod messages_generated;
use messages_generated::switchboard::*;

#[path="snapscreen/snapscreen.rs"]
mod snapscreen;
use snapscreen::Snapper;

// In order to pass state between threaded callbacks, we create a cheaply clonable structure with 
// an Arc core.
#[derive(Clone)]
pub struct Publisher {
    inner: Arc<Inner>,
}

struct Inner {
    id: String,
    chan: Arc<Channel>,
    conn: Connection,
    ex: String,
}

impl Publisher {
    pub fn new( amqp: String, timeout: u64, exchange: &str, queue: &str) -> Result<Publisher, std::io::Error> {
        let mut conn = Publisher::get_connection(&amqp, timeout)?;
        let chan = Arc::new(Publisher::get_channel(&mut conn)?);
        Publisher::declare_exchange(chan.clone(), exchange)?;
        let id =  Uuid::new_v4().to_string();

        info!("Publisher created with id {}", id);
        return Ok(Publisher{ 
            inner: Arc::new(Inner{
                id,
                chan: chan.clone(),
                conn,
                ex: String::from(exchange),
            }),
        });
    }

    /// Due to apparent deficiencies in Lapin, this won't return early when a connection is rejected.
    /// From experimentation, this seems to only be an issue on Windows (it will return immediately on Linux)
    fn get_connection(amqp: &str, _timeout: u64) -> Result<Connection, std::io::Error> {
        let connection = Connection::insecure_open( amqp )
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;
        Ok(connection)
    }

    /// Get a channel for the connection
    fn get_channel(conn: &mut Connection) -> Result<Channel, std::io::Error> {
        let channel = conn.open_channel(None)
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;
        
        Ok(channel)
    }

    /// Declare the named exchange, creating it if it doesn exist already.
    fn declare_exchange(chan: Arc<Channel>, exchange: &str) -> Result<(), std::io::Error> {
        let opts = ExchangeDeclareOptions{ durable: false, auto_delete: true, internal:false, arguments: FieldTable::default() };
        chan.exchange_declare(ExchangeType::Headers, exchange, opts )
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;

        Ok(()) // what was I thinking?? 
    }

    fn create_shared_bindings(chan: Arc<Channel>, queue: &str, exchange: &str) -> Result<(), std::io::Error> {
        let bindings = vec![ ("type", "ViewStart") ];
        Publisher::create_type_binding(chan.clone(), queue, exchange, bindings)?;

        Ok(())
    }

    fn get_view_update(&self, session: &str) -> bytes::Bytes {
        // Get an initial screenshot
        //let screen = self.inner.snapper.write().unwrap().snap();
        let mut snapper = Snapper::new();
        let screen = snapper.snap();
        let mut writer = Vec::<u8>::new();
        let mut header = Header::new();
        header.set_size( ((screen.len() / snapper.height) /4) as u32, snapper.height as u32 );
        header.set_color( ColorType::TruecolorAlpha, 8).expect("set color died mysteriously");

        let now = std::time::Instant::now();
        let mut options = Options::new();
        options.set_compression_level(CompressionLevel::Fast);
        let mut encoder = Encoder::new(writer, &options);

        encoder.write_header(&header).expect("failed writing header");
        encoder.write_image_rows(&screen).expect("failed writing rows");
        let png = encoder.finish().unwrap();
        info!("Encoded screen to size {} in {}", png.len(), now.elapsed().as_millis());

        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let data = builder.create_vector_direct(&png);
        let update = ViewUpdate::create(&mut builder, &ViewUpdateArgs{ sqn: 4, incremental: false, data: Some(data) });
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
            headers.insert(String::from(*name), AmqpValue::LongString(String::from(*val)));
        }

        let props = AmqpProperties::default().with_headers(headers);

        let opts = Publish{ body: &message, routing_key: String::from(""), mandatory: false, immediate: false, properties: props };
        self.inner.chan.basic_publish(self.inner.ex.clone(), opts);

        Ok(())
    }

    fn create_type_binding(chan: Arc<Channel>, queue: &str, exchange: &str, bindings: Vec<(&str, &str)> ) -> Result<(), std::io::Error> {
        let mut fields = FieldTable::default();

        for (name, val) in bindings.iter(){
            fields.insert(String::from(*name), AmqpValue::LongString(String::from(*val)));
        }

        chan.queue_bind( queue, exchange, "", fields )
            .map_err(|e| Error::new(ErrorKind::ConnectionReset, e))?;

        Ok(())
    }

    //fn create_session_bindings(chan: Arc<Channel>, queue: &str, exchange: &str, session: &str) -> Result<(), std::io::Error> {
    fn create_session_bindings(&self, session: &str) -> Result<(), std::io::Error> {
        let session_queue_name = format!("publisher.{}", self.inner.id);
        Publisher::create_type_binding(self.inner.chan.clone(), &session_queue_name.clone(), &self.inner.ex, vec![ ("session", session), ("type","ViewAck")] )?;
        Publisher::create_type_binding(self.inner.chan.clone(), &session_queue_name.clone(), &self.inner.ex, vec![ ("session", session), ("type", "ViewEnd")] )?;

        Ok(())
    }

    /*
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
              let update = self.get_view_update(session);
              info!("Received ACK at {}",  (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis()));
              self.dispatch_message(update, vec![ ("type", "ViewUpdate"), ("sender_id", &self.inner.id), ("session",session), ("dest_id", dest_id) ]);
            },
            x => warn!("unhandled message type {:?}", x),
        };
    }
    */

    /*
    pub fn stop_consumer(&self, tag: &str) -> Result<(), std::io::Error> {
        self.inner.chan.basic_cancel(tag, BasicCancelOptions::default()).wait()
            .map_err(|e| Error::new(ErrorKind::ConnectionReset, e))?;
        Ok(())
    }
    */

    pub fn consume(&self) -> Result<(), String> {
        let opts = QueueDeclareOptions{ durable: false, exclusive: false, auto_delete: true, arguments: FieldTable::default() };
        let chan = self.inner.chan.clone();
        let queue = chan.queue_declare("publisher", opts)
            .map_err(|e| format!("Failed to declare queue; {:?}", e))?;

        Publisher::create_shared_bindings(self.inner.chan.clone(), "publisher", &self.inner.ex)
            .map_err(|e| format!("Failed to create shared bindings; {:?}", e ))?;

        let shared_consumer = queue.consume(ConsumerOptions::default())
            .map_err(|e| format!("Failed to consume from queue; {:?}", e ))?;

        for (i,message) in shared_consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    println!("Got a message at {}", (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis()));
                },
                e => {
                    println!("Consumer ended {:?}", e);
                    break;
                },
            }
        }
        println!("Done consuming!");

        Ok(())
    }
    /*
    // Attach consumers to both the shared queue and the session-specific queue.
    pub fn consume(&self) -> Result<(), std::io::Error> {
        let exchange = self.inner.ex.clone();
        let chan = self.inner.chan.clone();

        // Preconfigure delegate to handle session messages.
        let session_self = self.clone(); 
        let session_opts = BasicConsumeOptions{ no_local: true, no_ack: false, exclusive: true, nowait: true };
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
        let shared_opts = BasicConsumeOptions{ no_local: true, no_ack: false, exclusive: false, nowait: true };
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
    */
}
