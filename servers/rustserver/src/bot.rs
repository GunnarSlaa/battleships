use std::net::SocketAddr;
use futures::channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct Bot {
    address: String,
    tx: UnboundedSender<Message>,
    name: String,
    game_id: usize,
}

impl Bot {
    pub fn new(address: SocketAddr, tx: UnboundedSender<Message>, name: String) -> Bot{
        Bot {address: address.to_string(), tx, name, game_id: 0}
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_game_id(&mut self, game_id: usize) {
        self.game_id = game_id.clone();
    }

    pub fn get_game_id(&self) -> usize {
        self.game_id.clone()
    }

    pub fn send_msg(&self, message: &str) {
        self.tx.unbounded_send(message.into()).unwrap();
    }
}