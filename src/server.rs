use actix_web::{
    middleware, 
    web, 
    App, 
    Error, 
    HttpRequest, 
    HttpResponse, 
    HttpServer, 
    Result
};
use actix_http::header::map::HeaderMap;
// макросы роутинга
use actix_web::{get, post};
use std::io::Write;
use futures::{TryStreamExt};
use std::fs::OpenOptions;


// HTTP example

// curl -F 'file=@/home/jeka/test.txt' -H  'Content-Type: multipart/form-data' -H 'Content-Length: 6' -H 'Content-Disposition: attachment; filename="test.txt"'  http://192.168.0.104:4011/test
#[post("sound")]
async fn save_file(mut payload: web::Payload,req: HttpRequest) -> Result<HttpResponse, Error>{
    let map:&HeaderMap = req.headers();
    let content_len:usize = map.get("Content-Length").unwrap().to_str().unwrap().parse::<usize>().unwrap();
    println!("HTTP Content-Length={}",content_len);

    let mut body = web::BytesMut::new();
    /*
        let mut count = 0;
        let mut data;
        
        while content_len > body.len() {
            count+=1;
            data = payload.next();
            println!("-");
            
            if let Some(field) = data.await{
                println!("await size_hint={:?}",payload.size_hint());
                let chunk = field?;
                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(actix_web::error::ErrorBadRequest("overflow"));
                }
                body.extend_from_slice(&chunk);
            }else{
                return Err(actix_web::error::ErrorBadRequest("Error await"));
            }
            println!("+ count={}",count);
            println!("size_hint={:?}",payload.size_hint());
        }
        Ok(HttpResponse::Ok().finish())
    */

   
    while let Ok(Some(chunk)) = payload.try_next().await {
        //println!(" chank_count={}",chunk.len());
        body.extend_from_slice(&chunk);
    }

    let filepath = "./sound/increment.raw".to_string();
    let mut f = OpenOptions::new().append(true).create(true).open(filepath).unwrap();
    f = web::block(move || f.write_all(&body).map(|_| f).unwrap()).await?;
    Ok(HttpResponse::Ok().finish())
/*
    match std::str::from_utf8(&body){
        Ok(res) => {
            // println!("Res: {:?}", res);
             Ok(HttpResponse::Ok().finish())
        },
        Err(err) =>{
            // println!("Err: {:?}", err);
            Err(actix_web::error::ErrorBadRequest("UTF-8 invalid"))
        }
    }  */
    
}

// ==========================================================================================
// Websockets Server example
/*
ctx: ws::WebsocketContext<Self> 

ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"))
ctx.binary(Bytes::from(0_usize.to_be_bytes().to_vec()))

let text:String = String::from("");
ctx.text(text)
ctx.notify(CommandRunner(text))

*/

use actix::prelude::*;
use actix::{Context,Actor, StreamHandler};
use actix_web_actors::ws;
use actix::AsyncContext;
use actix_http::ws::Item;
use std::io::BufReader;
use bytes::Bytes;
use std::time::{Duration, Instant};

/// Как часто отправляются эхо-запросы сердцебиения
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(500);
/// TIMEOUT проверка присутствия клиента на линии
const CLIENT_TIMEOUT: Duration = Duration::from_secs(500);

/// Define HTTP actor
struct ActorServer {
    /// Клиент должен отправлять пинг не реже одного раза в 10 секунд (CLIENT_TIMEOUT),
     /// иначе разрываем соединение.
     duration_heartbeat: Instant,
}

impl ActorServer {
    fn new() -> Self {
        Self { duration_heartbeat: Instant::now() }
    }

    /// вспомогательный метод, который каждую секунду отправляет пинг клиенту.
    /// также этот метод проверяет сердцебиение от клиента
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        // Context ctx сдесь это Struct actix_web_actors::ws::WebsocketContext<ActorServer>
        // от Trait actix::Actor досталось:
        //    - create, start, start_default, start_in_arbiter, started, stopped, stopping

        // Struct actix_web_actors::ws::WebsocketContext имеет свои собственные методы: 
        //    - binary, close, create, create_with_addr, handle, ping, pong, set_mailbox_capacity, text, with_codec, with_factory, write_raw
        
        // actix_web_actors::ws::WebsocketContext реализует трейт:
        // 1. actix::prelude::ActorContext добавляя методы: 
        //    - stop, terminate, state

        // 2. actix::prelude::AsyncContext
        //    - address, cancel_future, spawn, wait, waiting, (далее реализованные самим трейтом) add_message_stream, add_stream, notify, notify_later, run_interval, run_later

        // 3.   actix::dev::AsyncContextParts
        //    - parts

