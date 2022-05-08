//! Simple websocket client.

use std::{io, thread};
use awc::{ ws::{Frame}};
use actix_http::ws::Item;
use tokio::{select, sync::mpsc};
use futures_util::{SinkExt as _};
use bytes::Bytes;
use futures::stream::{StreamExt};
use std::convert::TryFrom;

/*

// OLD 

use std::{io, thread};
use actix::io::SinkWrite;
use actix::Arbiter;
use actix::Context;
use actix::*;
use bytestring::ByteString;

use actix_codec::{Framed};

use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket,
};
use awc::ClientResponse;
use actix_http::ws::Item;

//use awc::Client;
//use actix_web::client::Client;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::{select, sync::mpsc};
use futures_util::{SinkExt as _, StreamExt as _};

use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt,SplitStream};
use std::time::{Duration, Instant};
use std::convert::TryFrom;

/// Как часто отправляются эхо-запросы сердцебиения
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(400);
/// TIMEOUT проверка присутствия клиента на линии
const SERVER_TIMEOUT: Duration = Duration::from_secs(500);

struct ActorClient{
    writer: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    duration_heartbeat: Instant,
}

impl actix::io::WriteHandler<WsProtocolError> for ActorClient {
    fn error(&mut self, err: WsProtocolError, ctx: &mut Self::Context) -> Running {
        eprintln!("{:?}",err);
       // Io(Os { code: 32, kind: BrokenPipe, message: "Broken pipe" })
       // https://cachelogica.com/en/os-error-code-32-broken-pipe/
       
       //  Чтобы недопустить отправки сообщения в незавершенный канал, следует дождаться от сервера обратной команды !
       
       Running::Stop
    }
}

impl Actor for ActorClient {
    type Context = Context<Self>;// Struct actix::prelude::Context 

    fn started(&mut self, ctx: &mut Self::Context) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        println!("Client: Started");
        self.heartbeat(ctx)
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("Client: Disconnected");
        // Stop application on disconnect
        System::current().stop();
    }
}

impl ActorClient {
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        // Context ctx сдесь это Struct actix::prelude::Context<ActorClient>
        // имеет свои собственные методы: 
        // connected, handle, into_future, new, run, set_mailbox_capacity, with_receiver
        
        // от Trait  actix::Actor досталось:
        //    - create, start, start_default, start_in_arbiter, started, stopped, stopping

        // от Trait actix::prelude::ActorContext досталось:
        //    - stop, terminate, state

        // от Trait actix::prelude::AsyncContext досталось:
        //    - address, cancel_future, spawn, wait, waiting, (далее реализованные самим трейтом) add_message_stream, add_stream, notify, notify_later, run_interval, run_later

        // Trait Debug 

        // Trait  AsyncContextParts 
        //    -  parts

        ctx.run_later(HEARTBEAT_INTERVAL, |act, ctx| {
            act.writer.write(Message::Ping(Bytes::from_static(b"msg client to server")));
            act.heartbeat(ctx);// еще раз вызвать себя, можно переделать на ctx.run_interval

            // клиент также должен проверить здесь тайм-аут, аналогично коду сервера
            if Instant::now().duration_since(act.duration_heartbeat) > SERVER_TIMEOUT {
                // heartbeat timed out
                println!("Client:Websocket Client heartbeat failed, disconnecting!");
                // stop actor
                ctx.stop();
                // не пытайтесь отправить пинг
                return;
            }
        });

    }
}


/// Handle server websocket messages
/// Trait actix::prelude::StreamHandler добавляет методы
///     - add_stream, finished, started
impl StreamHandler<Result<Frame, WsProtocolError>> for ActorClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, ctx: &mut Self::Context) {
        
            match msg {
                Ok(Frame::Ping(p_msg)) => {
                    println!("Client:Type msg Ping: {:?}", p_msg);
                    self.duration_heartbeat = Instant::now();
                },
                Ok(Frame::Pong(_)) => {
                   // println!("Client:Type msg Pong");
                    self.duration_heartbeat = Instant::now();
                },
                Ok(Frame::Text(text)) => {
                   // println!("Client:Type msg Text: {:?}", text);
                    self.duration_heartbeat = Instant::now();
                },
                Ok(Frame::Binary(bin)) => {
                    //println!("Client:Type msg Binary: {:?}", bin);
                    self.duration_heartbeat = Instant::now();

                    let size_byte = 4096;//65536-1 4096 16384 32768   (max=65536 2 byte/ 16bit)
                    let buff = (0..=size_byte).map(|_|78).collect::<Vec<u8>>();

                    if let Ok(n) = Number::try_from(bin){
                        match n.0 {
                            0 => {
                               self.writer.write( Message::Continuation( Item::FirstBinary( Bytes::from_iter(buff.clone())))); 
                            },
                            e @ 1..=8 => {
                                self.writer.write( Message::Continuation( Item::Continue( Bytes::from_iter(buff.clone())))); 
                            },
                            e @ 9  => {
                                self.writer.write( Message::Continuation( Item::Last( Bytes::from_iter(buff.clone()))));
                            },
                           
                            _ => {
                                println!("Client:send end");
                            }
                        }
                    }

                },
                Ok(Frame::Close(reason)) => {
                    println!("Client:Type msg Close");
                    ctx.stop();
                }
                _ => (),
            } 
    }

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Client: Connected");
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("Client: Server disconnected");
        ctx.stop()
    }
}

// ------------------------------------------------------------------------------------
// Доплнительный источник команд

// https://actix.rs/actix/actix/#derives
#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(Command);

enum Command{
    ReadySend(String)
}

/// Еще один обработчик из консоли
/// Handle stdin commands
impl Handler<ClientCommand> for ActorClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Self::Context) -> Self::Result  {
        log::info!("handle");

        // Запуск сценария обмена
         match msg.0 {
            Command::ReadySend(s) => {
                log::info!("send command");
                self.writer.write(Message::Text(bytestring::ByteString::from(s))).unwrap();
                
            },
            _ =>{
                log::error!("command not found");
            }
         }
        // self.writer.write(Message::Text("ready load".to_string()));
        // self.writer.write(Message::Binary(Bytes::from_iter( msg.0 )));
    }
}
// ------------------------------------------------------------------------------------

fn main() {
    // cargo run --bin websocket-client

    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = System::new(); 
    // Арбитры обеспечивают асинхронную среду выполнения для субъектов, функций и фьючерсов.
    // Когда создается Арбитр, он порождает новый поток ОС и размещает цикл событий.
    // Некоторые функции арбитра выполняются в текущем потоке.
    let arbitr = actix::Arbiter::new();

     arbitr.spawn(async {
        // Connect to server
        //-----------------------------
        
       let webRequest:awc::ws::WebsocketsRequest = awc::ClientBuilder::new()
        //.disable_timeout()
        //.initial_connection_window_size(65540)
        .finish()
        .ws("ws://192.168.0.104:4011/ws/");

        let (response, framed):(ClientResponse,Framed<BoxedSocket, Codec>) = webRequest
            //.max_frame_size(65540)
            //.server_mode()
            .connect()
            .await
            .map_err(|e| {
                println!("Error build: {}", e);
            })
            .unwrap();
            
        //-----------------------------
        // or
            let (response, framed):(ClientResponse,Framed<BoxedSocket, Codec>) = awc::Client::new()
                .ws("ws://192.168.0.104:4011/ws/")
                .connect()
                .await
                .map_err(|e| {
                    println!("Error: {}", e);
                })
                .unwrap();
        
        println!("Response {:#?}", response);

         // Struct actix_codec::Framed
         // Struct awc::BoxedSocket
         // Struct actix_http::Codec
         // Struct futures_util::stream::SplitSink
         // Struct futures_util::stream::SplitStream  

        let (sink, stream):
        (
            SplitSink<Framed<BoxedSocket, Codec>, awc::ws::Message>, 
            SplitStream<Framed<BoxedSocket, Codec>>
        ) = framed.split();// https://docs.rs/futures-util/0.3.17/futures_util/stream/trait.StreamExt.html#method.split

        let addr = ActorClient::create(|mut ctx| { // create от Trait actix::Actor
            println!("{:?}",ctx);// Context { parts: ContextParts { flags: RUNNING }, mb: Some(Mailbox { capacity: 16 }) }
            
            // let addr:actix::Addr<ActorClient> = ctx.address();
           
            ActorClient::add_stream(stream, ctx);// add_stream от Trait actix::prelude::StreamHandler
            ActorClient{
                writer:SinkWrite::new(sink, ctx),
                duration_heartbeat: Instant::now()
            }
        });
        //-----------------------------

        // Также можно запустить в отдельном потоке прослушивание команд консоли и передавать их серверу
        thread::spawn(move || loop {
            let mut cmd = String::new();
            if io::stdin().read_line(&mut cmd).is_err() {
                println!("error");
                return;
            }
            if cmd == "start\n".to_string(){ 
                addr.do_send(ClientCommand( Command::ReadySend("ready send".to_string()) ));   
            }   
        });

    });
    sys.run().unwrap();
}*/
 


