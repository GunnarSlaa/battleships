use crate::bot::Bot;
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex},};
// use rand::Rng;
type BotMap = Arc<Mutex<HashMap<SocketAddr, Bot>>>;

#[derive(Clone)]
pub struct Game {
    player1: SocketAddr,
    player2: SocketAddr,
    bots: BotMap,
    board: [char; 9],
    current_turn: bool,
    game_id: usize,
}

impl Game{
    pub fn new(player1: SocketAddr, player2: SocketAddr, bots: BotMap, game_id: usize) -> Game {
        //TODO: Randomize starting player
        Game {player1, player2, bots, board: ['_'; 9], current_turn: true, game_id}
    }

    pub fn print_board(&self) {
        let board_string = self.board.into_iter().collect::<String>();
        println!("{}", &board_string[0..3]);
        println!("{}", &board_string[3..6]);
        println!("{}", &board_string[6..9]);
    }

    pub fn start_game(&self) {
        self.next_turn();
    }

    fn next_turn(&self){
        let msg = format!(r#"{{
            "type": "next_turn",
            "board": "{}"
          }}"#, self.board.into_iter().collect::<String>());
        if self.current_turn{
            self.bots.lock().unwrap().get(&self.player1).expect("Bot not found").send_msg(&msg);
        }
        else {
            self.bots.lock().unwrap().get(&self.player2).expect("Bot not found").send_msg(&msg);
        }
    }

    fn check_win(&self) -> Option<usize>{
        let lines: [[usize; 3]; 8] = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
        for line in lines{
            if line.iter().all(|x| self.board[*x] == 'X'){
                return Some(0)
            }
            else if line.iter().all(|x| self.board[*x] == 'O'){
                return Some(1)
            }
        }
        if !self.board.iter().any(|x| x == &'_'){
            return Some(2)
        }
        return None
    }

    pub fn handle_turn(&mut self, square: &str, bot: &str){
        if bot == self.player1.to_string() {
            self.board[square.parse::<usize>().unwrap()] = 'X';
            self.current_turn = false;
        }
        else if bot == self.player2.to_string() {
            self.board[square.parse::<usize>().unwrap()] = 'O';
            self.current_turn = true;
        }
        else {
            println!("BotAddr: {}", bot);
            println!("Player1: {}", self.player1.to_string());
            println!("Player2: {}", self.player2.to_string());
        }
        self.print_board();
        let state = self.check_win();
        match state {
            Some(state) => {
                match state{
                    0 => println!("X wins"),
                    1 => println!("O wins"),
                    2 => println!("Draw"),
                    _ => todo!()
                }
            }
            None => {
                self.next_turn();
            }
        }
    }
}