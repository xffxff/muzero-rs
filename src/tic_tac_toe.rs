use anyhow::bail;
use std::fmt;

use crate::game::Game;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Player {
    X,
    O,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Spot {
    Empty,
    Filled(Player),
}

#[derive(Debug, Clone)]
pub(crate) struct TicTacToe {
    spots: [[Spot; 3]; 3],
    pub(crate) current_player: Player,
}

impl Game for TicTacToe {
    type Action = (usize, usize);

    type Player = Player;

    fn step(&mut self, action: Self::Action) -> anyhow::Result<f32> {
        let (row, col) = action;
        match self.spots[row][col] {
            Spot::Empty => {
                self.spots[row][col] = Spot::Filled(self.current_player);
                self.current_player = match self.current_player {
                    Player::X => Player::O,
                    Player::O => Player::X,
                };

                let terminated = self.check_winner().is_some(); // Check if the game has a winner
                let reward = if terminated { 1.0 } else { 0.0 }; // Implement according to your needs
                Ok(reward)
            }
            Spot::Filled(_) => bail!("Spot is already filled"),
        }
    }

    fn get_available_moves(&self) -> Vec<Self::Action> {
        let mut available_moves = Vec::new();
        for (i, row) in self.spots.iter().enumerate() {
            for (j, &spot) in row.iter().enumerate() {
                if spot == Spot::Empty {
                    available_moves.push((i, j));
                }
            }
        }
        available_moves
    }

    fn current_player(&self) -> Self::Player {
        self.current_player
    }

    fn done(&self) -> bool {
        self.check_winner().is_some() || self.get_available_moves().is_empty()
    }

    fn check_winner(&self) -> Option<Self::Player> {
        // Check rows
        for row in 0..3 {
            if let Spot::Filled(player) = self.spots[row][0] {
                if self.spots[row][1] == Spot::Filled(player)
                    && self.spots[row][2] == Spot::Filled(player)
                {
                    return Some(player);
                }
            }
        }

        // Check columns
        for col in 0..3 {
            if let Spot::Filled(player) = self.spots[0][col] {
                if self.spots[1][col] == Spot::Filled(player)
                    && self.spots[2][col] == Spot::Filled(player)
                {
                    return Some(player);
                }
            }
        }

        // Check diagonals
        if let Spot::Filled(player) = self.spots[0][0] {
            if self.spots[1][1] == Spot::Filled(player) && self.spots[2][2] == Spot::Filled(player)
            {
                return Some(player);
            }
        }
        if let Spot::Filled(player) = self.spots[0][2] {
            if self.spots[1][1] == Spot::Filled(player) && self.spots[2][0] == Spot::Filled(player)
            {
                return Some(player);
            }
        }

        None
    }
}

impl TicTacToe {
    pub(crate) fn new() -> Self {
        Self {
            spots: [[Spot::Empty; 3]; 3],
            current_player: Player::X,
        }
    }
}

impl fmt::Display for TicTacToe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.spots {
            for spot in row {
                let symbol = match spot {
                    Spot::Empty => ".",
                    Spot::Filled(Player::X) => "X",
                    Spot::Filled(Player::O) => "O",
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let game = TicTacToe::new();
        assert_eq!(game.spots, [[Spot::Empty; 3]; 3]);
        assert_eq!(game.current_player, Player::X);
    }

    #[test]
    fn test_step() {
        let mut game = TicTacToe::new();
        assert!(game.step((0, 0)).is_ok());
        assert_eq!(game.spots[0][0], Spot::Filled(Player::X));
        assert_eq!(game.current_player, Player::O);

        assert!(game.step((0, 0)).is_err());
        assert_eq!(game.spots[0][0], Spot::Filled(Player::X));
        assert_eq!(game.current_player, Player::O);

        assert!(game.step((0, 1)).is_ok());
        assert_eq!(game.spots[0][1], Spot::Filled(Player::O));
        assert_eq!(game.current_player, Player::X);
    }

    #[test]
    fn test_check_winner() {
        let mut game = TicTacToe::new();
        assert_eq!(game.check_winner(), None);

        game.spots = [
            [Spot::Filled(Player::X), Spot::Empty, Spot::Empty],
            [Spot::Empty, Spot::Filled(Player::X), Spot::Empty],
            [Spot::Empty, Spot::Empty, Spot::Filled(Player::X)],
        ];
        assert_eq!(game.check_winner(), Some(Player::X));

        game.spots = [
            [Spot::Filled(Player::O), Spot::Empty, Spot::Empty],
            [Spot::Empty, Spot::Filled(Player::O), Spot::Empty],
            [Spot::Empty, Spot::Empty, Spot::Filled(Player::O)],
        ];
        assert_eq!(game.check_winner(), Some(Player::O));

        game.spots = [
            [
                Spot::Filled(Player::X),
                Spot::Filled(Player::O),
                Spot::Empty,
            ],
            [
                Spot::Empty,
                Spot::Filled(Player::X),
                Spot::Filled(Player::O),
            ],
            [Spot::Filled(Player::O), Spot::Empty, Spot::Empty],
        ];
        assert_eq!(game.check_winner(), None);
    }
}