#[derive(Debug)]
struct Number(usize);

impl TryFrom<Bytes> for Number {
    type Error = String;
    fn try_from(item: Bytes) -> Result<Self, Self::Error> {
        let v = item.into_iter().collect::<Vec<u8>>();
        match v.try_into() {
            Ok(arr) =>{ Ok(Number(usize::from_be_bytes(arr))) },
            Err(e) => { Err(format!("{:?}",e)) }
        }
    }
}

// Клиент через консоль сигнализирует серверу о начале загрузки, сервер посылает команду bin в обработчик ActorClient
// и начинается отправка пакетов, когда сервер отправит определенное количество полученных пакетов, клиент прекратит загрузку


#[actix_rt::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting echo WebSocket client");

     
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(cmd_rx);

    // Также можно запустить в отдельном потоке прослушивание команд консоли и передавать их серверу
    // run blocking terminal input reader on separate thread
    let input_thread = thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }
        if cmd == "start\n".to_string(){ 
            cmd_tx.send("ready send".to_string()).unwrap();
           // addr.do_send(ClientCommand( Command::ReadySend("ready send".to_string()) ));   
        } 
    }); 
     
    let (res, mut framed) = awc::Client::new()
        .ws("ws://192.168.2.104:4011/ws/")
        .connect()
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
        })
        .unwrap();
 
  
        log::info!("connected; server will echo messages sent");
 
        loop {
        select! {
            Some(msg) = framed.next() => {
                match msg {
                    Ok(awc::ws::Frame::Text(txt)) => {
                        // log echoed messages from server
                        log::info!("Client:Type msg Text: {:?}", txt)
                    }

                    Ok(awc::ws::Frame::Ping(_)) => {
                        log::info!("Client:Type msg Ping"); 
                        // respond to ping probes
                        framed.send(awc::ws::Message::Pong(Bytes::new())).await.unwrap();
                    }

                    Ok(Frame::Pong(_)) => {
                        log::info!("Client:Type msg Pong");  
                    }
 
                    Ok(Frame::Binary(bin)) => {
                        log::info!("Client:Type msg Binary: {:?}", bin);
                         
     
                         let size_byte = 4096;//65536-1 4096 16384 32768   (max=65536 2 byte/ 16bit)
                         let buff = (0..=size_byte).map(|_|78).collect::<Vec<u8>>();
     
                         if let Ok(n) = Number::try_from(bin){
                             match n.0 {
                                 0 => {
                                    framed.send( awc::ws::Message::Continuation( Item::FirstBinary( Bytes::from_iter(buff.clone())))); 
                                 },
                                 e @ 1..=8 => {
                                    framed.send( awc::ws::Message::Continuation( Item::Continue( Bytes::from_iter(buff.clone())))); 
                                 },
                                 e @ 9  => {
                                    framed.send( awc::ws::Message::Continuation( Item::Last( Bytes::from_iter(buff.clone()))));
                                 },
                                 _ => {
                                     println!("Client:send end");
                                 }
                             }
                         }
                     }

                    _ => {
                        log::info!("Other message type");
                    }
                }
            }

            Some(cmd) = cmd_rx.next() => {
                if cmd.is_empty() {
                    continue;
                }

                framed.send(awc::ws::Message::Text(cmd.into())).await.unwrap();
            }

            else => break
        }
    }
     input_thread.join().unwrap();  
}