mod game;
mod mcts;
mod tic_tac_toe;

use std::io;

use mcts::Mcts;
use tic_tac_toe::TicTacToe;

use crate::game::Game;
use crate::tic_tac_toe::Player;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut game = TicTacToe::new();
    let mcts = Mcts::<TicTacToe>::new(100);

    loop {
        println!("{}", game);

        if game.done() {
            if let Some(winner) = game.check_winner() {
                println!("Player {:?} wins!", winner);
                break;
            } else {
                println!("Draw!");
                break;
            }
        }

        let (row, col) = match game.current_player {
            Player::X => {
                // Ask the user for their move
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let mut parts = input.split_whitespace();
                let row: usize = parts.next().unwrap().parse()?;
                let col: usize = parts.next().unwrap().parse()?;
                (row, col)
            }
            Player::O => {
                // Use MCTS to select the best move
                mcts.search(&game)
            }
        };

        game.step((row, col))?;
    }

    Ok(())
}
