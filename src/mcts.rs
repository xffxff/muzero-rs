use std::{sync::atomic::AtomicUsize, collections::HashMap, hash::Hash};

use log::debug;
use rand::seq::IteratorRandom;

use crate::tic_tac_toe::{TicTacToe, Player};

pub(crate) struct MCTS {

}

static NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct NodeId(usize);

impl NodeId {
    fn new() -> Self {
        NodeId(NODE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

struct Node {
    visits: usize,
    wins: usize,
    to_play: Player,
    parent: Option<NodeId>,
    children: HashMap<(usize, usize), NodeId>,
    unvisited_moves: Vec<(usize, usize)>,
    done: bool,
}

impl Node {
    fn new(game: &TicTacToe) -> Self {
        let available_moves = game.get_available_moves();
        Self {
            visits: 0,
            wins: 0,
            to_play: game.current_player,
            parent: None,
            children: HashMap::new(),
            unvisited_moves: available_moves,
            done: game.done()
        }
    }
}

struct Database {
    nodes: HashMap<NodeId, Node>,
}

impl Database {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    fn insert(&mut self, id: NodeId, node: Node) {
        self.nodes.insert(id, node);
    }
}

impl MCTS {
    pub(crate) fn select_move(&self, game: &TicTacToe) -> anyhow::Result<(usize, usize)> {
        let root = Node::new(game);
        let root_id = NodeId::new();
        let mut db = Database::new();
        db.insert(root_id, root);
        for _ in 0..200 {
            let game = &mut game.clone();
            let leaf_id = self.tree_policy(root_id, &mut db, game);
            debug!("Tree policy finished");
            let winner = self.default_policy(game);
            debug!("Default policy finished, winner: {:?}", winner);
            self.backpropagate(leaf_id, winner, &mut db);
            debug!("Backpropagation finished");
            debug!("\n{}", Self::print_tree(root_id, &db, 0));
        }
        let best_action = self.best_action(root_id, &db);
        Ok(best_action)
    }

    fn print_tree(root: NodeId, db: &Database, indent: usize) -> String {
        fn indent_str(indent: usize) -> String {
            let mut s = "".to_string();
            for _ in 0..indent {
                s.push_str(" ")
            }
            s
        }
        let mut s = String::new();
        let node = db.get(root).unwrap();

        if indent == 0 {
            s.push_str(&format!("win rate for {:?}: {:?} / {:?}\n", node.to_play, node.wins, node.visits));
        }

        // s.push_str(indent_str(indent).as_str());
        // s.push_str(&format!("{:?}: {:?} / {:?}\n", node.to_play, node.wins, node.visits));
        for (&action, &child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            s.push_str(indent_str(indent).as_str());
            s.push_str(&format!("  {:?} make move {:?} win rate for {:?}: {:?}/{:?} \n", node.to_play, action, child.to_play, child.wins, child.visits));
            s.push_str(&Self::print_tree(child_id, db, indent + 4));
        }
        s
    }

    fn tree_policy(&self, root: NodeId, db: &mut Database, game: &mut TicTacToe) -> NodeId {
        let mut node_id = root;
        loop {
            let node = db.get(node_id).unwrap();
            if node.done {
                debug!("Found a winner");
                break;
            }
            if node.unvisited_moves.len() > 0 {
                return self.expand(node_id, db, game);
            } else {
                let (action, child) = self.best_child(db, node_id);
                game.step(action.0, action.1).unwrap();
                debug!("\n{game}");
                node_id = child;
            }
        }
        node_id
    }

    fn expand(&self, node_id: NodeId, db: &mut Database, game: &mut TicTacToe) -> NodeId {
        debug!("Expanding");
        let node = db.get_mut(node_id).unwrap();
        let action = node.unvisited_moves.pop().unwrap();
        game.step(action.0, action.1).unwrap();
        let mut new_node = Node::new(game);
        new_node.parent = Some(node_id);
        let node_id = NodeId::new();
        node.children.insert(action, node_id);
        db.insert(node_id, new_node);
        debug!("\n{game}");
        node_id
    }

    fn best_child(&self, db: &Database, node_id: NodeId) -> ((usize, usize), NodeId) {
        // FIXME: select the best child according to the UCB formula
        let node = db.get(node_id).unwrap();
        debug!("children: {:?}", node.children);
        let (action, child_id) = node.children.iter().choose(&mut rand::thread_rng()).unwrap();
        (*action, *child_id)
    }

    fn default_policy(&self, game: &mut TicTacToe) -> Option<Player> {
        loop {
            if let Some(winner) = game.check_winner() {
                return Some(winner);
            }
            let available_moves = game.get_available_moves();
            if available_moves.len() == 0 {
                return None;
            }
            let action = available_moves.iter().choose(&mut rand::thread_rng()).unwrap();
            game.step(action.0, action.1).unwrap();
            debug!("\n{game}");
        }
    }

    fn backpropagate(&self, node_id: NodeId, winner: Option<Player>, db: &mut Database) {
        let mut node_id = node_id;
        loop {
            let node = db.get_mut(node_id).unwrap();
            node.visits += 1;
            if Some(node.to_play) == winner {
                node.wins += 1;
            }
            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    fn best_action(&self, node_id: NodeId, db: &Database) -> (usize, usize) {
        let node = db.get(node_id).unwrap();
        let mut best_action = None;
        let mut best_value = 0.0;
        for (&action, &child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            let value = child.wins as f32 / child.visits as f32;
            let value = 1. - value;
            debug!("action: {:?}, value: {:?}", action, value);
            if best_action.is_none() || value > best_value {
                best_action = Some(action);
                best_value = value;
            }
        }
        debug!("best action: {:?}", best_action);
        best_action.unwrap()
    }
}