mod tic_tac_toe;
mod mcts;

use std::io;

use mcts::MCTS;
use tic_tac_toe::TicTacToe;

use crate::tic_tac_toe::Player;

fn main() -> anyhow::Result<()> {
    let mut game = TicTacToe::new();
    let mcts = MCTS {};

    loop {
        println!("{}", game);

        if let Some(winner) = game.check_winner() {
            println!("Player {:?} wins!", winner);
            break;
        }

        let (row, col) = match game.current_player {
            Player::X => {
                // Ask the user for their move
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let mut parts = input.trim().split_whitespace();
                let row: usize = parts.next().unwrap().parse()?;
                let col: usize = parts.next().unwrap().parse()?;
                (row, col)
            },
            Player::O => {
                // Use MCTS to select the best move
                mcts.select_move(&game)?
            },
        };

        game.step(row, col)?;
    }

    Ok(())
}