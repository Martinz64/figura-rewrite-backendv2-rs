use async_std::{prelude::*, task};
use bytes::{BufMut, BytesMut};
use tide_websockets::{Message, WebSocket};
mod routes;
mod utils;
mod state;
mod objects;

use std::{env, time::{Duration, Instant}};

use crate::state::State;

#[async_std::main]
async fn main() -> tide::Result<()> {
    femme::start(log::LevelFilter::Info)?;
    
    let mut app = tide::with_state(State::new());

    //authentication
    app.at("/api//auth/id").get(routes::login_stage1);
    app.at("/api//auth/verify").get(routes::login_stage2);

    //extra daya
    app.at("/api/version").get(routes::version);
    app.at("/api/limits").get(routes::limits);

    app.at("/api/debug").get(routes::debug_info);

    //avatar uploading
    app.at("/api/avatar").put(routes::upload_avatar);
    app.at("/api/equip").post(routes::equip_avatar);

    //user info & avatar download
    app.at("/api/:uuid/avatar").get(routes::download_avatar);
    app.at("/api/:uuid").get(routes::user_info);

    //websocet stuf
    app.at("/ws")
        .with(WebSocket::new(|_request, mut stream| async move {
            
            let mut interval = async_std::stream::interval(Duration::from_secs(2));
            
            let stream2 = stream.clone();

            task::spawn(async move {
                let mut pingtimes : i32 = 0;
                loop {
                    interval.next().await;
                    if let Err(_) = stream2.send(Message::Ping(pingtimes.to_be_bytes().to_vec())).await {
                        break;
                    }

                    let mut buffer = BytesMut::new();
                    buffer.put_u8(4);
                    
                    let msg_string = format!("Test ping {}",pingtimes);
                    buffer.put(msg_string.as_bytes());
                    buffer.put_u8(0);

                    /*if let Err(_) = stream2.send(Message::Binary(buffer.to_vec())).await {
                        break;
                    }*/
                    pingtimes+=1;
                }
            });
            
            while let Some(result) = stream.next().await {
                let message = match result {
                    Ok(message) => message,
                    Err(_) => {
                        break;
                    }
                };

                match message{
                    Message::Ping(_) => {
                        if let Err(e) = stream.send(Message::Pong(Vec::new())).await {
                            break;
                        }
                    }
                    Message::Binary(input) => {
                        if input[0] == 0 {
                            //respond to auth message
                            stream.send_bytes([0].to_vec())
                            .await?;
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
                 
            }
            Ok(())
        }))
        .get(|_| async move { Ok("this was not a websocket request") });

    app.at("/").get(|_| async move { Ok("OwO") });

    let mut port = 8080;
    for arg in env::args() {
        if arg == "p8081"{
            port = 8081;
        }
    }

    app.listen(format!("0.0.0.0:{}",port)).await?;
    Ok(())
}

