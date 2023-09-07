use std::hash::Hash;

pub(crate) trait Game: Clone + std::fmt::Display {
    type Action: std::fmt::Debug + Hash + PartialEq + Eq + Clone;
    type Player: PartialEq + std::fmt::Debug + Clone;

    fn step(&mut self, action: Self::Action) -> anyhow::Result<f32>;

    fn get_available_moves(&self) -> Vec<Self::Action>;

    fn current_player(&self) -> Self::Player;

    fn done(&self) -> bool;

    fn check_winner(&self) -> Option<Self::Player>;
}