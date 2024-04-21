use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Result},
};
use futures_util::{StreamExt, SinkExt};
use std::env;
use serde_json::{Value};
use rand::seq::SliceRandom;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let name = &args[1];

    let url = url::Url::parse("ws://localhost:9002").unwrap();

    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    println!("sending");

    write.send(Message::Text(format!(r#"{{
        "type": "subscribe",
        "botname": "{}"
      }}"#, name).to_string()+"\n")).await.unwrap();

    println!("sent");

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        println!("{}", msg.clone().into_text().unwrap());
                        if msg.is_text() || msg.is_binary() {
                            let json: Value = serde_json::from_str(&msg.clone().into_text().unwrap()).unwrap();
                            println!("json type: {}", json["type"].as_str().unwrap());
                            match json["type"].as_str().unwrap().as_ref(){
                                "next_turn" => {
                                    let msg = format!(r#"{{
                                        "type": "move",
                                        "square": "{}"
                                      }}"#, do_turn(json["board"].as_str().unwrap()));
                                    write.send(msg.into()).await?;
                                },
                                "tick" => {
                                    let msg = format!(r#"{{
                                        "type": "tock"
                                      }}"#);
                                    write.send(msg.into()).await?;
                                },
                                &_ => todo!()
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn do_turn(board: &str) -> String {
    let index: Vec<_> = board.match_indices('_').map(|(i,_)|i).collect();
    println!("Index: {:?}", index);
    let value = index.choose(&mut rand::thread_rng()).unwrap().clone();
    value.to_string()
}