/*
use std::collections::HashMap;

use rlp::{DecoderError as RlpDecoderError, Prototype as RlpPrototype,
          RlpStream,
          UntrustedRlp,
};

use indy_api_types::errors::prelude::*;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Extension(Extension),
    Full(FullNode),
    Hash(Vec<u8>),
    Blank,
}

impl Node {
    const RADIX: usize = 16;
    const FULL_SIZE: usize = Node::RADIX + 1;
    const PAIR_SIZE: usize = 2;
    const HASH_SIZE: usize = 32;
    const EMPTY_SIZE: usize = 0;
    const IS_LEAF_MASK: u8 = 0x20;
    const IS_PATH_ODD_MASK: u8 = 0x10;
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct FullNode {
    nodes: [Option<Box<Node>>; Node::RADIX],
    value: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Leaf {
    path: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Extension {
    path: Vec<u8>,
    next: Box<Node>,
}

impl rlp::Encodable for Node {
    fn rlp_append(&self, s: &mut RlpStream) {
        match *self {
            Node::Hash(ref hash) => {
                s.append_internal(&hash.as_slice());
            }
            Node::Leaf(ref pair) => {
                s.begin_list(Node::PAIR_SIZE);
                s.append(&pair.path);
                s.append(&pair.value);
            }
            Node::Extension(ref ext) => {
                s.begin_list(Node::PAIR_SIZE);
                s.append(&ext.path);
                s.append(ext.next.as_ref());
            }
            Node::Full(ref node) => {
                s.begin_list(Node::FULL_SIZE);
                for node in &node.nodes {
                    if let Some(ref node) = *node {
                        s.append(node.as_ref());
                    } else {
                        s.append_empty_data();
                    }
                }
                if let Some(ref value) = node.value {
                    s.append(value);
                } else {
                    s.append_empty_data();
                }
            }
            Node::Blank => {
                s.append_empty_data();
            }
        }
    }
}

impl rlp::Decodable for Node {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, RlpDecoderError> {
        match rlp.prototype()? {
            RlpPrototype::List(Node::PAIR_SIZE) => {
                let path: Vec<u8> = rlp.at(0)?.as_val()?;
                if path[0] & Node::IS_LEAF_MASK == Node::IS_LEAF_MASK {
                    Ok(Node::Leaf(Leaf {
                        path: rlp.at(0)?.as_val()?,
                        value: rlp.at(1)?.as_val()?,
                    }))
                } else if path[0] & Node::IS_LEAF_MASK == 0x00 {
                    Ok(Node::Extension(Extension {
                        path: rlp.at(0)?.as_val()?,
                        next: Box::new(rlp.at(1)?.as_val()?),
                    }))
                } else {
                    error!("RLP for path in Patricia Merkle Trie contains incorrect flags byte {}", path[0]);
                    Err(RlpDecoderError::Custom("Path contains incorrect flags byte"))
                }
            }
            RlpPrototype::List(Node::FULL_SIZE) => {
                let mut nodes: [Option<Box<Node>>; Node::RADIX] = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];
                for i in 0..Node::RADIX {
                    let cur = rlp.at(i)?;
                    match cur.prototype()? {
                        RlpPrototype::Data(0) => {
                            continue
                        }
                        _ => {
                            nodes[i] = Some(Box::new(cur.as_val()?));
                        }
                    }
                }
                let value: Option<Vec<u8>> = if !rlp.at(Node::RADIX)?.is_empty() {
                    Some(rlp.at(Node::RADIX)?.as_val()?)
                } else {
                    None
                };
                Ok(Node::Full(FullNode {nodes, value }))
            }
            RlpPrototype::Data(Node::HASH_SIZE) => {
                Ok(Node::Hash(rlp.as_val()?))
            }
            RlpPrototype::Data(Node::EMPTY_SIZE) => {
                Ok(Node::Blank)
            }
            _ => {
                error!("Unexpected data while parsing Patricia Merkle Trie: {:?}: {:?}", rlp.prototype(), rlp);
                Err(RlpDecoderError::Custom("Unexpected data"))
            }
        }
    }
}

type NodeHash = sha3::digest::generic_array::GenericArray<u8, <sha3::Sha3_256 as sha3::digest::FixedOutput>::OutputSize>;
pub type TrieDB<'a> = HashMap<NodeHash, &'a Node>;

impl Node {
    pub fn get_hash(&self) -> NodeHash{
        use rlp::{
            encode as rlp_encode
        };
        use sha3::{digest::FixedOutput};
        use sha3::Digest;
        let encoded = rlp_encode(self);
        let mut hasher = sha3::Sha3_256::default();
        hasher.input(encoded.to_vec().as_slice());
        hasher.fixed_result()
    }
    pub fn get_str_value<'a, 'b>(&'a self, db: &'a TrieDB, path: &'b [u8]) -> IndyResult<Option<String>> {
        let value = self.get_value(db, path)?;
        if let Some(vec) = value {
            let str = String::from_utf8(vec)
                .to_indy(IndyErrorKind::InvalidStructure, "Patricia Merkle Trie contains malformed utf8 string")?;

            trace!("Str value from Patricia Merkle Trie {}", str);
            Ok(Some(str))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_values<'a>(&'a self, db: &'a TrieDB, prefix: Option<&[u8]>) -> IndyResult<Vec<(String, String)>> {
        let node_and_prefix = prefix.map(|prf| self.get_node(db, &prf)).unwrap_or_else(|| Ok(Some((self, vec![]))))?;
        if let Some((node, prf)) = node_and_prefix {
            let vals = node._get_all_values(db, prf)?;
            let mut res: Vec<(String, String)> = Vec::with_capacity(vals.len());
            for (key, val) in vals {
                res.push((Node::_nibbles_to_str(&key)?, val))
            }
            Ok(res)
        } else {
            Ok(vec![])
        }
    }

    pub fn get_node<'a, 'b>(&'a self, db: &'a TrieDB, path: &'b [u8]) -> IndyResult<Option<(&Node, Vec<u8>)>> {
        trace!("Node::get_node >> path: {:?}", path);
        let nibble_path = Node::path_to_nibbles(path);
        trace!("Node::get_node >> made some nibbles >> nibbles: {:?}", nibble_path);
        self._get_node(db, nibble_path.as_slice(), vec![].as_slice())
    }

    fn _get_node<'a, 'b>(&'a self, db: &'a TrieDB, path: &'b [u8], seen_path: &'b [u8]) -> IndyResult<Option<(&Node, Vec<u8>)>> {
        trace!("Getting node for prefix, cur node: {:?}, prefix: {:?}", self, path);
        match *self {
            Node::Full(ref node) => {
                if path.is_empty() {
                    return Ok(Some((self, seen_path.to_vec())));
                }
                if let Some(ref next) = node.nodes[path[0] as usize] {
                    let mut new_seen_path = Vec::with_capacity(seen_path.len() + path[..1].len());
                    new_seen_path.extend_from_slice(seen_path);
                    new_seen_path.extend_from_slice(&path[..1]);
                    return next._get_node(db, &path[1..], new_seen_path.as_slice());
                }
                Ok(None)
            }
            Node::Hash(ref hash) => {
                let hash = NodeHash::from_slice(hash.as_slice());
                if let Some(ref next) = db.get(hash) {
                    next._get_node(db, path, seen_path)
                } else {
                    Err(err_msg(IndyErrorKind::InvalidStructure, "Incomplete key-value DB for Patricia Merkle Trie to get value by the key"))
                }
            }
            Node::Leaf(ref pair) => {
                let (is_leaf, pair_path) = Node::parse_path(pair.path.as_slice());

                if !is_leaf {
                    return Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: node marked as leaf but path contains extension flag"));
                }

                trace!("Node::_get_value in Leaf searched path {:?}, stored path {:?}", String::from_utf8(path.to_vec()), String::from_utf8(pair_path.clone()));

                if pair_path.starts_with(&path) {
                    Ok(Some((self, seen_path.to_vec())))
                } else {
                    Ok(None)
                }
            }
            Node::Extension(ref pair) => {
                let (is_leaf, pair_path) = Node::parse_path(pair.path.as_slice());

                if is_leaf {
                    return Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: node marked as extension but path contains leaf flag"));
                }

                trace!("current extension node path: {:?}", pair_path);

                if path.starts_with(&pair_path) {
                    let mut new_seen_path = Vec::with_capacity(seen_path.len() + pair_path.len());
                    new_seen_path.extend_from_slice(seen_path);
                    new_seen_path.extend_from_slice(pair_path.as_slice());
                    pair.next._get_node(db, &path[pair_path.len()..], new_seen_path.as_slice())
                } else if pair_path.starts_with(&path) {
                    Ok(Some((self, seen_path.to_vec())))
                } else {
                    Ok(None)
                }
            }
            Node::Blank => {
                Ok(None)
            }
        }
    }

    fn _get_all_values<'a>(&'a self, db: &'a TrieDB, prefix: Vec<u8>) -> IndyResult<Vec<(Vec<u8>, String)>> {
        trace!("Getting all values, cur node: {:?}", self);
        match *self {
            Node::Full(ref node) => {
                let mut res = vec![];
                for nibble in 0..16 {
                    let next = &node.nodes[nibble];
                    if let Some(next) = next {
                        trace!("Checking nibble {}", nibble);
                        let mut prefix_nibble = prefix.clone();
                        prefix_nibble.push(nibble as u8);
                        let mut sub_res: Vec<(Vec<u8>, String)> = next._get_all_values(db, prefix_nibble)?;
                        trace!("Got some values: {:?}", sub_res);
                        res.append(&mut sub_res);
                        trace!("Intermediate result: {:?}", res);
                    }
                }
                if let Some(ref value) = node.value.as_ref() {
                    let mut vec: Vec<Vec<u8>> = UntrustedRlp::new(value).as_list().unwrap_or_default(); //default will cause error below

                    if let Some(val) = vec.pop() {
                        if vec.is_empty() {
                            res.push((prefix.clone(), String::from_utf8(val.clone())
                                .to_indy(IndyErrorKind::InvalidStructure, "Patricia Merkle Trie contains malformed utf8 string")?));
                        }
                    }
                }
                Ok(res)
            }
            Node::Hash(ref hash) => {
                trace!("Node::_get_all_values in Hash");
                let hash = NodeHash::from_slice(hash.as_slice());
                if let Some(ref next) = db.get(hash) {
                    trace!("found value in db");
                    next._get_all_values(db, prefix)
                } else {
                    Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: empty hash node when it should not be empty"))
                }
            }
            Node::Leaf(ref pair) => {
                let (is_leaf, pair_path) = Node::parse_path(pair.path.as_slice());

                if !is_leaf {
                    return Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: node marked as leaf but path contains extension flag"));
                }

                trace!("Node::_get_all_values in Leaf stored path {:?}", String::from_utf8(pair_path.clone()));

                let mut vec: Vec<Vec<u8>> = UntrustedRlp::new(pair.value.as_slice()).as_list().unwrap_or_default(); //default will cause error below

                let mut full_path = prefix.clone();
                let mut path_left = pair_path.clone();
                full_path.append(&mut path_left);

                if let Some(val) = vec.pop() {
                    if vec.is_empty() {
                        return Ok(vec![(full_path, String::from_utf8(val.clone())
                            .to_indy(IndyErrorKind::InvalidStructure, "Patricia Merkle Trie contains malformed utf8 string")?)]);
                    }
                }
                Err(err_msg(IndyErrorKind::InvalidStructure, "Unexpected data format of value in Patricia Merkle Trie"))
            }
            Node::Extension(ref pair) => {
                let (is_leaf, pair_path) = Node::parse_path(pair.path.as_slice());

                if is_leaf {
                    return Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: node marked as extension but path contains leaf flag"));
                }

                let mut full_path = prefix.clone();
                let mut path_left = pair_path.clone();
                full_path.append(&mut path_left);

                let values = pair.next._get_all_values(db, full_path)?;

                Ok(values)
            }
            Node::Blank => {
                Ok(vec![])
            }
        }
    }

    fn get_value<'a, 'b>(&'a self, db: &'a TrieDB, path: &'b [u8]) -> IndyResult<Option<Vec<u8>>> {
        let nibble_path = Node::path_to_nibbles(path);
        match self._get_value(db, nibble_path.as_slice())? {
            Some(v) => {
                trace!("Raw value from Patricia Merkle Trie {:?}", v);
                let mut vec: Vec<Vec<u8>> = UntrustedRlp::new(v.as_slice()).as_list().unwrap_or_default(); //default will cause error below

                if let Some(val) = vec.pop() {
                    if vec.is_empty() {
                        return Ok(Some(val));
                    }
                }

                Err(err_msg(IndyErrorKind::InvalidStructure, "Unexpected data format of value in Patricia Merkle Trie"))
            }
            None => Ok(None)
        }
    }
    fn _get_value<'a, 'b>(&'a self, db: &'a TrieDB, path: &'b [u8]) -> IndyResult<Option<&'a Vec<u8>>> {
        trace!("Check proof, cur node: {:?}", self);
        match self._get_node(db, path, vec![].as_slice())? {
            Some((Node::Full(ref node), _)) => {
                Ok(node.value.as_ref())
            }
            Some((Node::Hash(_), _)) => Ok(None),
            Some((Node::Leaf(ref pair), _)) => {
                let (is_leaf, _) = Node::parse_path(pair.path.as_slice());

                if !is_leaf {
                    return Err(err_msg(IndyErrorKind::InvalidState, "Incorrect Patricia Merkle Trie: node marked as leaf but path contains extension flag"));
                }

                Ok(Some(&pair.value))
            }
            Some((Node::Extension(_), _)) => Ok(None),
            Some((Node::Blank, _)) => Ok(None),
            None => Ok(None)
        }
    }

    fn path_to_nibbles(path: &[u8]) -> Vec<u8> {
        let mut nibble_path: Vec<u8> = Vec::with_capacity(2*path.len());

        for s in path {
            nibble_path.push(s >> 4);
            nibble_path.push(s & 0x0F);
        }

        nibble_path
    }

    fn _nibbles_to_str(nibbles: &[u8]) -> IndyResult<String> {
        trace!("Node::_nibbles_to_str >> nibbles: {:?}", nibbles);
        let mut res: Vec<u8> = Vec::with_capacity(nibbles.len() / 2);
        for x in (0..nibbles.len()).step_by(2) {
            let h: u8 = (nibbles[x] << 4) + nibbles[x+1];
            res.push(h)
        }
        String::from_utf8(res).to_indy(IndyErrorKind::InvalidStructure, "Patricia Merkle Trie contains malformed utf8 string")
    }

    fn parse_path(path: &[u8]) -> (bool, Vec<u8>) {
        let is_leaf: bool = path[0] & Node::IS_LEAF_MASK == Node::IS_LEAF_MASK;
        let is_odd: bool = path[0] & Node::IS_PATH_ODD_MASK == Node::IS_PATH_ODD_MASK;
        let mut nibbles: Vec<u8> = Node::path_to_nibbles(&path[1..]); //TODO avoid copy
        if is_odd {
            nibbles.insert(0, path[0] & 0x0F);
        }
        (is_leaf, nibbles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indy_utils::crypto::base64;

    #[test]
    fn node_deserialize_works_for_emtpy() {
        assert_eq!(UntrustedRlp::new(&base64::decode("wYA=").unwrap()).as_list::<Node>().unwrap(),
                   vec![Node::Blank]);
    }

    #[test]
    fn node_serialize_works_for_emtpy() {
        assert_eq!(base64::encode(&rlp::encode_list(&vec![Node::Blank])),
                   "wYA=");
    }
}
*/
