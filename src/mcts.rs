use std::{collections::HashMap, hash::Hash, sync::atomic::AtomicUsize};

use log::debug;
use rand::seq::IteratorRandom;

use crate::game::Game;

pub(crate) struct Mcts<T: Game> {
    _phantom: std::marker::PhantomData<T>,
    num_simulations: usize,
}

static NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct NodeId(usize);

impl NodeId {
    fn new() -> Self {
        NodeId(NODE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

struct Node<T: Game> {
    visits: usize,
    wins: i32,
    to_play: T::Player,
    parent: Option<NodeId>,
    children: HashMap<T::Action, NodeId>,
    unvisited_actions: Vec<T::Action>,
    done: bool,
}

impl<T: Game> Node<T> {
    fn new(db: &mut NodeMap<T>, game: &T, parent: Option<NodeId>) -> NodeId {
        let available_moves = game.get_available_moves();
        let node = Node {
            visits: 0,
            wins: 0,
            to_play: game.current_player(),
            parent,
            children: HashMap::new(),
            unvisited_actions: available_moves,
            done: game.done(),
        };
        let node_id = NodeId::new();
        db.insert(node_id, node);
        node_id
    }
}

type NodeMap<T> = HashMap<NodeId, Node<T>>;

impl<T: Game> Mcts<T> {
    pub(crate) fn new(num_simulations: usize) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            num_simulations,
        }
    }

    pub(crate) fn search(&self, game: &T) -> T::Action {
        let mut db = NodeMap::new();
        let root = Node::new(&mut db, game, None);

        for _ in 0..self.num_simulations {
            let (path, leaf) = self.selection(&db, root);
            let mut game = game.clone();
            self.apply_actions(&mut game, path);
            let expanded_node = self.expansion(&mut db, leaf, &mut game);
            let winner = self.simulation(&mut game);
            self.backpropagation(&mut db, expanded_node, winner);
        }
        self.print_tree(&db, &root, 0);
        self.best_action(&db, root)
    }

    fn print_tree(&self, db: &NodeMap<T>, root: &NodeId, level: usize) {
        let node = db.get(root).unwrap();
        let indent = " ".repeat(level * 2);
        if level == 0 {
            debug!(
                "{}{:?} {:?} {:?} {:?}",
                indent, node.to_play, node.visits, node.wins, node.done
            );
        }
        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            debug!(
                "{}{:?} {:?} {:?} {:?}",
                indent, action, child.to_play, child.wins as f32/child.visits as f32, child.done
            );
            self.print_tree(db, child_id, level + 1);
        }
    }

    fn selection(&self, db: &NodeMap<T>, root_id: NodeId) -> (Vec<T::Action>, NodeId) {
        // Start from root R and select successive child nodes until a leaf node L is reached. 
        // The root is the current game state and a leaf is any node that has a potential child from which no simulation (playout) has yet been initiated.
        let mut node_id = root_id;
        let mut path = vec![];
        loop {
            let node = db.get(&node_id).unwrap();
            if node.done {
                break;
            }
            if node.unvisited_actions.is_empty() {
                let (action, child_id) = self.best_child(db, node_id);
                path.push(action);
                node_id = child_id;
            } else {
                break;
            }
        }
        (path, node_id)
    }

    fn best_child(&self, db: &NodeMap<T>, node_id: NodeId) -> (T::Action, NodeId) {
        // select the child node with the highest UCT value.
        let node = db.get(&node_id).unwrap();
        let mut best_action = None;
        let mut best_node_id = None;
        let mut best_value = 0.0;
        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            let win_rate_for_opponent = child.wins as f32 / child.visits as f32;
            let win_rate = 1. - win_rate_for_opponent;
            let value = win_rate + (2. * (node.visits as f32).ln() / child.visits as f32).sqrt();
            if best_action.is_none() || value > best_value {
                best_action = Some(action);
                best_node_id = Some(child_id);
                best_value = value;
            }
        }
        (best_action.unwrap().clone(), best_node_id.unwrap().clone())
    }

    fn apply_actions(&self, game: &mut T, actions: Vec<T::Action>) {
        for action in actions {
            game.step(action).unwrap();
        }
    }

    fn expansion(&self, db: &mut NodeMap<T>, node_id: NodeId, game: &mut T) -> NodeId {
        // Unless L ends the game decisively (e.g. win/loss/draw) for either player,
        // create a new child node N of L and move to it.

        let node = db.get(&node_id).unwrap();
        if node.done {
            return node_id;
        }

        let action = {
            let node = db.get_mut(&node_id).unwrap();
            // if !node.done, then node.unvisited_actions should not be empty
            node.unvisited_actions.pop().unwrap()
        };

        game.step(action.clone()).unwrap();
        let new_node_id = Node::new(db, game, Some(node_id));
        let node = db.get_mut(&node_id).unwrap();
        node.children.insert(action, new_node_id);
        new_node_id
    }

    fn simulation(&self, game: &mut T) -> Option<T::Player> {
        // Play a random playout from node N. This is typically done by selecting uniform random moves until the game is finished.
        loop {
            if let Some(winner) = game.check_winner() {
                return Some(winner);
            }
            let available_moves = game.get_available_moves();
            if available_moves.is_empty() {
                return None;
            }
            let action = available_moves
                .iter()
                .choose(&mut rand::thread_rng())
                .unwrap();
            game.step(action.clone()).unwrap();
        }
    }

    fn backpropagation(&self, db: &mut NodeMap<T>, node_id: NodeId, winner: Option<T::Player>) {
        // Update the current move sequence with the simulation result. 
        // Backpropagate this result up the tree. This updates the win and visit count of each node.

        let mut node_id = node_id;
        loop {
            let node = db.get_mut(&node_id).unwrap();
            node.visits += 1;
            if let Some(winner) = &winner {
                if &node.to_play == winner {
                    node.wins += 1;
                } else {
                    node.wins -= 1;
                }
            }
            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    fn best_action(&self, db: &NodeMap<T>, node_id: NodeId) -> T::Action {
        let node = db.get(&node_id).unwrap();
        let mut best_action = None;
        let mut best_value = 0.0;
        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            let win_rate_for_opponent = child.wins as f32 / child.visits as f32;
            let win_rate = 1. - win_rate_for_opponent;
            if best_action.is_none() || win_rate > best_value {
                best_action = Some(action);
                best_value = win_rate;
            }
        }
        best_action.unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tic_tac_toe::TicTacToe;

    #[test]
    fn test_mcts() {
        let game = TicTacToe::new();
        let mcts = Mcts::<TicTacToe>::new(100);
        let action = mcts.search(&game);
        assert!(action == (1, 1))
    }
}