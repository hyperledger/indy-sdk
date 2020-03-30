/*
use std::cmp;

use indy_api_types::errors::prelude::*;
pub use crate::services::ledger::merkletree::proof::{
    Lemma,
    Positioned,
    Proof
};
use indy_utils::crypto::hash::{Hash};

pub type TreeLeafData = Vec<u8>;

/// Binary Tree where leaves hold a stand-alone value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tree {
    Empty {
        hash: Vec<u8>
    },

    Leaf {
        hash: Vec<u8>,
        value: TreeLeafData
    },

    Node {
        hash: Vec<u8>,
        left: Box<Tree>,
        right: Box<Tree>
    }
}

impl Tree {
    /// Create an empty tree
    pub fn empty(hash: Vec<u8>) -> Self {
        Tree::Empty {
            hash: hash.to_vec()
        }
    }

    /// Create a new tree
    pub fn new(hash: Vec<u8>, value: TreeLeafData) -> Self {
        Tree::Leaf {
            hash: hash.to_vec(),
            value
        }
    }

    /// Create a new leaf
    pub fn new_leaf(value: TreeLeafData) -> IndyResult<Tree> {

        let hash = Hash::hash_leaf(&value)?;
        Ok(Tree::new(hash, value))
    }

    /// Returns a hash from the tree.
    pub fn hash(&self) -> &Vec<u8> {
        match *self {
            Tree::Empty { ref hash }    => hash,
            Tree::Leaf { ref hash, .. } => hash,
            Tree::Node { ref hash, .. } => hash
        }
    }

    /// Returns a borrowing iterator over the leaves of the tree.
    pub fn iter(&self) -> LeavesIterator {
        LeavesIterator::new(self)
    }

    pub fn get_height(&self) -> usize {
        match *self {
            Tree::Empty { .. } => { 0 },
            Tree::Node { ref left, ref right, .. } => {
                1 + cmp::max(left.get_height(),right.get_height())
            },
            Tree::Leaf { .. } => { 0 }
        }
    }

    pub fn get_count(&self) -> usize {
        match *self {
            Tree::Empty { .. } => { 0 },
            Tree::Node { ref left, ref right, .. } => {
                left.get_count() + right.get_count()
            },
            Tree::Leaf { .. } => { 1 }
        }
    }
}


/// An borrowing iterator over the leaves of a `Tree`.
/// Adapted from http://codereview.stackexchange.com/q/110283.
#[allow(missing_debug_implementations)]
pub struct LeavesIterator<'a> {
    current_value: Option<&'a TreeLeafData>,
    right_nodes: Vec<&'a Tree>
}

impl <'a> LeavesIterator<'a> {

    fn new(root: &'a Tree) -> Self {
        let mut iter = LeavesIterator {
            current_value: None,
            right_nodes: Vec::new()
        };

        iter.add_left(root);

        iter
    }

    fn add_left(&mut self, mut tree: &'a Tree) {
        loop {
            match *tree {
                Tree::Empty { .. } => {
                    self.current_value = None;
                    break;
                },

                Tree::Node { ref left, ref right, .. } => {
                    self.right_nodes.push(right);
                    tree = left;
                },

                Tree::Leaf { ref value, .. } => {
                    self.current_value = Some(value);
                    break;
                }
            }
        }
    }

}

impl <'a> Iterator for LeavesIterator<'a> {

    type Item = &'a TreeLeafData;

    fn next(&mut self) -> Option<&'a TreeLeafData> {
        let result = self.current_value.take();

        if let Some(rest) = self.right_nodes.pop() {
            self.add_left(rest);
        }

        result
    }

}

/// An iterator over the leaves of a `Tree`.
#[allow(missing_debug_implementations)]
pub struct LeavesIntoIterator {
    current_value: Option<TreeLeafData>,
    right_nodes: Vec<Tree>
}

impl LeavesIntoIterator {

    fn new(root: Tree) -> Self {
        let mut iter = LeavesIntoIterator {
            current_value: None,
            right_nodes: Vec::new()
        };

        iter.add_left(root);

        iter
    }

    fn add_left(&mut self, mut tree: Tree) {
        loop {
            match tree {
                Tree::Empty { .. } => {
                    self.current_value = None;
                    break;
                },

                Tree::Node { left, right, .. } => {
                    self.right_nodes.push(*right);
                    tree = *left;
                },

                Tree::Leaf { value, .. } => {
                    self.current_value = Some(value);
                    break;
                }
            }
        }
    }

}

impl Iterator for LeavesIntoIterator {

    type Item = TreeLeafData;

    fn next(&mut self) -> Option<TreeLeafData> {
        let result = self.current_value.take();

        if let Some(rest) = self.right_nodes.pop() {
            self.add_left(rest);
        }

        result
    }

}

impl IntoIterator for Tree {

    type Item     = TreeLeafData;
    type IntoIter = LeavesIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        LeavesIntoIterator::new(self)
    }

}
*/
