use uuid::Uuid;
use std::panic;
use log::{info, warn, error, debug};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Error, ErrorKind};

use mtpng::encoder::{Encoder, Options};
use mtpng::{ColorType, CompressionLevel, Header};

use amiquip::{Connection, Publish, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, ExchangeType, ExchangeDeclareOptions, FieldTable, AmqpValue, AmqpProperties, Channel };

extern crate flatbuffers;
use flatbuffers::FlatBufferBuilder;
#[allow(unused_imports)]
mod messages_generated;
use messages_generated::switchboard::*;

#[path="snapscreen/snapscreen.rs"]
mod snapscreen;
use snapscreen::Snapper;

pub struct Publisher {
    id: String,
    chan: Rc<Channel>,
    _conn: Rc<RefCell<Connection>>, // hold a reference so it is not dropped prematurely.
    ex: String,
    queue: String,
}

impl Publisher {
    pub fn new( amqp: String, timeout: u64, exchange: &str, queue: &str) -> Result<Publisher, std::io::Error> {
        let conn = Rc::new(RefCell::new(Publisher::get_connection(&amqp, timeout)?));
        let chan = Rc::new(Publisher::get_channel( &mut conn.borrow_mut() )?);
        Publisher::declare_exchange(chan.clone(), exchange)?;
        let id =  Uuid::new_v4().to_string();

        info!("Publisher created with id {}", id);
        return Ok(Publisher{ 
            id,
            chan: chan.clone(),
            _conn: conn.clone(),
            ex: String::from(exchange),
            queue: String::from(queue),
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
    fn declare_exchange(chan: Rc<Channel>, exchange: &str) -> Result<(), std::io::Error> {
        let opts = ExchangeDeclareOptions{ durable: false, auto_delete: true, internal:false, arguments: FieldTable::default() };
        chan.exchange_declare(ExchangeType::Headers, exchange, opts )
            .map_err(|e| Error::new(ErrorKind::NotConnected, e))?;

        Ok(()) // what was I thinking?? 
    }

    fn create_shared_bindings(&self) -> Result<(), String> {
        let bindings = vec![ ("type", "ViewStart") ];
        self.create_type_binding(&self.queue, bindings)?;

        Ok(())
    }

    fn get_view_update(&self, session: &str, snapper: &mut Snapper, mut builder: &mut FlatBufferBuilder, incremental: bool) -> bytes::Bytes {
        return match incremental {
            true => self.get_incremental_view_update(session, snapper, builder),
            false => self.get_full_view_update(session, snapper, builder),
        };
    }

    fn get_full_view_update(&self, session: &str, snapper: &mut Snapper, mut builder: &mut FlatBufferBuilder) -> bytes::Bytes {
        let screen = snapper.snap();
        let writer = Vec::<u8>::new();
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
        debug!("Encoded screen to size {} in {}", png.len(), now.elapsed().as_millis());

        let data = builder.create_vector_direct(&png);
        let update = ViewUpdate::create(&mut builder, &ViewUpdateArgs{ sqn: 0, incremental: false, data: Some(data), tiles: None });
        let ses = builder.create_string(session);

        let message = Msg::create(&mut builder, &MsgArgs{
            content_type: Content::ViewUpdate,
            session: Some(ses),
            content: Some(update.as_union_value()),
        });

        builder.finish(message, None);
        let bytes = bytes::Bytes::from(builder.finished_data());
        builder.reset();

        return bytes;
    }

    fn get_incremental_view_update(&self, session: &str, snapper: &mut Snapper, mut builder: &mut FlatBufferBuilder) -> bytes::Bytes {
        let hextiles = snapper.snap_hextile();

        let mut vtiles = Vec::<flatbuffers::WIPOffset<Tile>>::with_capacity(hextiles.len());
        for hex in &hextiles {
            println!("vector length is {}", hex.tile.len());
            let data = builder.create_vector_direct(&hex.tile);
            vtiles.push( Tile::create(&mut builder, &TileArgs{x: hex.x, y: hex.y, data: Some(data) } ));
        }
        let tiles = builder.create_vector(&vtiles);

        info!("Incremental update with {} changed tiles", hextiles.len());
        let update = ViewUpdate::create(&mut builder, &ViewUpdateArgs{ sqn: 0, incremental: true, data: None, tiles: Some(tiles) });
        let ses = builder.create_string(session);

        let message = Msg::create(&mut builder, &MsgArgs{
            content_type: Content::ViewUpdate,
            session: Some(ses),
            content: Some(update.as_union_value()),
        });

        builder.finish(message, None);
        let bytes = bytes::Bytes::from(builder.finished_data());
        builder.reset();

        return bytes;
    }

    fn dispatch_message(&self, message: bytes::Bytes, args: Vec<(&str, &str)>) -> Result<(), Error> {
        let mut headers = FieldTable::default();

        for (name, val) in args.iter(){
            debug!("adding header {}, value {}", name, val);
            headers.insert(String::from(*name), AmqpValue::LongString(String::from(*val)));
        }

        let props = AmqpProperties::default().with_headers(headers);

        let opts = Publish{ body: &message, routing_key: String::from(""), mandatory: false, immediate: false, properties: props };
        self.chan.basic_publish(self.ex.clone(), opts);

        Ok(())
    }

    fn create_type_binding(&self, queue: &str, bindings: Vec<(&str, &str)> ) -> Result<(), String> {
        let mut fields = FieldTable::default();

        for (name, val) in bindings.iter(){
            fields.insert(String::from(*name), AmqpValue::LongString(String::from(*val)));
        }

        self.chan.queue_bind( queue, &self.ex, "", fields )
            .map_err(|e| format!("Could not bind queue: {:?}", e))?;

        Ok(())
    }

    fn consume_session(&self, session: &str, dest_id: &str, incremental: bool) -> Result<(), String> {
        let session_queue = format!("publisher.{}", self.id);
        let opts = QueueDeclareOptions{ durable: false, exclusive: true, auto_delete: true, arguments: FieldTable::default() };

        let queue = self.chan.queue_declare(session_queue.clone(), opts)
            .map_err(|e| format!("Failed to declare queue; {:?}", e))?;

        self.create_type_binding(&session_queue.clone(), vec![ ("session", session), ("type","ViewAck")] )?;
        self.create_type_binding(&session_queue, vec![ ("session", session), ("type", "ViewEnd")] )?;

        let consumer = queue.consume(ConsumerOptions::default())
            .map_err(|e| format!("Failed to consume from queue; {:?}", e ))?;

        // BEWARE scrap on windows can only exist in one thread at a time. It's broken!
        let mut snapper = Snapper::new();
        let mut builder = FlatBufferBuilder::new();
        let update = self.get_view_update(&session, &mut snapper, &mut builder, false);
        self.dispatch_message(update, vec![ ("type", "ViewUpdate"), ("sender_id", &self.id), ("session", &session), ("dest_id", &dest_id) ]);

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    match delivery.properties.headers().as_ref() {
                        None => error!("Rabbit message received without headers"),
                        Some(headers) => {
                            consumer.ack(delivery.clone())
                                .map_err(|e| format!("Rabbit rejected an ack: {:?}", e))?;

                            match panic::catch_unwind(|| get_root_as_msg(&delivery.body)) {
                                Ok(msg) => {
                                    match msg.content_type() {
                                        Content::ViewAck => {
                                            debug!("Sending update for session {}", session);
                                            let update = self.get_view_update(&session, &mut snapper, &mut builder, incremental );
                                            self.dispatch_message(update, vec![ ("type", "ViewUpdate"), ("sender_id", &self.id), ("session", &session), ("dest_id", &dest_id) ]);
                                        },
                                        Content::ViewEnd => {
                                            warn!("Got a ViewEnd - dropping session");
                                            consumer.cancel();
                                        },
                                        t =>  warn!("Dropping unhandled message type {:?}", t),
                                    };
                                },
                                Err(_) => error!("Dropping invalid message"),
                            };
                        },
                    };
                },
                e => {
                    debug!("Consumer ended in thread {}: {:?}", self.id, e);
                    break;
                },
            }
        }
        drop(snapper);
        self.consume_shared();

        Ok(())
    }

