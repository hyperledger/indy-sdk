extern crate libc;
extern crate serde_json;

use super::ErrorCode;
use super::sequence;

use self::libc::c_char;

use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::str::Utf8Error;
use std::sync::Mutex;

/// C types helpers
pub fn c_str_to_string<'a>(cstr: *const c_char) -> Result<Option<&'a str>, Utf8Error> {
    if cstr.is_null() {
        return Ok(None);
    }

    unsafe {
        match CStr::from_ptr(cstr).to_str() {
            Ok(str) => Ok(Some(str)),
            Err(err) => Err(err)
        }
    }
}

macro_rules! check_useful_c_str {
    ($x:ident, $e:expr) => {
        let $x = match c_str_to_string($x) {
            Ok(Some(val)) => val.to_string(),
            _ => return $e,
        };

        if $x.is_empty() {
            return $e
        }
    }
}

macro_rules! check_useful_c_byte_array {
    ($ptr:ident, $len:expr, $err1:expr, $err2:expr) => {
        if $ptr.is_null() {
            return $err1;
        }

        if $len <= 0 {
            return $err2;
        }

        let $ptr = unsafe { ::std::slice::from_raw_parts($ptr, $len as usize) };
        let $ptr = $ptr.to_vec();
    }
}

#[derive(Debug)]
struct InmemWalletContext {
    id: String
}

#[derive(Debug, Clone)]
struct InmemWalletRecord {
    type_: CString,
    id: CString,
    value: Vec<u8>,
    tags: CString
}

#[derive(Debug, Clone)]
struct InmemWalletEntity {
    metadata: CString,
    records: HashMap<String, InmemWalletRecord>
}

lazy_static! {
    static ref INMEM_WALLETS: Mutex<HashMap<String, InmemWalletEntity>> = Default::default();
}

lazy_static! {
    static ref INMEM_OPEN_WALLETS: Mutex<HashMap<i32, InmemWalletContext>> = Default::default();
}

lazy_static! {
    static ref ACTIVE_METADATAS: Mutex<HashMap<i32, CString>> = Default::default();
}

lazy_static! {
    static ref ACTIVE_RECORDS: Mutex<HashMap<i32, InmemWalletRecord>> = Default::default();
}

lazy_static! {
    static ref ACTIVE_SEARCHES: Mutex<HashMap<i32, Vec<InmemWalletRecord>,>> = Default::default();
}

pub struct InmemWallet {}

impl InmemWallet {
    pub extern "C" fn create(id: *const c_char,
                             _: *const c_char,
                             _: *const c_char,
                             metadata: *const c_char) -> ErrorCode {
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(metadata, ErrorCode::CommonInvalidStructure);

        let mut wallets = INMEM_WALLETS.lock().unwrap();
        if wallets.contains_key(&id) {
            // Invalid state as "already exists" case must be checked on service layer
            return ErrorCode::CommonInvalidState;
        }
        wallets.insert(id.clone(), InmemWalletEntity { metadata: CString::new(metadata).unwrap(), records: HashMap::new() });

        ErrorCode::Success
    }

    pub unsafe extern "C" fn open(id: *const c_char,
                           _: *const c_char,
                           _: *const c_char,
                           handle: *mut i32) -> ErrorCode {
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);

