/*
pub mod tree;
pub mod proof;
pub mod merkletree;

use self::tree::*;
use self::merkletree::*;
use indy_api_types::errors::prelude::*;
use indy_utils::crypto::hash::Hash;

impl MerkleTree {
    fn count_bits(v: usize) -> usize {
        let mut ret = 0;
        let mut val = v;
        while val != 0 {
            val &= val - 1;
            ret += 1;
        }
        ret
    }

    pub fn find_hash<'a>(from: &'a Tree, required_hash: &Vec<u8>) -> Option<&'a Tree> {
        match *from {
            Tree::Empty { .. } => {
                assert!(false);
                None
            }
            Tree::Node { ref left, ref right, ref hash, .. } => {
                if hash == required_hash {
                    Some(from)
                } else {
                    let right = MerkleTree::find_hash(right, required_hash);
                    match right {
                        Some(r) => {
                            Some(r)
                        }
                        None => {
                            let left = MerkleTree::find_hash(left, required_hash);
                            match left {
                                Some(r) => {
                                    Some(r)
                                }
                                None => {
                                    None
                                }
                            }
                        }
                    }
                }
            }
            Tree::Leaf { ref hash, .. } => {
                if hash == required_hash {
                    Some(from)
                } else {
                    None
                }
            }
        }
    }

    pub fn consistency_proof(&self,
                             new_root_hash: &Vec<u8>, new_size: usize,
                             proof: &Vec<Vec<u8>>) -> IndyResult<bool> {
        if self.count == 0 {
            // empty old tree
            return Ok(true);
        }
        if self.count == new_size && self.root_hash() == new_root_hash {
            // identical trees
            return Ok(true);
        }
        if self.count > new_size {
            // old tree is bigger!
            assert!(false);
            return Ok(false);
        }

        let mut old_node = self.count - 1;
        let mut new_node = new_size - 1;

        while old_node % 2 != 0 {
            old_node /= 2;
            new_node /= 2;
        }

        let mut proofs = proof.iter();
        let mut old_hash: Vec<u8>;
        let mut new_hash: Vec<u8>;

        if old_node != 0 {
            new_hash = unwrap_opt_or_return!(proofs.next(), Ok(false)).to_vec();
            old_hash = new_hash.clone();
        } else {
            new_hash = self.root_hash().to_vec();
            old_hash = new_hash.clone();
        }

        while old_node != 0 {
            if old_node % 2 != 0 {
                let next_proof = unwrap_opt_or_return!(proofs.next(), Ok(false));
                old_hash = Hash::hash_nodes(next_proof, &old_hash)?.to_vec();
                new_hash = Hash::hash_nodes(next_proof, &new_hash)?.to_vec();
            } else if old_node < new_node {
                new_hash = Hash::hash_nodes(&new_hash,
                                            unwrap_opt_or_return!(proofs.next(), Ok(false)))?.to_vec();
            }
            old_node /= 2;
            new_node /= 2;
        }

        while new_node != 0 {
            let n = unwrap_opt_or_return!(proofs.next(), Ok(false));
            new_hash = Hash::hash_nodes(&new_hash, n)?.to_vec();
            new_node /= 2;
        }

        if new_hash != *new_root_hash {
            // new hash differs
            return Ok(false);
        }

        if old_hash != *self.root_hash() {
            // old hash differs
            return Ok(false);
        }

        Ok(true)
    }

    pub fn append(&mut self, node: TreeLeafData) -> IndyResult<()> {
        if self.count == 0 {
            // empty tree
            self.root = Tree::new_leaf(node)?;
            self.count += 1;
        } else if Self::count_bits(self.count) != 1 {
            // add to right subtree
            match self.root.clone() {
                Tree::Node { ref left, ref right, .. } => {
                    let mut iter = right.iter().map(|x| (*x).clone()).collect::<Vec<TreeLeafData>>();
                    iter.push(node);
                    let new_right = MerkleTree::from_vec(iter)?;
                    let combined_hash = Hash::hash_nodes(
                        left.hash(),
                        new_right.root_hash() as &Vec<u8>
                    )?;
                    self.root = Tree::Node {
                        left: (*left).clone(),
                        right: Box::new(new_right.root),
                        hash: combined_hash.to_vec()
                    };
                    self.count += 1;
                    self.nodes_count += 1;
                }
                _ => {
                    assert!(false);
                }
            }
        } else {
            // add tree layer
            let new_right = MerkleTree::from_vec(vec![node])?;
            match self.root.clone() {
                Tree::Node { ref hash, .. } => {
                    let combined_hash = Hash::hash_nodes(
                        hash,
                        new_right.root_hash()
                    )?;
                    self.root = Tree::Node {
                        left: Box::new(self.root.clone()),
                        right: Box::new(new_right.root),
                        hash: combined_hash.to_vec()
                    };
                    self.count += 1;
                    self.nodes_count += 1;
                }
                Tree::Leaf { ref hash, ref value } => {
                    let combined_hash = Hash::hash_nodes(
                        hash,
                        new_right.root_hash()
                    )?;
                    self.root = Tree::Node {
                        left: Box::new(Tree::new_leaf((*value).clone())?),
                        right: Box::new(new_right.root),
                        hash: combined_hash.to_vec()
                    };
                    self.count += 1;
                    self.nodes_count += 1;
                }
                _ => {
                    assert!(false);
                }
            }
            self.height += 1;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_base58::FromBase58;

    #[test]
    fn append_works() {
        let values = vec![
            "{\"data\":{\"alias\":\"Node1\",\"client_ip\":\"192.168.1.35\",\"client_port\":9702,\"node_ip\":\"192.168.1.35\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}",
            "{\"data\":{\"alias\":\"Node2\",\"client_ip\":\"192.168.1.35\",\"client_port\":9704,\"node_ip\":\"192.168.1.35\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}",
            "{\"data\":{\"alias\":\"Node3\",\"client_ip\":\"192.168.1.35\",\"client_port\":9706,\"node_ip\":\"192.168.1.35\",\"node_port\":9705,\"services\":[\"VALIDATOR\"]},\"dest\":\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\",\"identifier\":\"2yAeV5ftuasWNgQwVYzeHeTuM7LwwNtPR3Zg9N4JiDgF\",\"txnId\":\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\",\"type\":\"0\"}",
            "{\"data\":{\"alias\":\"Node4\",\"client_ip\":\"192.168.1.35\",\"client_port\":9708,\"node_ip\":\"192.168.1.35\",\"node_port\":9707,\"services\":[\"VALIDATOR\"]},\"dest\":\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\",\"identifier\":\"FTE95CVthRtrBnK2PYCBbC9LghTcGwi9Zfi1Gz2dnyNx\",\"txnId\":\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\",\"type\":\"0\"}"];
        let mut mt = MerkleTree::from_vec(vec![]).unwrap();

        for i in values {
            mt.append(String::from(i).as_bytes().to_vec()).unwrap();
        }
        assert_eq!(mt.root_hash_hex(), "1285070cf01debc1155cef8dfd5ba54c05abb919a4c08c8632b079fb1e1e5e7c");
    }

    #[test]
    fn find_hash_works() {
        let values = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"];
        let mut mt = MerkleTree::from_vec(vec![]).unwrap();

        for i in values {
            mt.append(String::from(i).as_bytes().to_vec()).unwrap();
        }

        assert_eq!(mt.count, 9);
        assert_eq!(mt.nodes_count, 8);

        let mut rh: Vec<u8>;

        rh = vec![0xe8, 0xbc, 0xd9, 0x7e, 0x34, 0x96, 0x93, 0xdc,
                  0xfe, 0xc0, 0x54, 0xfe, 0x21, 0x9a, 0xb3, 0x57,
                  0xb7, 0x5d, 0x3c, 0x1c, 0xd9, 0xf8, 0xbe, 0x17,
                  0x67, 0xf6, 0x09, 0x0f, 0x9c, 0x86, 0xf9, 0xfd];
        assert_ne!(MerkleTree::find_hash(&mt.root, &rh), None);

        rh = vec![0x22, 0x15, 0xe8, 0xac, 0x4e, 0x2b, 0x87, 0x1c,
                  0x2a, 0x48, 0x18, 0x9e, 0x79, 0x73, 0x8c, 0x95,
                  0x6c, 0x08, 0x1e, 0x23, 0xac, 0x2f, 0x24, 0x15,
                  0xbf, 0x77, 0xda, 0x19, 0x9d, 0xfd, 0x92, 0x0c];
        assert_ne!(MerkleTree::find_hash(&mt.root, &rh), None);

        rh = vec![0x23, 0x15, 0xe8, 0xac, 0x4e, 0x2b, 0x87, 0x1c,
                  0x2a, 0x48, 0x18, 0x9e, 0x79, 0x73, 0x8c, 0x95,
                  0x6c, 0x08, 0x1e, 0x23, 0xac, 0x2f, 0x24, 0x15,
                  0xbf, 0x77, 0xda, 0x19, 0x9d, 0xfd, 0x92, 0x0d];
        assert_eq!(MerkleTree::find_hash(&mt.root, &rh), None);
    }

    #[test]
    fn consistency_proof_works_for_valid_proof() {
        let values = vec![
            "{\"data\":{\"alias\":\"Node1\",\"client_ip\":\"192.168.1.35\",\"client_port\":9702,\"node_ip\":\"192.168.1.35\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}",
            "{\"data\":{\"alias\":\"Node2\",\"client_ip\":\"192.168.1.35\",\"client_port\":9704,\"node_ip\":\"192.168.1.35\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}"];
        let mut mt = MerkleTree::from_vec(vec![]).unwrap();

        for i in values {
            mt.append(String::from(i).as_bytes().to_vec()).unwrap();
        }

        let proofs: Vec<Vec<u8>> = vec![
            vec![0x26, 0x06, 0x53, 0x99, 0xf1, 0xe9, 0x0d, 0xba,
                 0x37, 0xe1, 0x86, 0xd8, 0x83, 0x3c, 0x07, 0x21,
                 0x26, 0xe3, 0xf4, 0xdf, 0xe6, 0x03, 0xe4, 0x1b,
                 0x41, 0x27, 0x1d, 0x83, 0x74, 0x72, 0x6f, 0x74],
            vec![0xf1, 0xb0, 0x51, 0xa9, 0x11, 0x4b, 0x69, 0xa7,
                 0x0f, 0x82, 0x91, 0xe3, 0x77, 0xf0, 0x78, 0x1f,
                 0x06, 0x63, 0xe6, 0x5c, 0x8b, 0xbc, 0x11, 0xe9,
                 0x00, 0x74, 0x8b, 0xb7, 0x55, 0xf3, 0xcd, 0x6e],
            vec![0x22, 0x6c, 0x66, 0x53, 0x08, 0xe4, 0xa8, 0x5a,
                 0x01, 0x7d, 0x52, 0x24, 0x24, 0x17, 0x91, 0xdc,
                 0xfa, 0x9e, 0x38, 0x55, 0x5a, 0x38, 0x7b, 0x33,
                 0x61, 0x4d, 0x7f, 0x5a, 0x68, 0x72, 0x60, 0xd6]
        ];

        assert!(mt.consistency_proof(&vec![0x77 as u8, 0xf1, 0x5a, 0x58, 0x07, 0xfd, 0xaa, 0x56,
                                           0x51, 0x28, 0xc5, 0x8f, 0x59, 0x1f, 0x4f, 0x03,
                                           0x25, 0x81, 0xfe, 0xe7, 0xd8, 0x61, 0x99, 0xae,
                                           0xf8, 0xae, 0xac, 0x7b, 0x05, 0x80, 0xbe, 0x0a],
                                     4,
                                     &proofs).unwrap());
    }

    #[test] // IS-708 Crash while consistency proof include empty 'proof' and invalid root_hash
    fn consistency_proof_works_for_empty_proof_and_invalid_root_hash() {
        let values = vec![
            "{\"data\":{\"alias\":\"Node1\",\"client_ip\":\"192.168.1.35\",\"client_port\":9702,\"node_ip\":\"192.168.1.35\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}",
            "{\"data\":{\"alias\":\"Node2\",\"client_ip\":\"192.168.1.35\",\"client_port\":9704,\"node_ip\":\"192.168.1.35\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}"];
        let mut mt = MerkleTree::from_vec(vec![]).unwrap();

        for i in values {
            mt.append(String::from(i).as_bytes().to_vec()).unwrap();
        }

        let proofs: Vec<Vec<u8>> = vec![];

        assert_eq!(false, mt.consistency_proof(&vec![0x77 as u8, 0xf1, 0x5a],
                                               4,
                                               &proofs).unwrap());
    }

    #[test]
    fn gen_proof_and_proof_validate_work() {
        let strvals = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
        let values = strvals.iter().map(|x| String::from(*x).as_bytes().to_vec()).collect::<Vec<_>>();
        let tree = MerkleTree::from_vec(values.clone()).unwrap();
        let root_hash = tree.root_hash();

        for value in values {
            let proof = tree.gen_proof(value).unwrap();
            let is_valid = proof.map(|p| p.validate(&root_hash).unwrap()).unwrap_or(false);

            assert!(is_valid);
        }
    }

    #[test]
    fn serialize_works() {
        let strvals = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
        let values = strvals.iter().map(|x| String::from(*x).as_bytes().to_vec()).collect::<Vec<_>>();
        let mt = MerkleTree::from_vec(values.clone()).unwrap();
        let serialized = serde_json::to_string(&mt).unwrap();
        let newmt: MerkleTree = serde_json::from_str(serialized.as_str()).unwrap();

        let mut collected = Vec::new();
        for value in &newmt {
            collected.push(value);
        }
        let refs = values.iter().collect::<Vec<_>>();
        assert_eq!(refs, collected);

        assert_eq!(mt.root_hash(), newmt.root_hash());
    }

    #[test]
    fn consistency_proof_works_for_old4_new8() {
        let all_str_values = vec![
            r#"{"data":{"alias":"Node1","client_ip":"10.0.0.2","client_port":9702,"node_ip":"10.0.0.2","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#,
            r#"{"data":{"alias":"Node2","client_ip":"10.0.0.2","client_port":9704,"node_ip":"10.0.0.2","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#,
            r#"{"data":{"alias":"Node3","client_ip":"10.0.0.2","client_port":9706,"node_ip":"10.0.0.2","node_port":9705,"services":["VALIDATOR"]},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya","identifier":"2yAeV5ftuasWNgQwVYzeHeTuM7LwwNtPR3Zg9N4JiDgF","txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4","type":"0"}"#,
            r#"{"data":{"alias":"Node4","client_ip":"10.0.0.2","client_port":9708,"node_ip":"10.0.0.2","node_port":9707,"services":["VALIDATOR"]},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA","identifier":"FTE95CVthRtrBnK2PYCBbC9LghTcGwi9Zfi1Gz2dnyNx","txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008","type":"0"}"#,
            r#"{"data":{"alias":"Node5","client_ip":"10.0.0.2","client_port":9710,"node_ip":"10.0.0.2","node_port":9709,"services":["VALIDATOR"]},"dest":"4SWokCJWJc69Tn74VvLS6t2G2ucvXqM9FDMsWJjmsUxe","identifier":"5NekXKJvGrxHvfxbXThySmaG8PmpNarXHCf1CkwTLfrg","txnId":"5abef8bc27d85d53753c5b6ed0cd2e197998c21513a379bfcf44d9c7a73c3a7e","type":"0"}"#,
            r#"{"data":{"alias":"Node6","client_ip":"10.0.0.2","client_port":9712,"node_ip":"10.0.0.2","node_port":9711,"services":["VALIDATOR"]},"dest":"Cv1Ehj43DDM5ttNBmC6VPpEfwXWwfGktHwjDJsTV5Fz8","identifier":"A2yZJTPHZyqJDELb8E1mhxUqWPEW5vgH2ePLTiTDQayp","txnId":"a23059dc16aaf4513f97ca91f272235e809f8bda8c40f6688b88615a2c318ff8","type":"0"}"#,
            r#"{"data":{"alias":"Node7","client_ip":"10.0.0.2","client_port":9714,"node_ip":"10.0.0.2","node_port":9713,"services":["VALIDATOR"]},"dest":"BM8dTooz5uykCbYSAAFwKNkYfT4koomBHsSWHTDtkjhW","identifier":"6pYGZXnqXLxLAhrEBhVjyvuhnV2LUgM9iw1gHds8JDqT","txnId":"e5f11aa7ec7091ca6c31a826eec885da7fcaa47611d03fdc3562b48247f179cf","type":"0"}"#,
            r#"{"data":{"alias":"Node8","client_ip":"10.0.0.2","client_port":9716,"node_ip":"10.0.0.2","node_port":9715,"services":["VALIDATOR"]},"dest":"98VysG35LxrutKTNXvhaztPFHnx5u9kHtT7PnUGqDa8x","identifier":"B4xQBURedpCS3r6v8YxTyz5RYh3Nh5Jt2MxsmtAUr1rH","txnId":"2b01e69f89514be94ebf24bfa270abbe1c5abc72415801da3f0d58e71aaa33a2","type":"0"}"#,
        ];
        let all_values: Vec<Vec<u8>> = all_str_values.iter().map(|x| String::from(*x).as_bytes().to_vec()).collect::<Vec<_>>();
        let mt_full = MerkleTree::from_vec(all_values.clone()).unwrap();
        let full_root_hash = mt_full.root_hash();
        let mut start_values = all_values.clone();
        let mut mt = MerkleTree::from_vec(start_values.drain(0..4).collect()).unwrap();

        //try to add 5th node
        let proofs_for_5: Vec<&str> = vec![
            "9fVeiDkVJ4YrNB1cy9PEeRYXE5BhxapQsGu85WZ8MyiE",
            "8p6GotiwYFiWgjMvY7KYNYcbz6hCFBJhcD9Sjo1PQANU",
            "BqHByHYX9gAHye1SoKKiLXLFB7TDntyUoMtZQjMW2w7U",
            "BhXMcoxZ9eu3Cu85bzr4G4Msrw77BT3R6Mw6P6bM9wQe"
        ];
        let proofs_for_5: Vec<Vec<u8>> = proofs_for_5.into_iter().map(|x| x.from_base58().unwrap()).collect();
        //add 5th node
        mt.append(all_values[5 - 1].clone()).unwrap();
        assert!(mt.consistency_proof(&full_root_hash, 8, &proofs_for_5).unwrap());

        //try to add 6th node
        let proofs_for_6: Vec<&str> = vec![
            "HhkWitSAXG12Ugn4KFtrUyhbZHi9XrP4jnbLuSthynSu",
            "BqHByHYX9gAHye1SoKKiLXLFB7TDntyUoMtZQjMW2w7U",
            "BhXMcoxZ9eu3Cu85bzr4G4Msrw77BT3R6Mw6P6bM9wQe"
        ];
        let proofs_for_6: Vec<Vec<u8>> = proofs_for_6.into_iter().map(|x| x.from_base58().unwrap()).collect();
        //add 6th node
        mt.append(all_values[6 - 1].clone()).unwrap();
        assert!(mt.consistency_proof(&full_root_hash, 8, &proofs_for_6).unwrap());

        //try to add 7th node
        let proofs_for_7: Vec<&str> = vec![
            "2D1aU5DeP8uPmaisGSpNoF2tNS35YhaRvfk2KPZzY2ue",
            "5cVBJRrdFraAtDzUhezeifS6W4Gsgo3TdPXs8847p95L",
            "HhkWitSAXG12Ugn4KFtrUyhbZHi9XrP4jnbLuSthynSu",
            "BhXMcoxZ9eu3Cu85bzr4G4Msrw77BT3R6Mw6P6bM9wQe"
        ];
        let proofs_for_7: Vec<Vec<u8>> = proofs_for_7.into_iter().map(|x| x.from_base58().unwrap()).collect();
        //add 7th node
        mt.append(all_values[7 - 1].clone()).unwrap();
        assert!(mt.consistency_proof(&full_root_hash, 8, &proofs_for_7).unwrap());

        //try to add 8th node, empty proof
        let proofs_for_8: Vec<Vec<u8>> = Vec::new();
        //add 7th node
        mt.append(all_values[8 - 1].clone()).unwrap();
        assert!(mt.consistency_proof(&full_root_hash, 8, &proofs_for_8).unwrap());
    }
}
*/