        // 4. actix::dev::ToEnvelope
        //    - pack

        /*ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // проверка клиента сердцебиения
            if Instant::now().duration_since(act.duration_heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // stop actor
                ctx.stop();
                // не пытайтесь отправить пинг
                return;
            }

            ctx.ping(b"msg server to client");
        });*/
        
    }
}

impl Actor for ActorServer {
    type Context = ws::WebsocketContext<Self>; // Struct actix_web_actors::ws::WebsocketContext

    /// Метод вызывается при запуске актера. Здесь мы запускаем процесс сердцебиения.
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Server: Client started");  
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("Server: Client disconnected");   
    }
}

static mut count_payload:usize = 0;
static mut bytes_len_payload:usize = 0;
 
/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ActorServer {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        // println!("WS: {:?}", msg);
       
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("Server:Type msg Ping: {:?}", msg);
                self.duration_heartbeat = Instant::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.duration_heartbeat = Instant::now();
            },
            Ok(ws::Message::Text(text)) => {
                println!("Server:Type msg Text: {:?}", text);
                self.duration_heartbeat = Instant::now();
               
                // Go load
                if text == "ready send".to_string() {
                    unsafe{ bytes_len_payload=0;}
                    ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"));  
                }
            },
            Ok(ws::Message::Binary(bin)) => { 
                unsafe{count_payload+=1;}

                //println!("Server:Type msg Binary: {:?}", bin);
                self.duration_heartbeat = Instant::now();
                
                unsafe{
                    bytes_len_payload+=bin.len();
                    println!("Binary count={} bin.len={} bytes_len={}",count_payload,bin.len(),bytes_len_payload);
                }

                let mut f = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open("./sound/increment.raw".to_string())
                                .unwrap();

                f.write_all(&web::BytesMut::from_iter(bin));

                ctx.binary(Bytes::from(0_usize.to_be_bytes().to_vec()));
            },
            Ok(ws::Message::Continuation(item)) => { unsafe{count_payload+=1;}
              // frame size 65536 bytes => count=132 bin.len=65536 bytes_len=8650752
              // frame size 4096 bytes =>  count=2537 bin.len=4097 bytes_len=10394089

                //println!("Server:Type msg Binary: {:?}", bin);
                self.duration_heartbeat = Instant::now();

                let mut f = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open("./sound/increment.raw".to_string())
                                .unwrap();

                match item {
                    Item::FirstBinary(bin) => {
                        
                        unsafe{
                            bytes_len_payload+=bin.len();
                            println!("Continuation FirstBinary count={} bin.len={} bytes_len={}",count_payload,bin.len(),bytes_len_payload);
                        }
                        //println!("{:?}",bin.clone().iter().take(100));
                        f.write_all(&web::BytesMut::from_iter(bin)); 
                        
                        ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"));  
                    },
                    Item::Continue(bin) => {
                        unsafe{
                            bytes_len_payload+=bin.len();
                            println!("Continuation Continue count={} bin.len={} bytes_len={}",count_payload,bin.len(),bytes_len_payload);
                        }
                        f.write_all(&web::BytesMut::from_iter(bin));

                        ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"));
                    },
                    Item::Last(bin) => {
                        unsafe{
                            bytes_len_payload+=bin.len();
                            println!("Continuation Last count={} bin.len={} bytes_len={}",count_payload,bin.len(),bytes_len_payload);
                            ctx.text( format!("{}",count_payload) )
                        }
                        f.write_all(&web::BytesMut::from_iter(bin));
                        
                        ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"));
                    },
                    Item::FirstText(text) => {
                        unsafe{println!("Continuation FirstText count={} bin.len={} bytes_len={}",count_payload,text.len(),bytes_len_payload);}
                         // Go load
                        if text == "ready send".to_string() {
                            println!("SERVER send to Client 'Go load'");
                            unsafe{ bytes_len_payload=0;}
                            //ctx.binary(Bytes::from_static(b"\0\0\0\0\0\0\0\0"))
                            ctx.binary(Bytes::from(1_usize.to_be_bytes().to_vec() ) )
                        }
                    },
                    _ => {
                        eprintln!("WTF {:?}",item);
                        ctx.pong(&[0])
                    }
                   }   
            },

            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),// (),
        }
    }
}

#[get("ws/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    // Когда устанавлмвается коннект с клиентом
    let resp = ws::start(ActorServer::new(), &req, stream);
    println!("Response ws_index:{:?}", resp);
    /*
    Response ws_index:Ok(
        Response HTTP/1.1 101 Switching Protocols
        headers:
            "upgrade": "websocket"
            "sec-websocket-accept": "1P+uWczvElWXFabaqh2A4EImAW0="
            "transfer-encoding": "chunked"
        body: Stream
        )
    */
    resp
}
// testing requires specific headers:
// Upgrade: websocket
// Connection: Upgrade
// Sec-WebSocket-Key: SOME_KEY
// Sec-WebSocket-Version: 13

