extern crate sodiumoxide;

use errors::prelude::*;
use sodiumoxide::crypto::hash::sha256::*;

pub const HASHBYTES: usize = 32;

// these bytes are the same as openssl_hash(MessageDigest::sha256(), &[]) so we do not have to actually call the hash function
pub const EMPTY_HASH_BYTES : [u8; HASHBYTES] = [227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85];

pub fn hash(input: &[u8]) -> Result<Vec<u8>, IndyError> {
    let mut hasher = Hash::new_context()?;
    hasher.update(input)?;
    Ok(hasher.finish().map(|b| b.to_vec())?)
}

#[derive(Clone, Copy)]
pub struct Hasher {
    state: State,
}

impl Hasher {
    pub fn new() -> Result<Self, IndyError> {
        Ok(Hasher { state: State::new() })
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), IndyError> {
        self.state.update(data);
        Ok(())
    }

    pub fn finish(&mut self) -> Result<[u8; 32], IndyError> {
        let Digest(result) = self.state.finalize();
        Ok(result)
    }
}

pub struct Hash {}

impl Hash {
    pub fn new_context() -> Result<Hasher, IndyError> {
        Ok(Hasher::new()?)
    }

    pub fn hash_leaf<T>(leaf: &T) -> Result<Vec<u8>, IndyError> where T: Hashable {
        let mut ctx = Hash::new_context()?;
        ctx.update(&[0x00])?;
        leaf.update_context(&mut ctx)?;
        Ok(ctx.finish().map(|b| b.to_vec())?)
    }

    pub fn hash_nodes<T>(left: &T, right: &T) -> Result<Vec<u8>, IndyError> where T: Hashable {
        let mut ctx = Hash::new_context()?;
        ctx.update(&[0x01])?;
        left.update_context(&mut ctx)?;
        right.update_context(&mut ctx)?;
        Ok(ctx.finish().map(|b| b.to_vec())?)
    }
}

/// The type of values stored in a `MerkleTree` must implement
/// this trait, in order for them to be able to be fed
/// to a Ring `Context` when computing the hash of a leaf.
///
/// A default instance for types that already implements
/// `AsRef<[u8]>` is provided.
///
/// ## Example
///
/// Here is an example of how to implement `Hashable` for a type
/// that does not (or cannot) implement `AsRef<[u8]>`:
///
/// ```ignore
/// impl Hashable for PublicKey {
///     fn update_context(&self, context: &mut Hasher) -> Result<(), CommonError> {
///         let bytes: Vec<u8> = self.to_bytes();
///         Ok(context.update(&bytes)?)
///     }
/// }
/// ```
pub trait Hashable {
    /// Update the given `context` with `self`.
    fn update_context(&self, context: &mut Hasher) -> Result<(), IndyError>;
}

impl<T: AsRef<[u8]>> Hashable for T {
    fn update_context(&self, context: &mut Hasher) -> Result<(), IndyError> {
        context.update(self.as_ref())
    }
}
