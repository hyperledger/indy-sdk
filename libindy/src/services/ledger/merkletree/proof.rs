/*
use crate::services::ledger::merkletree::tree::{Tree, TreeLeafData};
use indy_utils::crypto::hash::Hash;
use indy_api_types::errors::prelude::*;

/// An inclusion proof represent the fact that a `value` is a member
/// of a `MerkleTree` with root hash `root_hash`.
#[derive(Clone, Debug)]
pub struct Proof {
    /// The hash of the root of the original `MerkleTree`
    pub root_hash: Vec<u8>,

    /// The first `Lemma` of the `Proof`
    pub lemma: Lemma,

    /// The value concerned by this `Proof`
    pub value: TreeLeafData
}

impl Proof {

    /// Constructs a new `Proof`
    pub fn new(root_hash: Vec<u8>, lemma: Lemma, value: TreeLeafData) -> Self {
        Proof {
            root_hash,
            lemma,
            value
        }
    }

    /// Checks whether this inclusion proof is well-formed,
    /// and whether its root hash matches the given `root_hash`.
    pub fn validate(&self, root_hash: &[u8]) -> IndyResult<bool> {
        if self.root_hash != root_hash || self.lemma.node_hash != root_hash {
            return Ok(false)
        }

        Ok(self.validate_lemma(&self.lemma)?)
    }

    fn validate_lemma(&self, lemma: &Lemma) -> IndyResult<bool> {
        match lemma.sub_lemma {

            None =>
                Ok(lemma.sibling_hash.is_none()),

            Some(ref sub) =>
                match lemma.sibling_hash {
                    None =>
                        Ok(false),

                    Some(Positioned::Left(ref hash)) => {
                        let combined = Hash::hash_nodes(hash, &sub.node_hash)?;
                        let hashes_match = combined.to_vec().as_slice() == lemma.node_hash.as_slice();
                        Ok(hashes_match && self.validate_lemma(sub)?)
                    }

                    Some(Positioned::Right(ref hash)) => {
                        let combined = Hash::hash_nodes(&sub.node_hash, hash)?;
                        let hashes_match = combined.to_vec().as_slice() == lemma.node_hash.as_slice();
                        Ok(hashes_match && self.validate_lemma(sub)?)
                    }

                }
        }
    }

}


/// A `Lemma` holds the hash of a node, the hash of its sibling node,
/// and a sub lemma, whose `node_hash`, when combined with this `sibling_hash`
/// must be equal to this `node_hash`.
#[derive(Clone, Debug, PartialEq)]
pub struct Lemma {
    pub node_hash: Vec<u8>,
    pub sibling_hash: Option<Positioned<Vec<u8>>>,
    pub sub_lemma: Option<Box<Lemma>>
}

impl Lemma {

    /// Attempts to generate a proof that the a value with hash `needle` is a member of the given `tree`.
    pub fn new(tree: &Tree, needle: &[u8]) -> Option<Lemma> {
        match *tree {
            Tree::Empty {.. } =>
                None,

            Tree::Leaf { ref hash, .. } =>
                Lemma::new_leaf_proof(hash, needle),

            Tree::Node { ref hash, ref left, ref right } =>
                Lemma::new_tree_proof(hash, needle, left, right)
        }
    }

    fn new_leaf_proof(hash: &[u8], needle: &[u8]) -> Option<Lemma> {
        if *hash == *needle {
            Some(Lemma {
                node_hash: hash.into(),
                sibling_hash: None,
                sub_lemma: None
            })
        } else {
            None
        }
    }

    fn new_tree_proof(hash: &[u8], needle: &[u8], left: &Tree, right: &Tree) -> Option<Lemma> {
        Lemma::new(left, needle)
            .map(|lemma| {
                let right_hash = right.hash().clone();
                let sub_lemma = Some(Positioned::Right(right_hash));
                (lemma, sub_lemma)
            })
            .or_else(|| {
                let sub_lemma = Lemma::new(right, needle);
                sub_lemma.map(|lemma| {
                    let left_hash = left.hash().clone();
                    let sub_lemma = Some(Positioned::Left(left_hash));
                    (lemma, sub_lemma)
                })
            })
            .map(|(sub_lemma, sibling_hash)| {
                Lemma {
                    node_hash: hash.into(),
                    sibling_hash,
                    sub_lemma: Some(Box::new(sub_lemma))
                }
            })
    }

}

/// Tags a value so that we know from which branch of a `Tree` (if any) it was found.
#[derive(Clone, Debug, PartialEq)]
pub enum Positioned<T> {

    /// The value was found in the left branch
    Left(T),

    /// The value was found in the right branch
    Right(T)
}
*/
