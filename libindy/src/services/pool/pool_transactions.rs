use std::{fs, io};
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

use serde_json;
use serde_json::Value as SJsonValue;

use indy_api_types::errors::prelude::*;
use crate::utils::environment;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use indy_vdr::pool::PoolTransactions;

const POOL_EXT: &str = "txn";

pub fn create(pool_name: &str) -> IndyResult<PoolTransactions> {
    let mut p = environment::pool_path(pool_name);

    let mut p_stored = p.clone();
    p_stored.push("stored");
    p_stored.set_extension("btxn");

    if !p_stored.exists() {
        trace!("Restoring merkle tree from genesis");
        p.push(pool_name);
        p.set_extension(POOL_EXT);

        if !p.exists() {
            trace!("here");
            return Err(err_msg(IndyErrorKind::PoolNotCreated, format!("Pool is not created for name: {:?}", pool_name)));
        }

        _from_genesis(&p)
    } else {
        trace!("Restoring merkle tree from cache");
        _from_cache(&p_stored)
    }
}

pub fn drop_cache(pool_name: &str) -> IndyResult<()> {
    let p = get_pool_stored_path(pool_name, false);
    if p.exists() {
        warn!("Cache is invalid -- dropping it!");
        fs::remove_file(p)
            .to_indy(IndyErrorKind::IOError, "Can't drop pool ledger cache file")?;
    }
    Ok(())
}

fn _from_cache(file_name: &PathBuf) -> IndyResult<PoolTransactions> {
    let mut transactions: Vec<Vec<u8>> = Vec::new();

    let mut f = fs::File::open(file_name)
        .to_indy(IndyErrorKind::IOError, "Can't open pool ledger cache file")?;

    trace!("Start recover from cache");

    loop {
        let bytes = match f.read_u64::<LittleEndian>() {
            Ok(bytes) => bytes,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_indy(IndyErrorKind::IOError, "Can't read from pool ledger cache file"))
        };

        trace!("bytes: {:?}", bytes);
        let mut buf = vec![0; bytes as usize];

        match f.read_exact(buf.as_mut()) {
            Ok(()) => (),
            Err(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => return Err(e.to_indy(IndyErrorKind::InvalidState, "Malformed pool ledger cache file")),
                _ => return Err(e.to_indy(IndyErrorKind::IOError, "Can't read from pool ledger cache file"))
            }
        }

        transactions.push(buf.to_vec());
    }

    let transactions = PoolTransactions::from_transactions(transactions);

    Ok(transactions)
}

fn _from_genesis(pool_txns_path: &PathBuf) -> IndyResult<PoolTransactions> {
    PoolTransactions::from_file_path(&pool_txns_path)
        .map_err(|err| IndyError::from_msg(IndyErrorKind::PoolNotCreated, err.to_string()))
}

fn get_pool_stored_path(pool_name: &str, create_dir: bool) -> PathBuf {
    get_pool_stored_path_base(pool_name, create_dir, "stored", "btxn")
}

fn get_pool_stored_path_base(pool_name: &str, create_dir: bool, filename: &str, ext: &str) -> PathBuf {
    let mut path = environment::pool_path(pool_name);
    if create_dir {
        fs::create_dir_all(path.as_path()).unwrap();
    }
    path.push(filename);
    path.set_extension(ext);
    path
}

pub fn dump_new_txns<'a, I>(pool_name: &str, txns: I) -> IndyResult<()> where I: Iterator<Item = &'a Vec<u8>> {
    let p = get_pool_stored_path(pool_name, false);
    if !p.exists() {
        _dump_genesis_to_stored(&p, pool_name)?;
    }

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(p)
        .to_indy(IndyErrorKind::IOError, "Can't open pool ledger cache file")?;

    _dump_vec_to_file(txns, &mut file)
}

fn _dump_genesis_to_stored(p: &PathBuf, pool_name: &str) -> IndyResult<()> {
    let p_genesis = get_pool_stored_path_base(pool_name, false, pool_name, POOL_EXT);

    if !p_genesis.exists() {
        return Err(err_msg(IndyErrorKind::PoolNotCreated, format!("Pool is not created for name: {:?}", pool_name)));
    }

    let mut file = fs::File::create(p)
        .to_indy(IndyErrorKind::IOError, "Can't create pool ledger cache file")?;

    let genesis_vec = _genesis_to_binary(&p_genesis)?;
    _dump_vec_to_file(genesis_vec.iter(), &mut file)
}

