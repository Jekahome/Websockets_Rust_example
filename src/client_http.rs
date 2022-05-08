use awc::Client;
use actix_web::http::header;
use awc::ClientRequest;
use awc::ClientBuilder;
use std::time::{Duration, Instant};


//---------------------------------------------------------------------------------------------------------
// Client GET

async fn client_get_easy() {

    /*
    use actix_http::client::Connector;
    let connector = Connector::new()
        .timeout(Duration::from_secs(5))
        .finish();
    */   
    /*   
        let mut client = Client::default();
    
        // Create request builder and send request
        let response = client.get("http://192.168.0.104:4011/input-request/Jeka")
        .header("User-Agent", "actix-web/3.0")
        .send()     // <- Send request
        .await;     // <- Wait for response
    
        println!("Response: {:#?}", response);
    */

 // or ClientBuilder -----------------------------

    let client:Client = awc::ClientBuilder::new()
        .timeout(Duration::new(5, 0))
        .header("User-Agent", "actix-web/3.0")
        .initial_connection_window_size(65535)
        .finish();

    let request:ClientRequest = client.get("http://192.168.0.104:4011/input-request/Jeka");

    let send_client:awc::SendClientRequest = request.send();
    let response = send_client.await; 
    //println!("Response: {:#?}", response);

    response.and_then(|response| { 
        println!("Response: {:#?}", response);
        Ok(())
    });
 }
 //---------------------------------------------------------------------------------------------------------
 // Client POST

 async fn client_post(buff:Vec<u8>,size_byte:usize){

    let client:Client = ClientBuilder::new()
        //.timeout(Duration::new(5, 0))
        //.header("User-Agent", "actix-web/3.0")
       // .initial_connection_window_size(65535)
        .finish();
    
    let request:ClientRequest = client.post("http://192.168.0.104:4011/sound")
                                      .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN,"*"))
                                      //.header(header::CONTENT_TYPE, "multipart/form-data")
                                      .insert_header((header::CONTENT_TYPE, "application/octet-stream"))
                                      //.header(header::CONTENT_DISPOSITION, "attachment; filename=\"test.txt\"")
                                      .insert_header((header::CONTENT_LENGTH, format!("{}",size_byte)));
/*
    let body = stream::once( 
        async move {
         Ok::<_, Error>( Bytes::from_iter(buff) )
        }
    );

    let send_client = request.send_stream(Box::pin(body));
*/ 
    let send_client:awc::SendClientRequest = request.send_body(buff);
    
    let response = send_client.await; 
    response.and_then(|_response| { 
       // println!("Response: {:#?}", response);
        Ok(())
    });
 }

//---------------------------------------------------------------------------------------------------------
// Websockets Client





//---------------------------------------------------------------------------------------------------------

#[actix_web::main]
async fn main() {
   // client_get_easy().await;
   let now = Instant::now();
   let size_byte = 4096;// 16384 32768  
   let buff = (0..=size_byte).map(|_|78).collect::<Vec<u8>>();

   for i in (0..10000){// 10=41Kb 100=400Kb 1000=4Mb 10000=41Mb
        client_post(buff.clone(),size_byte).await;
   }
    println!("sec={}", now.elapsed().as_secs());// 9
}

