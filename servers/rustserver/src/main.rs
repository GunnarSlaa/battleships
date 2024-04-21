mod bot;
mod game;
use futures_util::{SinkExt, StreamExt};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex},};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};
use bot::Bot;
use game::Game;
use serde_json::{Value};
type Tx = UnboundedSender<Message>;
type BotVec = Arc<Mutex<Vec<SocketAddr>>>;
type BotMap = Arc<Mutex<HashMap<SocketAddr, Bot>>>;
type GameMap = Arc<Mutex<HashMap<usize, Tx>>>;

async fn accept_connection(bots: BotMap, lobby: BotVec, games: GameMap, peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(bots.clone(), lobby.clone(), games.clone(), peer, stream).await {
        println!("{} disconnected", &peer);
        bots.lock().unwrap().remove(&peer);
        lobby.lock().unwrap().retain(|x| *x != peer);
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => println!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(bots: BotMap, lobby: BotVec, games: GameMap, peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    println!("New WebSocket connection: {}", peer);
    let (mut outgoing, mut incoming) = ws_stream.split();
    let (tx, mut rx) = unbounded();
    let bot = Bot::new(peer, tx, "nameless".to_string());
    bots.lock().unwrap().insert(peer, bot.clone());

    loop {
        tokio::select! {
            msg = incoming.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            let mut json: Value = serde_json::from_str(&msg.clone().into_text().unwrap()).unwrap();
                            match json["type"].as_str().unwrap().as_ref(){
                                "subscribe" => {
                                    bots.lock().unwrap().get_mut(&peer).expect("Bot not found").set_name(&json["botname"].as_str().unwrap());
                                    lobby.lock().unwrap().push(peer);
                                    tokio::spawn(handle_match(bots.clone(), lobby.clone(), games.clone()));
                                },
                                "move" => {
                                    println!("Square: {}", &json["square"].as_str().unwrap());
                                    let game_id = bots.lock().unwrap().get(&peer).expect("Bot not found").get_game_id();
                                    let game_addr = games.lock().unwrap().get(&game_id).expect("Game not found").clone();
                                    json["player"] = peer.to_string().into();
                                    game_addr.unbounded_send(json.to_string().into()).unwrap();
                                },
                                "tock" => {},
                                &_ => todo!()
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            fw = rx.next() => {
                match fw {
                    Some(fw) => {
                        outgoing.send(fw).await?;
                    }
                    None => break,
                }
            }
        }
    }
    println!("{} disconnected", &peer);
    bots.lock().unwrap().remove(&peer);

    Ok(())
}

async fn handle_match(bots: BotMap, lobby: BotVec, games: GameMap){
    if lobby.lock().unwrap().len() >= 2{
        //TODO: Increment game_id
        let game_id = 1;
        let player1 = lobby.lock().unwrap().iter().next().unwrap().clone();
        let player1_name = bots.lock().unwrap().get(&player1).expect("Bot not found").get_name().to_owned();
        bots.lock().unwrap().get_mut(&player1).expect("Bot not found").set_game_id(game_id);
        lobby.lock().unwrap().remove(0);
        let player2 = lobby.lock().unwrap().iter().next().unwrap().clone();
        let player2_name = bots.lock().unwrap().get(&player2).expect("Bot not found").get_name().to_owned();
        bots.lock().unwrap().get_mut(&player2).expect("Bot not found").set_game_id(game_id);
        lobby.lock().unwrap().remove(0);
        let (tx, mut rx) = unbounded();

        println!("Starting game between {} and {}", player1_name, player2_name);
        let mut game = Game::new(player1,player2, bots.clone(), game_id);
        games.lock().unwrap().insert(game_id, tx);
        game.start_game();
        game.print_board();
        loop {
            tokio::select! {
                fw = rx.next() => {
                    match fw {
                        Some(fw) => {
                            let json: Value = serde_json::from_str(&fw.clone().into_text().unwrap()).unwrap();
                            game.handle_turn(&json["square"].as_str().unwrap(), &json["player"].as_str().unwrap());
                        }
                        None => break,
                    }
                }
            }
        }
    }
    else{
        println!("Not enough bots to start a game");
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    let bots = BotMap::new(Mutex::new(HashMap::new()));
    let lobby = BotVec::new(Mutex::new(Vec::new()));
    let games = GameMap::new(Mutex::new(HashMap::new()));
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        tokio::spawn(accept_connection(bots.clone(), lobby.clone(), games.clone(), peer, stream));
    }
}