    fn get_header_str( header: &str, headers: &FieldTable ) -> Result<String, String> {
        if let AmqpValue::LongString(session) = &headers[header] {
            return Ok(String::from(session));
        } else {
            return Err( format!("Discarding message without {} header", header));
        }
    }

    fn handle_view_start(&self, message: &Msg, headers: &FieldTable) -> Result<(), String> {
        match message.content_type() {
            Content::ViewStart => {
                let session = Publisher::get_header_str("session", headers)?;
                info!("Starting view updates for session {}", session);
                let dest_id = Publisher::get_header_str("sender_id", headers)?;
                let incremental = match message.content_as_view_start().unwrap().capabilities() {
                    0 => false,
                    _ => true,
                }; // rethink this? should the ACK specify the next mode?
                self.consume_session(&session, &dest_id, incremental)?;
            },
            t => warn!("Expected ViewStart but got {:?}", t),
        };

        Ok(())
    }

    pub fn consume_shared(&self) -> Result<(), String> {
        let opts = QueueDeclareOptions{ durable: false, exclusive: false, auto_delete: true, arguments: FieldTable::default() };
        let chan = self.chan.clone();
        let queue = chan.queue_declare(self.queue.clone(), opts)
            .map_err(|e| format!("Failed to declare queue; {:?}", e))?;

        self.create_shared_bindings()
            .map_err(|e| format!("Failed to create shared bindings; {:?}", e ))?;

        let shared_consumer = queue.consume(ConsumerOptions::default())
            .map_err(|e| format!("Failed to consume from queue; {:?}", e ))?;

        for message in shared_consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    match delivery.properties.headers().as_ref() {
                        None => error!("Rabbit message received without headers"),
                        Some(headers) => {
                            shared_consumer.ack(delivery.clone())
                                .map_err(|e| format!("Rabbit rejected an ack: {:?}", e))?;

                            match panic::catch_unwind(|| get_root_as_msg(&delivery.body)) {
                                Ok(msg) => {
                                    debug!("Got a '{:?}' message at {}", msg.content_type(), (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis()));
                                    match msg.content_type() {
                                        Content::ViewStart => {
                                            shared_consumer.cancel();
                                            self.handle_view_start(&msg, headers)?;
                                        },
                                        t => warn!("Dropping unhandled message type {:?}", t),
                                    }

                                },
                                Err(_) => error!("Dropping invalid message"),
                            };
                        },
                    };
                },
                e => {
                    debug!("Consumer ended in thread {}: {:?}", self.id, e);
                    break;
                },
            }
        }

        Ok(())
    }
}