        let wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&id) {
            return ErrorCode::CommonInvalidState;
        }

        let mut handles = INMEM_OPEN_WALLETS.lock().unwrap();
        let xhandle = sequence::get_next_id();
        handles.insert(xhandle, InmemWalletContext {
            id,
        });

        *handle = xhandle;
        ErrorCode::Success
    }

    fn build_record_id(type_: &str, id: &str) -> String {
        format!("{}-{}", type_, id)
    }

    pub extern "C" fn add_record(xhandle: i32,
                                 type_: *const c_char,
                                 id: *const c_char,
                                 value: *const u8,
                                 value_len: usize,
                                 tags_json: *const c_char) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_byte_array!(value, value_len, ErrorCode::CommonInvalidStructure, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(tags_json, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        wallet.records.insert(InmemWallet::build_record_id(&type_, &id), InmemWalletRecord {
            type_: CString::new(type_).unwrap(),
            id: CString::new(id).unwrap(),
            value,
            tags: CString::new(tags_json).unwrap(),
        });
        ErrorCode::Success
    }

    pub extern "C" fn update_record_value(xhandle: i32,
                                          type_: *const c_char,
                                          id: *const c_char,
                                          joined_value: *const u8,
                                          joined_value_len: usize) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_byte_array!(joined_value, joined_value_len, ErrorCode::CommonInvalidStructure, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        match wallet.records.get_mut(&InmemWallet::build_record_id(&type_, &id)) {
            Some(ref mut record) => record.value = joined_value,
            None => return ErrorCode::WalletItemNotFound
        }

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_record(xhandle: i32,
                                 type_: *const c_char,
                                 id: *const c_char,
                                 options_json: *const c_char,
                                 handle: *mut i32) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(options_json, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get(&wallet_context.id).unwrap();

        let key = InmemWallet::build_record_id(&type_, &id);

        if !wallet.records.contains_key(&key) {
            return ErrorCode::WalletItemNotFound;
        }

        let record = wallet.records.get(&key).unwrap();

        let record_handle = sequence::get_next_id();

        let mut handles = ACTIVE_RECORDS.lock().unwrap();
        handles.insert(record_handle, record.clone());

        *handle = record_handle;

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_record_id(xhandle: i32,
                                    record_handle: i32,
                                    id_ptr: *mut *const c_char) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let handles = ACTIVE_RECORDS.lock().unwrap();

        if !handles.contains_key(&record_handle) {
            return ErrorCode::CommonInvalidState;
        }

        let record = handles.get(&record_handle).unwrap();

        *id_ptr = record.id.as_ptr();

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_record_type(xhandle: i32,
                                      record_handle: i32,
                                      type_ptr: *mut *const c_char) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let handles = ACTIVE_RECORDS.lock().unwrap();

        if !handles.contains_key(&record_handle) {
            return ErrorCode::CommonInvalidState;
        }

        let record = handles.get(&record_handle).unwrap();

        *type_ptr = record.type_.as_ptr();

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_record_value(xhandle: i32,
                                       record_handle: i32,
                                       value_ptr: *mut *const u8,
                                       value_len: *mut usize) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let handles = ACTIVE_RECORDS.lock().unwrap();

        if !handles.contains_key(&record_handle) {
            return ErrorCode::CommonInvalidState;
        }

        let record = handles.get(&record_handle).unwrap();

        // unsafe
        *value_ptr = record.value.as_ptr() as *const u8;
        *value_len = record.value.len() as usize;

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_record_tags(xhandle: i32,
                                      record_handle: i32,
                                      tags_json_ptr: *mut *const c_char) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let handles = ACTIVE_RECORDS.lock().unwrap();

        if !handles.contains_key(&record_handle) {
            return ErrorCode::CommonInvalidState;
        }

        let record = handles.get(&record_handle).unwrap();

        // unsafe
        *tags_json_ptr = record.tags.as_ptr();

        ErrorCode::Success
    }


    pub extern "C" fn free_record(xhandle: i32, record_handle: i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let mut handles = ACTIVE_RECORDS.lock().unwrap();

        if !handles.contains_key(&record_handle) {
            return ErrorCode::CommonInvalidState;
        }
        handles.remove(&record_handle);

        ErrorCode::Success
    }

    pub extern "C" fn add_record_tags(xhandle: i32,
                                      type_: *const c_char,
                                      id: *const c_char,
                                      tags_json: *const c_char) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(tags_json, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        match wallet.records.get_mut(&InmemWallet::build_record_id(&type_, &id)) {
            Some(ref mut record) => {
                let curr_tags_json =record.tags.to_str().unwrap().to_string() ;

                let new_tags_result = serde_json::from_str::<HashMap<String, String>>(&tags_json);
                let curr_tags_result = serde_json::from_str::<HashMap<String, String>>(&curr_tags_json);

                let (new_tags, mut curr_tags) = match (new_tags_result, curr_tags_result) {
                    (Ok(new), Ok(cur)) => (new, cur),
                    _ => return ErrorCode::CommonInvalidStructure
                };

                curr_tags.extend(new_tags);

                let new_tags_json = serde_json::to_string(&curr_tags).unwrap();

                record.tags = CString::new(new_tags_json).unwrap();
            }
            None => return ErrorCode::WalletItemNotFound
        }

        ErrorCode::Success
    }

    pub extern "C" fn update_record_tags(xhandle: i32,
                                         type_: *const c_char,
                                         id: *const c_char,
                                         tags_json: *const c_char) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(tags_json, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        match wallet.records.get_mut(&InmemWallet::build_record_id(&type_, &id)) {
            Some(ref mut record) => record.tags = CString::new(tags_json).unwrap(),
            None => return ErrorCode::WalletItemNotFound
        }

        ErrorCode::Success
    }

    pub extern "C" fn delete_record_tags(xhandle: i32,
                                         type_: *const c_char,
                                         id: *const c_char,
                                         tag_names: *const c_char) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(tag_names, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        match wallet.records.get_mut(&InmemWallet::build_record_id(&type_, &id)) {
            Some(ref mut record) => {
                let curr_tags_json = record.tags.to_str().unwrap().to_string() ;

                let curr_tags_res = serde_json::from_str::<HashMap<String, String>>(&curr_tags_json);
                let tags_names_to_delete = serde_json::from_str::<Vec<String>>(&tag_names);

                let (mut curr_tags, tags_delete) = match (curr_tags_res, tags_names_to_delete) {
                    (Ok(cur), Ok(to_delete)) => (cur, to_delete),
                    _ => return ErrorCode::CommonInvalidStructure
                };

                for tag_name in tags_delete {
                    curr_tags.remove(&tag_name);
                }

                let new_tags_json = serde_json::to_string(&curr_tags).unwrap();

                record.tags = CString::new(new_tags_json).unwrap()
            }
            None => return ErrorCode::WalletItemNotFound
        }

        ErrorCode::Success
    }

    pub extern "C" fn delete_record(xhandle: i32,
                                    type_: *const c_char,
                                    id: *const c_char) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);
        check_useful_c_str!(id, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        let key = InmemWallet::build_record_id(&type_, &id);

        if !wallet.records.contains_key(&key) {
            return ErrorCode::WalletItemNotFound;
        }

        wallet.records.remove(&key);

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_storage_metadata(xhandle: i32, metadata_ptr: *mut *const c_char, metadata_handle: *mut i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get(&wallet_context.id).unwrap();

        let metadata = wallet.metadata.clone();
        let metadata_pointer = metadata.as_ptr();

        let handle = sequence::get_next_id();

        let mut metadatas = ACTIVE_METADATAS.lock().unwrap();
        metadatas.insert(handle, metadata);

        // unsafe
        *metadata_ptr = metadata_pointer;
        *metadata_handle = handle;

        ErrorCode::Success
    }

    pub extern "C" fn set_storage_metadata(xhandle: i32, metadata: *const c_char) -> ErrorCode {
        check_useful_c_str!(metadata, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get_mut(&wallet_context.id).unwrap();

        wallet.metadata = CString::new(metadata).unwrap();

        ErrorCode::Success
    }

    pub extern "C" fn free_storage_metadata(xhandle: i32, metadata_handler: i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let mut handles = ACTIVE_METADATAS.lock().unwrap();

        if !handles.contains_key(&metadata_handler) {
            return ErrorCode::CommonInvalidState;
        }
        handles.remove(&metadata_handler);

        ErrorCode::Success
    }

    pub unsafe extern "C" fn search_records(xhandle: i32, type_: *const c_char, _query_json: *const c_char, _options_json: *const c_char, handle: *mut i32) -> ErrorCode {
        check_useful_c_str!(type_, ErrorCode::CommonInvalidStructure);

        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get(&wallet_context.id).unwrap();

        let search_records = wallet.records
            .iter()
            .filter(|&(key, _)| key.starts_with(&type_))
            .map(|(_, value)| value.clone())
            .collect::<Vec<InmemWalletRecord>>();

        let search_handle = sequence::get_next_id();

        let mut searches = ACTIVE_SEARCHES.lock().unwrap();

        searches.insert(search_handle, search_records);

        *handle = search_handle;

        ErrorCode::Success
    }

    pub unsafe extern "C" fn search_all_records(xhandle: i32, handle: *mut i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet_context = handles.get(&xhandle).unwrap();

        let wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&wallet_context.id) {
            return ErrorCode::CommonInvalidState;
        }

        let wallet = wallets.get(&wallet_context.id).unwrap();

        let search_records = wallet.records
            .values()
            .cloned()
            .collect::<Vec<InmemWalletRecord>>();

        let search_handle = sequence::get_next_id();

        let mut searches = ACTIVE_SEARCHES.lock().unwrap();

        searches.insert(search_handle, search_records);

        *handle = search_handle;

        ErrorCode::Success
    }

    pub unsafe extern "C" fn get_search_total_count(xhandle: i32, search_handle: i32, count: *mut usize) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let searches = ACTIVE_SEARCHES.lock().unwrap();

        match searches.get(&search_handle) {
            Some(records) => {
                // unsafe
                *count = records.len();
            }
            None => return ErrorCode::CommonInvalidState
        }

        ErrorCode::Success
    }

    pub unsafe extern "C" fn fetch_search_next_record(xhandle: i32, search_handle: i32, record_handle: *mut i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let mut searches = ACTIVE_SEARCHES.lock().unwrap();

        match searches.get_mut(&search_handle) {
            Some(records) => {
                match records.pop() {
                    Some(record) => {
                        let handle = sequence::get_next_id();

                        let mut handles = ACTIVE_RECORDS.lock().unwrap();
                        handles.insert(handle, record.clone());

                        // unsafe
                        *record_handle = handle;
                    }
                    None => return ErrorCode::WalletItemNotFound
                }
            }
            None => return ErrorCode::CommonInvalidState
        }

        ErrorCode::Success
    }

    pub extern "C" fn free_search(xhandle: i32, search_handle: i32) -> ErrorCode {
        let handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        let mut handles = ACTIVE_SEARCHES.lock().unwrap();

        if !handles.contains_key(&search_handle) {
            return ErrorCode::CommonInvalidState;
        }
        handles.remove(&search_handle);

        ErrorCode::Success
    }

    pub extern "C" fn close(xhandle: i32) -> ErrorCode {
        let mut handles = INMEM_OPEN_WALLETS.lock().unwrap();

        if !handles.contains_key(&xhandle) {
            return ErrorCode::CommonInvalidState;
        }

        handles.remove(&xhandle);
        ErrorCode::Success
    }

    pub extern "C" fn delete(name: *const c_char,
                             _: *const c_char,
                             _: *const c_char) -> ErrorCode {
        check_useful_c_str!(name, ErrorCode::CommonInvalidStructure);

        let mut wallets = INMEM_WALLETS.lock().unwrap();

        if !wallets.contains_key(&name) {
            return ErrorCode::CommonInvalidState;
        }

        wallets.remove(&name);
        ErrorCode::Success
    }

    pub fn cleanup() {
        let mut wallets = INMEM_WALLETS.lock().unwrap();
        wallets.clear();

        let mut handles = INMEM_OPEN_WALLETS.lock().unwrap();
        handles.clear();
    }
}
