use std::collections::HashMap;
use rand::seq::SliceRandom;

use crate::tic_tac_toe::TicTacToe;
use anyhow::Result;

pub(crate) struct Mcts {
    exploration_constant: f32,
    simulations: usize,
}

impl Mcts {
    pub(crate) fn new(exploration_constant: f32, simulations: usize) -> Self {
        Self {
            exploration_constant,
            simulations,
        }
    }

    pub(crate) fn select_move(&self, game: &TicTacToe) -> Result<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let mut wins = HashMap::new();
        let mut plays = HashMap::new();
        let available_moves = game.get_available_moves();

        for _ in 0..self.simulations {
            let chosen_move = *available_moves.choose(&mut rng).unwrap();
            self.run_simulation(chosen_move, &mut wins, &mut plays, game)?;
        }

        let mut best_move = None;
        let mut best_score = -1.0;

        for &mv in &available_moves {
            let win = *wins.get(&mv).unwrap_or(&0.0);
            let play = *plays.get(&mv).unwrap_or(&1.0);
            let score = win / play + self.exploration_constant * (plays.len() as f32).sqrt() / (1.0 + play);

            if score > best_score {
                best_move = Some(mv);
                best_score = score;
            }
        }

        best_move.ok_or(anyhow::anyhow!("No available moves"))
    }

    fn run_simulation(&self, mv: (usize, usize), wins: &mut HashMap<(usize, usize), f32>, plays: &mut HashMap<(usize, usize), f32>, game: &TicTacToe) -> Result<()> {
        let mut game = game.clone();
        game.step(mv.0, mv.1)?;

        let mut player = game.current_player;
        let mut reward = 0.0;

        loop {
            match game.check_winner() {
                Some(winner) => {
                    if winner == player {
                        reward = 1.0;
                    } else {
                        reward = -1.0;
                    }
                    break;
                },
                None => {
                    let moves = game.get_available_moves();
                    if moves.is_empty() {
                        break;
                    }
                    let mv = *moves.choose(&mut rand::thread_rng()).unwrap();
                    game.step(mv.0, mv.1)?;
                    player = game.current_player;
                }
            }
        }

        *wins.entry(mv).or_insert(0.0) += reward;
        *plays.entry(mv).or_insert(0.0) += 1.0;

        Ok(())
    }
}