fn _dump_vec_to_file<'a, I>(v: I, file: &mut fs::File) -> IndyResult<()> where I: Iterator<Item = &'a Vec<u8>> {
    for ref line in v {
        file.write_u64::<LittleEndian>(line.len() as u64)
            .to_indy(IndyErrorKind::IOError, "Can't write to pool ledger cache file")?;

        file.write_all(line)
            .to_indy(IndyErrorKind::IOError, "Can't write to pool ledger cache file")?;
    }

    Ok(())
}

fn _genesis_to_binary(p: &PathBuf) -> IndyResult<Vec<Vec<u8>>> {
    let f = fs::File::open(p)
        .to_indy(IndyErrorKind::IOError, "Can't open genesis txn file")?;

    let reader = io::BufReader::new(&f);
    let mut txns: Vec<Vec<u8>> = vec![];

    for line in reader.lines() {
        let line = line
            .to_indy(IndyErrorKind::IOError, "Can't read from genesis txn file")?;

        txns.push(_parse_txn_from_json(&line)?);
    }

    Ok(txns)
}

fn _parse_txn_from_json(txn: &str) -> IndyResult<Vec<u8>> {
    let txn = txn.trim();

    if txn.is_empty() {
        return Ok(vec![]);
    }

    let txn: SJsonValue = serde_json::from_str(txn)
        .to_indy(IndyErrorKind::InvalidStructure, "Genesis txn is mailformed json")?;

    rmp_serde::encode::to_vec_named(&txn)
        .to_indy(IndyErrorKind::InvalidState, "Can't encode genesis txn as message pack")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use byteorder::LittleEndian;

    use crate::utils::test;

    use super::*;
    use indy_vdr::pool::PoolBuilder;

    fn _write_genesis_txns(pool_name: &str, txns: &str) {
        let path = get_pool_stored_path_base(pool_name, true, pool_name, POOL_EXT);
        let mut f = fs::File::create(path.as_path()).unwrap();
        f.write(txns.as_bytes()).unwrap();
        f.flush().unwrap();
        f.sync_all().unwrap();
    }

    #[test]
    pub fn pool_worker_works_for_deserialize_cache() {
        test::cleanup_storage("pool_worker_works_for_deserialize_cache");
        {
            let node_txns = test::gen_txns();

            let txn1_json: serde_json::Value = serde_json::from_str(&node_txns[0]).unwrap();
            let txn2_json: serde_json::Value = serde_json::from_str(&node_txns[1]).unwrap();
            let txn3_json: serde_json::Value = serde_json::from_str(&node_txns[2]).unwrap();
            let txn4_json: serde_json::Value = serde_json::from_str(&node_txns[3]).unwrap();

            let pool_cache = vec![rmp_serde::to_vec_named(&txn1_json).unwrap(),
                                  rmp_serde::to_vec_named(&txn2_json).unwrap(),
                                  rmp_serde::to_vec_named(&txn3_json).unwrap(),
                                  rmp_serde::to_vec_named(&txn4_json).unwrap()];

            let pool_name = "pool_worker_works_for_deserialize_cache";
            let path = get_pool_stored_path(pool_name, true);
            let mut f = fs::File::create(path.as_path()).unwrap();
            pool_cache.iter().for_each(|vec| {
                f.write_u64::<LittleEndian>(vec.len() as u64).unwrap();
                f.write_all(vec).unwrap();
            });

            let transactions = create(pool_name).unwrap();

            PoolBuilder::default().transactions(transactions).unwrap().into_shared().unwrap();
        }
        test::cleanup_storage("pool_worker_works_for_deserialize_cache");
    }

    #[test]
    fn pool_worker_restore_merkle_tree_works_from_genesis_txns() {
        test::cleanup_storage("pool_worker_restore_merkle_tree_works_from_genesis_txns");

        let node_txns = test::gen_txns();
        let txns_src = format!("{}\n{}",
                               node_txns[0].replace(environment::test_pool_ip().as_str(), "10.0.0.2"),
                               node_txns[1].replace(environment::test_pool_ip().as_str(), "10.0.0.2"));
        _write_genesis_txns("pool_worker_restore_merkle_tree_works_from_genesis_txns", &txns_src);

        let pool_transactions = create("pool_worker_restore_merkle_tree_works_from_genesis_txns").unwrap();
        let mt = pool_transactions.merkle_tree().unwrap();

        assert_eq!(mt.count(), 2);
        assert_eq!(mt.root_hash_hex(), "c715aef44aaacab8746c9a505ba106b5554fe6d29ec7f0a2abc9d7723fdea523", "test restored MT root hash");

        test::cleanup_storage("pool_worker_restore_merkle_tree_works_from_genesis_txns");
    }
}