// ==========================================================================================

// 2. Настроить websockets https://github.com/actix/actix-website/tree/master/examples/websockets
// https://learn.javascript.ru/websockets
// https://github.com/actix/examples/blob/master/websockets/chat/src/main.rs
// https://github.com/antholeole/actix-sockets/blob/main/src/ws.rs

//************************************************************************************************

/*
   // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    Actix-web поддерживает веб-сокеты с crate actix-web-actors. 
    Можно преобразовать запрос Payload в поток ws::Message с помощью web::Payload, 
    а затем использовать комбинаторы потоков для обработки фактических сообщений, 
    но проще обрабатывать обмен данными через веб-сокеты с помощью http actor.
*/


// lsof -n -i4TCP:4011 | grep LISTEN | tr -s ' ' | cut -f 2 -d ' ' | xargs kill -9
// ngrok http 4000 --authtoken 1okjXfwKGugN4HzH0fsrBFRpynN_7E3F3A4SnMUe**** -host-header=rewrite
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // cargo run --bin websocket-server

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    std::fs::create_dir_all("./sound").unwrap();
  
    HttpServer::new(move || {
        App::new()
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Access-Control-Allow-Origin", "*"))
            // enable logger
            /*.wrap(middleware::Logger::new("---------------------------\n"))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("Content-Disposition %{Content-Disposition}i"))
            .wrap(middleware::Logger::new("Content-Type %{Content-Type}i"))
            .wrap(middleware::Logger::new("Content-Length %{Content-Length}i"))
            .data(web::JsonConfig::default().limit(4096))*/
            .service(save_file)
            .service(ws_index)
            //.service(web::resource("/ws/").route(web::get().to(ws_index)))
           
    })
    //.max_connections(100)
    .workers(6)  
    //.keep_alive(75)
    //.client_timeout(5)
    //.client_shutdown(5)
    //.shutdown_timeout(5)
    .bind("0.0.0.0:4011")?
    .run()
    .await
}


/*
Различия между TCP сокетами и веб-сокетами
https://coderoad.ru/16945345/%D0%A0%D0%B0%D0%B7%D0%BB%D0%B8%D1%87%D0%B8%D1%8F-%D0%BC%D0%B5%D0%B6%D0%B4%D1%83-TCP-%D1%81%D0%BE%D0%BA%D0%B5%D1%82%D0%B0%D0%BC%D0%B8-%D0%B8-%D0%B2%D0%B5%D0%B1-%D1%81%D0%BE%D0%BA%D0%B5%D1%82%D0%B0%D0%BC%D0%B8-%D0%B5%D1%89%D0%B5-%D1%80%D0%B0%D0%B7

При отправке байтов из буфера с обычным сокетом TCP функция send возвращает количество байтов буфера, которые были отправлены. 
Если это неблокирующий сокет или неблокирующая отправка, то количество отправленных байтов может быть меньше размера буфера. 
Если это блокирующий сокет или блокирующая отправка, то возвращаемый номер будет соответствовать размеру буфера, но вызов может быть заблокирован. 
В случае WebSockets данные, передаваемые методу send, всегда отправляются либо целиком "message", либо вообще не отправляются. 
Кроме того, реализации browser WebSocket не блокируют вызов send.

Но есть более важные различия на принимающей стороне вещей. 
Когда получатель выполняет recv (или read ) на сокете TCP, нет никакой гарантии, что количество возвращаемых байтов соответствует одной отправке (или записи) на стороне отправителя. 
Это может быть то же самое, это может быть меньше (или ноль), и это может быть даже больше (в этом случае принимаются байты от нескольких отправок/записей). 
В случае WebSockets получатель сообщения управляется событиями (обычно вы регистрируете процедуру обработки сообщений), и данные в событии всегда представляют собой все сообщение, отправленное другой стороной.

Обратите внимание, что вы можете осуществлять связь на основе сообщений с помощью сокетов TCP, но вам нужен дополнительный слой/инкапсуляция, которая добавляет данные о границах кадрирования/сообщений в сообщения, чтобы исходные сообщения можно было повторно собрать из фрагментов. 
Фактически, WebSockets построен на обычных сокетах TCP и использует заголовки кадров, которые содержат размер каждого кадра и указывают, какие кадры являются частью сообщения. 
WebSocket API повторно собирает фрагменты данных TCP в фреймы, которые собираются в сообщения, прежде чем вызывать обработчик событий сообщения один раз для каждого сообщения.
*/