use std::fmt;
use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Player {
    X,
    O,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Spot {
    Empty,
    Filled(Player),
}

#[derive(Debug)]
struct TicTacToe {
    spots: [[Spot; 3]; 3],
    current_player: Player,
}

impl TicTacToe {
    pub fn new() -> Self {
        Self {
            spots: [[Spot::Empty; 3]; 3],
            current_player: Player::X,
        }
    }

    // Take a step with a given action
    pub fn step(&mut self, row: usize, col: usize) -> Result<f32> {
        match self.spots[row][col] {
            Spot::Empty => {
                self.spots[row][col] = Spot::Filled(self.current_player.clone());
                self.current_player = match self.current_player {
                    Player::X => Player::O,
                    Player::O => Player::X,
                };

                let terminated = self.check_winner().is_some(); // Check if the game has a winner
                let reward = if terminated { 1.0 } else { 0.0 }; // Implement according to your needs
                Ok(reward)
            },
            Spot::Filled(_) => bail!("Spot is already filled"),
        }
    }

    // Check if there's a winner
    pub fn check_winner(&self) -> Option<Player> {
        // Check rows
        for row in 0..3 {
            if let Spot::Filled(player) = self.spots[row][0] {
                if self.spots[row][1] == Spot::Filled(player) && self.spots[row][2] == Spot::Filled(player) {
                    return Some(player.clone());
                }
            }
        }

        // Check columns
        for col in 0..3 {
            if let Spot::Filled(player) = self.spots[0][col] {
                if self.spots[1][col] == Spot::Filled(player) && self.spots[2][col] == Spot::Filled(player) {
                    return Some(player.clone());
                }
            }
        }

        // Check diagonals
        if let Spot::Filled(player) = self.spots[0][0] {
            if self.spots[1][1] == Spot::Filled(player) && self.spots[2][2] == Spot::Filled(player) {
                return Some(player.clone());
            }
        }
        if let Spot::Filled(player) = self.spots[0][2] {
            if self.spots[1][1] == Spot::Filled(player) && self.spots[2][0] == Spot::Filled(player) {
                return Some(player.clone());
            }
        }

        None
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
        assert!(game.step(0, 0).is_ok());
        assert_eq!(game.spots[0][0], Spot::Filled(Player::X));
        assert_eq!(game.current_player, Player::O);

        assert!(game.step(0, 0).is_err());
        assert_eq!(game.spots[0][0], Spot::Filled(Player::X));
        assert_eq!(game.current_player, Player::O);

        assert!(game.step(0, 1).is_ok());
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
            [Spot::Filled(Player::X), Spot::Filled(Player::O), Spot::Empty],
            [Spot::Empty, Spot::Filled(Player::X), Spot::Filled(Player::O)],
            [Spot::Filled(Player::O), Spot::Empty, Spot::Empty],
        ];
        assert_eq!(game.check_winner(), None);
    }
}