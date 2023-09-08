use std::{collections::HashMap, hash::Hash, sync::atomic::AtomicUsize};

use log::debug;
use rand::seq::IteratorRandom;

use crate::game::Game;

pub(crate) struct Mcts<T: Game> {
    _phantom: std::marker::PhantomData<T>,
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
    reward: f32,
    to_play: T::Player,
    parent: Option<NodeId>,
    children: HashMap<T::Action, NodeId>,
    unvisited_moves: Vec<T::Action>,
    done: bool,
}

impl<T: Game> Node<T> {
    fn new(game: &T) -> Self {
        let available_moves = game.get_available_moves();
        Self {
            visits: 0,
            reward: 0.,
            to_play: game.current_player(),
            parent: None,
            children: HashMap::new(),
            unvisited_moves: available_moves,
            done: game.done(),
        }
    }
}

type NodeMap<T> = HashMap<NodeId, Node<T>>;

impl<T: Game> Mcts<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn select_move(&self, game: &T) -> anyhow::Result<T::Action> {
        let root = Node::new(game);
        let root_id = NodeId::new();
        let mut db = NodeMap::new();
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

    fn print_tree(root: NodeId, db: &NodeMap<T>, indent: usize) -> String {
        fn indent_str(indent: usize) -> String {
            let mut s = "".to_string();
            for _ in 0..indent {
                s.push(' ')
            }
            s
        }
        let mut s = String::new();
        let node = db.get(&root).unwrap();

        if indent == 0 {
            s.push_str(&format!(
                "win rate for {:?}: {:?} / {:?}\n",
                node.to_play, node.reward, node.visits
            ));
        }

        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            s.push_str(indent_str(indent).as_str());
            s.push_str(&format!(
                "  {:?} make move {:?} win rate for {:?}: {:?}/{:?} \n",
                node.to_play, action, child.to_play, child.reward, child.visits
            ));
            s.push_str(&Self::print_tree(*child_id, db, indent + 4));
        }
        s
    }

    fn tree_policy(&self, root: NodeId, db: &mut NodeMap<T>, game: &mut T) -> NodeId {
        let mut node_id = root;
        loop {
            let node = db.get(&node_id).unwrap();
            if node.done {
                debug!("Found a winner");
                break;
            }
            if !node.unvisited_moves.is_empty() {
                return self.expand(node_id, db, game);
            } else {
                let (action, child) = self.best_child(db, node_id);
                game.step(action).unwrap();
                debug!("\n{game}");
                node_id = child;
            }
        }
        node_id
    }

    fn expand(&self, node_id: NodeId, db: &mut NodeMap<T>, game: &mut T) -> NodeId {
        debug!("Expanding");
        let node = db.get_mut(&node_id).unwrap();
        let action = node.unvisited_moves.pop().unwrap();
        game.step(action.clone()).unwrap();
        let mut new_node = Node::new(game);
        new_node.parent = Some(node_id);
        let node_id = NodeId::new();
        node.children.insert(action, node_id);
        db.insert(node_id, new_node);
        debug!("\n{game}");
        node_id
    }

    fn best_child(&self, db: &NodeMap<T>, node_id: NodeId) -> (T::Action, NodeId) {
        // FIXME: select the best child according to the UCB formula
        let node = db.get(&node_id).unwrap();
        let mut best_action = None;
        let mut best_node_id = None;
        let mut best_value = 0.0;
        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            let value = child.reward / child.visits as f32;
            let mut value = 1. - value;
            value += (2. * (node.visits as f32).ln() / child.visits as f32).sqrt();
            debug!("action: {:?}, value: {:?}", action, value);
            if best_action.is_none() || value > best_value {
                best_action = Some(action);
                best_node_id = Some(child_id);
                best_value = value;
            }
        }
        (best_action.unwrap().clone(), best_node_id.unwrap().clone())
    }

    fn default_policy(&self, game: &mut T) -> Option<T::Player> {
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
            debug!("\n{game}");
        }
    }

    fn backpropagate(&self, node_id: NodeId, winner: Option<T::Player>, db: &mut NodeMap<T>) {
        let mut node_id = node_id;
        loop {
            let node = db.get_mut(&node_id).unwrap();
            node.visits += 1;
            if let Some(winner) = &winner {
                if &node.to_play == winner {
                    node.reward += 1.;
                }
            } else {
                node.reward += 0.5;
            }
            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    fn best_action(&self, node_id: NodeId, db: &NodeMap<T>) -> T::Action {
        let node = db.get(&node_id).unwrap();
        let mut best_action = None;
        let mut best_value = 0.0;
        for (action, child_id) in node.children.iter() {
            let child = db.get(child_id).unwrap();
            let value = child.reward as f32 / child.visits as f32;
            let value = 1. - value;
            debug!("action: {:?}, value: {:?}", action, value);
            if best_action.is_none() || value > best_value {
                best_action = Some(action);
                best_value = value;
            }
        }
        debug!("best action: {:?}", best_action);
        best_action.unwrap().clone()
    }
}
