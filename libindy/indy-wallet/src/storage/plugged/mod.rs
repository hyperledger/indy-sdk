use std::{slice, str};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;

use libc::c_char;
use serde_json;

use indy_api_types::{ErrorCode, SearchHandle, INVALID_SEARCH_HANDLE};
use indy_api_types::wallet::*;
use indy_api_types::errors::prelude::*;
use crate::language;
use indy_utils::crypto::base64;

use super::{EncryptedValue, StorageIterator, StorageRecord, Tag, TagName, WalletStorage, WalletStorageType};
use super::super::{RecordOptions, SearchOptions};

#[derive(Debug, Deserialize)]
pub struct PluggedWalletJSONValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PluggedWalletJSONValues {
    pub values: Vec<PluggedWalletJSONValue>
}

// This struct is used as a helper to free the resource even in case of error.
// It is workaround for Rust's lack of try/catch.
struct ResourceGuard {
    storage_handle: i32,
    item_handle: i32,
    free_handler: extern fn(storage_handle: i32, item_handle: i32) -> ErrorCode,
}

impl ResourceGuard {
    fn new(storage_handle: i32,
           item_handle: i32,
           free_handler: extern fn(s_handle: i32, i_handle: i32) -> ErrorCode,
    ) -> Self {
        Self {
            storage_handle,
            item_handle,
            free_handler,
        }
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        (self.free_handler)(self.storage_handle, self.item_handle);
    }
}


#[derive(PartialEq, Debug)]
struct PluggedStorageIterator {
    storage_handle: i32,
    search_handle: SearchHandle,
    options: SearchOptions,
    fetch_search_next_record_handler: WalletFetchSearchNextRecord,
    get_search_total_count_handler: WalletGetSearchTotalCount,
    get_record_type_handler: WalletGetRecordType,
    get_record_id_handler: WalletGetRecordId,
    get_record_value_handler: WalletGetRecordValue,
    get_record_tags_handler: WalletGetRecordTags,
    free_record_handler: WalletFreeRecord,
    free_search_handler: WalletFreeSearch,
}

impl PluggedStorageIterator {
    fn new(storage: &PluggedStorage, search_handle: SearchHandle, options: SearchOptions) -> Self {
        Self {
            storage_handle: storage.handle,
            search_handle,
            options,
            fetch_search_next_record_handler: storage.fetch_search_next_record_handler,
            get_search_total_count_handler: storage.get_search_total_count_handler,
            get_record_type_handler: storage.get_record_type_handler,
            get_record_id_handler: storage.get_record_id_handler,
            get_record_value_handler: storage.get_record_value_handler,
            get_record_tags_handler: storage.get_record_tags_handler,
            free_record_handler: storage.free_record_handler,
            free_search_handler: storage.free_search_handler,
        }
    }
}

impl StorageIterator for PluggedStorageIterator {
    fn next(&mut self) -> IndyResult<Option<StorageRecord>> {
        let mut record_handle = -1;

        let err = (self.fetch_search_next_record_handler)(self.storage_handle,
                                                          self.search_handle.0,
                                                          &mut record_handle);

        if err == ErrorCode::WalletItemNotFound {
            return Ok(None);
        } else if err != ErrorCode::Success {
            return Err(err.into());
        }

        let _record_free_helper = ResourceGuard::new(self.storage_handle, record_handle, self.free_record_handler);

        let type_ = if self.options.retrieve_type {
            let mut type_ptr: *const c_char = ptr::null_mut();

            let err = (self.get_record_type_handler)(self.storage_handle,
                                                     record_handle,
                                                     &mut type_ptr);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            Some(
                base64::decode(unsafe {
                    CStr::from_ptr(type_ptr)
                        .to_str()
                        .to_indy(IndyErrorKind::InvalidState, "Record type contains non-utf8 symbol")?
                })?
            )
        } else {
            None
        };

        let id = {
            let mut id_ptr: *const c_char = ptr::null_mut();

            let err = (self.get_record_id_handler)(self.storage_handle,
                                                   record_handle,
                                                   &mut id_ptr);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            base64::decode(unsafe {
                CStr::from_ptr(id_ptr)
                    .to_str()
                    .to_indy(IndyErrorKind::InvalidState, "Record id contains non-utf8 symbol")?
            })?
        };

        let value = if self.options.retrieve_value {
            let mut value_bytes: *const u8 = ptr::null();
            let mut value_bytes_len: usize = 0;
            let err = (self.get_record_value_handler)(self.storage_handle,
                                                      record_handle,
                                                      &mut value_bytes,
                                                      &mut value_bytes_len);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            let value = unsafe { slice::from_raw_parts(value_bytes, value_bytes_len) };
            Some(EncryptedValue::from_bytes(value)?)
        } else { None };

        let tags = if self.options.retrieve_tags {
            let mut tags_ptr: *const c_char = ptr::null_mut();
            let err = (self.get_record_tags_handler)(self.storage_handle,
                                                     record_handle,
                                                     &mut tags_ptr);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            let tags_json = unsafe {
                CStr::from_ptr(tags_ptr)
                    .to_str()
                    .to_indy(IndyErrorKind::InvalidState, "Tags contains non-utf8 symbol")?
            };

            Some(_tags_from_json(tags_json)?)
        } else { None };

        Ok(Some(StorageRecord {
            type_,
            id,
            value,
            tags,
        }))
    }

    fn get_total_count(&self) -> IndyResult<Option<usize>> {
        let mut total_count = 0;

        if self.options.retrieve_total_count {
            let err = (self.get_search_total_count_handler)(self.storage_handle,
                                                            self.search_handle.0,
                                                            &mut total_count);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            Ok(Some(total_count))
        } else {
            Ok(None)
        }
    }
}

impl Drop for PluggedStorageIterator {
    fn drop(&mut self) {
        (self.free_search_handler)(self.storage_handle, self.search_handle.0);
    }
}

#[derive(PartialEq, Debug)]
struct PluggedStorage {
    handle: i32,
    add_record_handler: WalletAddRecord,
    update_record_value_handler: WalletUpdateRecordValue,
    update_record_tags_handler: WalletUpdateRecordTags,
    add_record_tags_handler: WalletAddRecordTags,
    delete_record_tags_handler: WalletDeleteRecordTags,
    delete_record_handler: WalletDeleteRecord,
    get_record_handler: WalletGetRecord,
    get_record_id_handler: WalletGetRecordId,
    get_record_type_handler: WalletGetRecordType,
    get_record_value_handler: WalletGetRecordValue,
    get_record_tags_handler: WalletGetRecordTags,
    free_record_handler: WalletFreeRecord,
    get_storage_metadata_handler: WalletGetStorageMetadata,
    set_storage_metadata_handler: WalletSetStorageMetadata,
    free_storage_metadata_handler: WalletFreeStorageMetadata,
    search_records_handler: WalletSearchRecords,
    search_all_records_handler: WalletSearchAllRecords,
    get_search_total_count_handler: WalletGetSearchTotalCount,
    fetch_search_next_record_handler: WalletFetchSearchNextRecord,
    free_search_handler: WalletFreeSearch,
    close_handler: WalletClose,
}

impl PluggedStorage {
    fn new(handle: i32,
           add_record_handler: WalletAddRecord,
           update_record_value_handler: WalletUpdateRecordValue,
           update_record_tags_handler: WalletUpdateRecordTags,
           add_record_tags_handler: WalletAddRecordTags,
           delete_record_tags_handler: WalletDeleteRecordTags,
           delete_record_handler: WalletDeleteRecord,
           get_record_handler: WalletGetRecord,
           get_record_id_handler: WalletGetRecordId,
           get_record_type_handler: WalletGetRecordType,
           get_record_value_handler: WalletGetRecordValue,
           get_record_tags_handler: WalletGetRecordTags,
           free_record_handler: WalletFreeRecord,
           get_storage_metadata_handler: WalletGetStorageMetadata,
           set_storage_metadata_handler: WalletSetStorageMetadata,
           free_storage_metadata_handler: WalletFreeStorageMetadata,
           search_records_handler: WalletSearchRecords,
           search_all_records_handler: WalletSearchAllRecords,
           get_search_total_count_handler: WalletGetSearchTotalCount,
           fetch_search_next_record_handler: WalletFetchSearchNextRecord,
           free_search_handler: WalletFreeSearch,
           close_handler: WalletClose) -> PluggedStorage {
        PluggedStorage {
            handle,
            add_record_handler,
            update_record_value_handler,
            update_record_tags_handler,
            add_record_tags_handler,
            delete_record_tags_handler,
            delete_record_handler,
            get_record_handler,
            get_record_id_handler,
            get_record_type_handler,
            get_record_value_handler,
            get_record_tags_handler,
            free_record_handler,
            get_storage_metadata_handler,
            set_storage_metadata_handler,
            free_storage_metadata_handler,
            search_records_handler,
            search_all_records_handler,
            get_search_total_count_handler,
            fetch_search_next_record_handler,
            free_search_handler,
            close_handler,
        }
    }
}

fn _tags_to_json(tags: &[Tag]) -> IndyResult<String> {
    let mut string_tags = HashMap::with_capacity(tags.len());

    for tag in tags {
        match *tag {
            Tag::Encrypted(ref name, ref value) => string_tags.insert(base64::encode(&name), base64::encode(&value)),
            Tag::PlainText(ref name, ref value) => string_tags.insert(format!("~{}", &base64::encode(&name)), value.to_string()),
        };
    }

    serde_json::to_string(&string_tags)
        .to_indy(IndyErrorKind::InvalidState, "Unable to serialize tags as json")
}

fn _tags_from_json(json: &str) -> IndyResult<Vec<Tag>> {
    let string_tags: HashMap<String, String> = serde_json::from_str(json)
        .to_indy(IndyErrorKind::InvalidState, "Unable to deserialize tags from json")?;

    let mut tags = Vec::with_capacity(string_tags.len());

    for (k, v) in string_tags {
        if k.starts_with('~') {
            let mut key = k;
            key.remove(0);
            tags.push(
                Tag::PlainText(
                    base64::decode(&key).to_indy(IndyErrorKind::InvalidState, "Unable to decode tag key from base64")?,
                    v,
                )
            );
        } else {
            tags.push(
                Tag::Encrypted(
                    base64::decode(&k).to_indy(IndyErrorKind::InvalidState, "Unable to decode tag key from base64")?,
                    base64::decode(&v).to_indy(IndyErrorKind::InvalidState, "Unable to decode tag value from base64")?,
                )
            );
        }
    }
    Ok(tags)
}

fn _tags_names_to_json(tag_names: &[TagName]) -> IndyResult<String> {
    let tags : Vec<String> = tag_names.iter().map(|tag_name|
        match *tag_name {
            TagName::OfEncrypted(ref tag_name) => base64::encode(tag_name),
            TagName::OfPlain(ref tag_name) => format!("~{}", base64::encode(tag_name))
        }).collect();

    serde_json::to_string(&tags)
        .to_indy(IndyErrorKind::InvalidState, "Unable to serialize tag names as json")
}

impl WalletStorage for PluggedStorage {
    fn get(&self, type_: &[u8], id: &[u8], options: &str) -> IndyResult<StorageRecord> {
        let type_cstr = CString::new(base64::encode(type_))?;
        let id_cstr = CString::new(base64::encode(id))?;
        let options_cstr = CString::new(options)?;

        let mut record_handle: i32 = -1;

        let options: RecordOptions = serde_json::from_str(options)
            .to_indy(IndyErrorKind::InvalidStructure, "RecordRetrieveOptions is malformed json")?;

        let err = (self.get_record_handler)(self.handle,
                                            type_cstr.as_ptr(),
                                            id_cstr.as_ptr(),
                                            options_cstr.as_ptr(),
                                            &mut record_handle);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        let _record_free_helper = ResourceGuard::new(self.handle, record_handle, self.free_record_handler);

        let value = if options.retrieve_value {
            let mut value_bytes: *const u8 = ptr::null();
            let mut value_bytes_len: usize = 0;
            let err = (self.get_record_value_handler)(self.handle,
                                                      record_handle,
                                                      &mut value_bytes,
                                                      &mut value_bytes_len);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            let value = unsafe { slice::from_raw_parts(value_bytes, value_bytes_len) };
            Some(EncryptedValue::from_bytes(value)?)
        } else { None };

        let tags = if options.retrieve_tags {
            let mut tags_ptr: *const c_char = ptr::null_mut();
            let err = (self.get_record_tags_handler)(self.handle,
                                                     record_handle,
                                                     &mut tags_ptr);

            if err != ErrorCode::Success {
                return Err(err.into());
            }

            let tags_json = unsafe {
                CStr::from_ptr(tags_ptr)
                    .to_str()
                    .to_indy(IndyErrorKind::InvalidState, "Tags contains non-utf8 symbol")?
            };

            Some(_tags_from_json(tags_json)?)
        } else { None };

        let result = StorageRecord {
            id: id.to_owned(),
            type_: if options.retrieve_type { Some(type_.to_vec()) } else { None },
            value,
            tags,
        };

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(result)
    }

    fn add(&self, type_: &[u8], id: &[u8], value: &EncryptedValue, tags: &[Tag]) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;
        let joined_value = value.to_bytes();
        let tags = CString::new(_tags_to_json(&tags)?)?;

        let err = (self.add_record_handler)(self.handle,
                                            type_.as_ptr(),
                                            id.as_ptr(),
                                            joined_value.as_ptr(),
                                            joined_value.len(),
                                            tags.as_ptr());

        if err == ErrorCode::WalletItemAlreadyExists {
            return Err(err_msg(IndyErrorKind::WalletItemAlreadyExists, "Wallet item already exists"));
        } else if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn add_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;
        let tags = CString::new(_tags_to_json(&tags)?)?;

        let err = (self.add_record_tags_handler)(self.handle,
                                                 type_.as_ptr(),
                                                 id.as_ptr(),
                                                 tags.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn update_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;
        let tags = CString::new(_tags_to_json(&tags)?)?;

        let err = (self.update_record_tags_handler)(self.handle,
                                                    type_.as_ptr(),
                                                    id.as_ptr(),
                                                    tags.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn delete_tags(&self, type_: &[u8], id: &[u8], tag_names: &[TagName]) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;
        let tag_names = CString::new(_tags_names_to_json(tag_names)?)?;

        let err = (self.delete_record_tags_handler)(self.handle,
                                                    type_.as_ptr(),
                                                    id.as_ptr(),
                                                    tag_names.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn update(&self, type_: &[u8], id: &[u8], value: &EncryptedValue) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;
        let joined_value = value.to_bytes();

        let err = (self.update_record_value_handler)(self.handle,
                                                     type_.as_ptr(),
                                                     id.as_ptr(),
                                                     joined_value.as_ptr(),
                                                     joined_value.len());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn delete(&self, type_: &[u8], id: &[u8]) -> IndyResult<()> {
        let type_ = CString::new(base64::encode(type_))?;
        let id = CString::new(base64::encode(id))?;

        let err = (self.delete_record_handler)(self.handle,
                                               type_.as_ptr(),
                                               id.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn get_storage_metadata(&self) -> IndyResult<Vec<u8>> {
        let mut metadata_ptr: *const c_char = ptr::null_mut();
        let mut metadata_handle = -1;

        let err: ErrorCode = (self.get_storage_metadata_handler)(self.handle,
                                                                 &mut metadata_ptr,
                                                                 &mut metadata_handle);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        let _metadata_free_helper = ResourceGuard::new(self.handle, metadata_handle, self.free_storage_metadata_handler);

        let metadata = base64::decode(unsafe {
            CStr::from_ptr(metadata_ptr)
                .to_str()
                .to_indy(IndyErrorKind::InvalidState, "Metadata contains non-utf8 symbol")?
        })?;

        Ok(metadata)
    }

    fn set_storage_metadata(&self, metadata: &[u8]) -> IndyResult<()> {
        let metadata = CString::new(base64::encode(metadata))?;

        let err = (self.set_storage_metadata_handler)(self.handle, metadata.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn get_all(&self) -> IndyResult<Box<dyn StorageIterator>> {
        let mut search_handle: SearchHandle = INVALID_SEARCH_HANDLE;

        let err = (self.search_all_records_handler)(self.handle, &mut search_handle.0);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(Box::new(
            PluggedStorageIterator::new(
                &self,
                search_handle,
                SearchOptions {
                    retrieve_records: true,
                    retrieve_total_count: false,
                    retrieve_type: true,
                    retrieve_value: true,
                    retrieve_tags: true,
                },
            )
        ))
    }

    fn search(&self, type_: &[u8], query: &language::Operator, options: Option<&str>) -> IndyResult<Box<dyn StorageIterator>> {
        let type_ = CString::new(base64::encode(type_))?;
        let query = CString::new(query.to_string())?;
        let options_cstr = CString::new(options.unwrap_or("{}"))?;

        let options: SearchOptions = serde_json::from_str(options.unwrap_or("{}"))
            .to_indy(IndyErrorKind::InvalidStructure, "Search options is malformed json")?;

        let mut search_handle: SearchHandle = INVALID_SEARCH_HANDLE;

        let err = (self.search_records_handler)(self.handle,
                                                type_.as_ptr(),
                                                query.as_ptr(),
                                                options_cstr.as_ptr(),
                                                &mut search_handle.0);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(Box::new(
            PluggedStorageIterator::new(
                &self,
                search_handle,
                options,
            )
        ))
    }

    fn close(&mut self) -> IndyResult<()> {
        let err = (self.close_handler)(self.handle);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        // invalidate the handle, just in case.
        self.handle = -1;

        Ok(())
    }
}

impl Drop for PluggedStorage {
    fn drop(&mut self) {
        // if storage is not closed, close it before drop.
        if self.handle >= 0 {
            self.close().unwrap();
        }
    }
}

#[derive(Debug)]
pub struct PluggedStorageType {
    create_handler: WalletCreate,
    open_handler: WalletOpen,
    close_handler: WalletClose,
    delete_handler: WalletDelete,
    add_record_handler: WalletAddRecord,
    update_record_value_handler: WalletUpdateRecordValue,
    update_record_tags_handler: WalletUpdateRecordTags,
    add_record_tags_handler: WalletAddRecordTags,
    delete_record_tags_handler: WalletDeleteRecordTags,
    delete_record_handler: WalletDeleteRecord,
    get_record_handler: WalletGetRecord,
    get_record_id_handler: WalletGetRecordId,
    get_record_type_handler: WalletGetRecordType,
    get_record_value_handler: WalletGetRecordValue,
    get_record_tags_handler: WalletGetRecordTags,
    free_record_handler: WalletFreeRecord,
    get_storage_metadata_handler: WalletGetStorageMetadata,
    set_storage_metadata_handler: WalletSetStorageMetadata,
    free_storage_metadata_handler: WalletFreeStorageMetadata,
    search_records_handler: WalletSearchRecords,
    search_all_records_handler: WalletSearchAllRecords,
    get_search_total_count_handler: WalletGetSearchTotalCount,
    fetch_search_next_record_handler: WalletFetchSearchNextRecord,
    free_search_handler: WalletFreeSearch,
}


impl PluggedStorageType {
    pub fn new(create_handler: WalletCreate,
               open_handler: WalletOpen,
               close_handler: WalletClose,
               delete_handler: WalletDelete,
               add_record_handler: WalletAddRecord,
               update_record_value_handler: WalletUpdateRecordValue,
               update_record_tags_handler: WalletUpdateRecordTags,
               add_record_tags_handler: WalletAddRecordTags,
               delete_record_tags_handler: WalletDeleteRecordTags,
               delete_record_handler: WalletDeleteRecord,
               get_record_handler: WalletGetRecord,
               get_record_id_handler: WalletGetRecordId,
               get_record_type_handler: WalletGetRecordType,
               get_record_value_handler: WalletGetRecordValue,
               get_record_tags_handler: WalletGetRecordTags,
               free_record_handler: WalletFreeRecord,
               get_storage_metadata_handler: WalletGetStorageMetadata,
               set_storage_metadata_handler: WalletSetStorageMetadata,
               free_storage_metadata_handler: WalletFreeStorageMetadata,
               search_records_handler: WalletSearchRecords,
               search_all_records_handler: WalletSearchAllRecords,
               get_search_total_count_handler: WalletGetSearchTotalCount,
               fetch_search_next_record_handler: WalletFetchSearchNextRecord,
               free_search_handler: WalletFreeSearch) -> PluggedStorageType {
        PluggedStorageType {
            create_handler,
            open_handler,
            close_handler,
            delete_handler,
            add_record_handler,
            update_record_value_handler,
            update_record_tags_handler,
            add_record_tags_handler,
            delete_record_tags_handler,
            delete_record_handler,
            get_record_handler,
            get_record_id_handler,
            get_record_type_handler,
            get_record_value_handler,
            get_record_tags_handler,
            free_record_handler,
            get_storage_metadata_handler,
            set_storage_metadata_handler,
            free_storage_metadata_handler,
            search_records_handler,
            search_all_records_handler,
            get_search_total_count_handler,
            fetch_search_next_record_handler,
            free_search_handler,
        }
    }
}

impl WalletStorageType for PluggedStorageType {
    fn create_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>, metadata: &[u8]) -> IndyResult<()> {
        let name = CString::new(id)?;
        let metadata = CString::new(base64::encode(metadata))?;

        let config = config
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let credentials = credentials
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let err = (self.create_handler)(name.as_ptr(),
                                        config.as_ref().map_or(ptr::null(), |x| x.as_ptr()),
                                        credentials.as_ref().map_or(ptr::null(), |x| x.as_ptr()),
                                        metadata.as_ptr());

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }

    fn open_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> IndyResult<Box<dyn WalletStorage>> {
        let mut handle: i32 = -1;
        let id = CString::new(id)?;

        let config = config
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let credentials = credentials
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let err = (self.open_handler)(id.as_ptr(),
                                      config.as_ref().map_or(ptr::null(), |x| x.as_ptr()),
                                      credentials.as_ref().map_or(ptr::null(), |x| x.as_ptr()),
                                      &mut handle);

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(Box::new(
            PluggedStorage::new(
                handle,
                self.add_record_handler,
                self.update_record_value_handler,
                self.update_record_tags_handler,
                self.add_record_tags_handler,
                self.delete_record_tags_handler,
                self.delete_record_handler,
                self.get_record_handler,
                self.get_record_id_handler,
                self.get_record_type_handler,
                self.get_record_value_handler,
                self.get_record_tags_handler,
                self.free_record_handler,
                self.get_storage_metadata_handler,
                self.set_storage_metadata_handler,
                self.free_storage_metadata_handler,
                self.search_records_handler,
                self.search_all_records_handler,
                self.get_search_total_count_handler,
                self.fetch_search_next_record_handler,
                self.free_search_handler,
                self.close_handler)))
    }

    fn delete_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> IndyResult<()> {
        let id = CString::new(id)?;

        let config = config
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let credentials = credentials
            .map(CString::new)
            .map_or(Ok(None), |r| r.map(Some))?;

        let err = (self.delete_handler)(id.as_ptr(),
                                        config.as_ref().map_or(ptr::null(), |x| x.as_ptr()),
                                        credentials.as_ref().map_or(ptr::null(), |x| x.as_ptr()));

        if err != ErrorCode::Success {
            return Err(err.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use std::clone::Clone;
    use std::sync::RwLock;

    use indy_api_types::ErrorCode;

    use super::*;

    use self::rand::{thread_rng, Rng};
    use rand::distributions::{Alphanumeric, Standard};
    use std::iter;

    impl PartialEq for StorageRecord {
        fn eq(&self, other: &StorageRecord) -> bool {
            self.id == other.id &&
                self.type_ == other.type_ &&
                self.value == other.value &&
                match (&self.tags, &other.tags) {
                    (&Some(ref tags1), &Some(ref tags2)) => {
                        let mut tags1 = tags1.clone();
                        let mut tags2 = tags2.clone();
                        tags1.sort_unstable();
                        tags2.sort_unstable();

                        tags1 == tags2
                    }
                    (&None, &None) => true,
                    (_, _) => false,
                }
        }
    }

    #[derive(PartialEq, Debug)]
    enum Call {
        CreateHandler(Option<String>, Option<String>, Option<String>, Option<String>),
        OpenHandler(Option<String>, Option<String>, Option<String>),
        CloseHandler(i32),
        DeleteHandler(Option<String>, Option<String>, Option<String>),
        AddRecordHandler(i32, Option<String>, Option<String>, Vec<u8>, HashMap<String, serde_json::Value>),
        UpdateRecordValueHandler(i32, Option<String>, Option<String>, Vec<u8>),
        UpdateRecordTagsHandler(i32, Option<String>, Option<String>, HashMap<String, serde_json::Value>),
        AddRecordTagsHandler(i32, Option<String>, Option<String>, HashMap<String, serde_json::Value>),
        DeleteRecordTagsHandler(i32, Option<String>, Option<String>, Option<String>),
        DeleteRecordHandler(i32, Option<String>, Option<String>),
        GetRecordHandler(i32, Option<String>, Option<String>, Option<String>),
        GetRecordIdHandler(i32, i32),
        GetRecordTypeHandler(i32, i32),
        GetRecordValueHandler(i32, i32),
        GetRecordTagsHandler(i32, i32),
        FreeRecordHandler(i32, i32),
        GetStorageMetadataHandler(i32),
        SetStorageMetadataHandler(i32, Option<String>),
        FreeStorageMetadataHandler(i32, i32),
        SearchRecordsHandler(i32, Option<String>, Option<String>, Option<String>),
        SearchAllRecordsHandler(i32),
        GetSearchTotalCountHandler(i32, i32),
        FetchSearchNextRecordHandler(i32, i32),
        FreeSearchHandler(i32, i32),
    }

    fn _random_vector(len: usize) -> Vec<u8> {
        thread_rng().sample_iter(&Standard).take(len).collect()
    }

    fn _random_string(len: usize) -> String {
        iter::repeat(())
            .map(|()| thread_rng().sample(Alphanumeric))
            .map(char::from)
            .take(len)
            .collect()
    }

    lazy_static!(
        static ref DEBUG_VEC: RwLock<Vec<Call>> = RwLock::new(Vec::new());
        static ref RETURN_TYPE: RwLock<(CString, Vec<u8>)> = RwLock::new({
            let data = _random_vector(32);
            let str = CString::new(
                base64::encode(&data)
            ).unwrap();
            (str, data)
        });
        static ref RETURN_ID: RwLock<(CString, Vec<u8>)> = RwLock::new({
            let data = _random_vector(64);
            let str = CString::new(
                base64::encode(&data)
            ).unwrap();
            (str, data)
        });
        static ref RETURN_VALUE: RwLock<(Vec<u8>, EncryptedValue)> = RwLock::new({
            let value = EncryptedValue{data:_random_vector(512), key:_random_vector(60)};
            let vec = value.to_bytes();
            (vec, value)
        });
        static ref RETURN_TAGS: RwLock<(CString, Vec<Tag>)> = RwLock::new({
            let mut tags = Vec::new();
            tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));
            tags.push(Tag::PlainText(_random_vector(32), _random_string(64)));
            tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));
            let tags_json = CString::new(_tags_to_json(&tags).unwrap()).unwrap();
            (tags_json, tags)
        });
        static ref RETURN_METADATA: RwLock<(CString, Vec<u8>)> = RwLock::new({
            let data = _random_vector(512);
            let str = CString::new(
                base64::encode(&data)
            ).unwrap();
            (str, data)
        });
    );

    static RETURN_STORAGE_HANDLE: i32 = 1i32;
    static RETURN_RECORD_HANDLE: i32 = 2i32;
    static RETURN_SEARCH_HANDLE: i32 = 3i32;
    static RETURN_METADATA_HANDLE: i32 = 4i32;
    static RETURN_SEARCH_TOTAL_COUNT: usize = 1024;

    fn _convert_c_string(str: *const c_char) -> Option<String> {
        if str != ptr::null() {
            Some(unsafe { CStr::from_ptr(str).to_str().unwrap() }.to_string())
        } else {
            None
        }
    }

    fn _convert_c_array(ptr: *const u8, len: usize) -> Vec<u8> {
        unsafe { slice::from_raw_parts(ptr, len) }.to_owned()
    }

    extern "C" fn _mock_create_handler(id: *const c_char,
                                       config: *const c_char,
                                       credentials: *const c_char,
                                       metadata: *const c_char) -> ErrorCode {
        assert_ne!(id, ptr::null());
        assert_ne!(credentials, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::CreateHandler(
                _convert_c_string(id),
                _convert_c_string(config),
                _convert_c_string(credentials),
                _convert_c_string(metadata),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_open_handler(id: *const c_char,
                                     config: *const c_char,
                                     credentials: *const c_char,
                                     storage_handle_p: *mut i32) -> ErrorCode {
        assert_ne!(id, ptr::null());
        assert_ne!(credentials, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::OpenHandler(
                _convert_c_string(id),
                _convert_c_string(config),
                _convert_c_string(credentials),
            )
        );

        unsafe { *storage_handle_p = 1; }

        ErrorCode::Success
    }

    extern "C" fn _mock_close_handler(storage_handle: i32) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::CloseHandler(storage_handle)
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_delete_handler(id: *const c_char,
                                       config: *const c_char,
                                       credentials: *const c_char) -> ErrorCode {
        assert_ne!(id, ptr::null());
        assert_ne!(credentials, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::DeleteHandler(
                _convert_c_string(id),
                _convert_c_string(config),
                _convert_c_string(credentials),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_add_record_handler(storage_handle: i32,
                                           type_: *const c_char,
                                           id: *const c_char,
                                           value: *const u8,
                                           value_len: usize,
                                           tags_json: *const c_char) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());
        assert_ne!(value, ptr::null());
        assert_ne!(tags_json, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::AddRecordHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                _convert_c_array(value, value_len),
                serde_json::from_str(&_convert_c_string(tags_json).unwrap()).unwrap(),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_update_record_value_handler(storage_handle: i32,
                                                    type_: *const c_char,
                                                    id: *const c_char,
                                                    value: *const u8,
                                                    value_len: usize) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());
        assert_ne!(value, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::UpdateRecordValueHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                _convert_c_array(value, value_len),
            )
        );


        ErrorCode::Success
    }

    extern "C" fn _mock_update_record_tags_handler(storage_handle: i32,
                                                   type_: *const c_char,
                                                   id: *const c_char,
                                                   tags_json: *const c_char) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());
        assert_ne!(tags_json, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::UpdateRecordTagsHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                serde_json::from_str(&_convert_c_string(tags_json).unwrap()).unwrap(),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_add_record_tags_handler(storage_handle: i32,
                                                type_: *const c_char,
                                                id: *const c_char,
                                                tags_json: *const c_char) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());
        assert_ne!(tags_json, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::AddRecordTagsHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                serde_json::from_str(&_convert_c_string(tags_json).unwrap()).unwrap(),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_delete_record_tags_handler(storage_handle: i32,
                                                   type_: *const c_char,
                                                   id: *const c_char,
                                                   tag_names_json: *const c_char) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());
        assert_ne!(tag_names_json, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::DeleteRecordTagsHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                _convert_c_string(tag_names_json),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_delete_record_handler(storage_handle: i32,
                                              type_: *const c_char,
                                              id: *const c_char) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::DeleteRecordHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_get_record_handler(storage_handle: i32,
                                           type_: *const c_char,
                                           id: *const c_char,
                                           options_json: *const c_char,
                                           record_handle_p: *mut i32) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(id, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::GetRecordHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(id),
                _convert_c_string(options_json),
            )
        );

        unsafe { *record_handle_p = 2; }

        ErrorCode::Success
    }

    extern "C" fn _mock_get_record_id_handler(storage_handle: i32,
                                              record_handle: i32,
                                              record_id_p: *mut *const c_char) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::GetRecordIdHandler(
                storage_handle,
                record_handle,
            )
        );

        unsafe { *record_id_p = RETURN_ID.read().unwrap().0.as_ptr(); }

        ErrorCode::Success
    }

    extern "C" fn _mock_get_record_type_handler(storage_handle: i32,
                                                record_handle: i32,
                                                record_type_p: *mut *const c_char) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::GetRecordTypeHandler(
                storage_handle,
                record_handle,
            )
        );

        unsafe { *record_type_p = RETURN_TYPE.read().unwrap().0.as_ptr(); }

        ErrorCode::Success
    }

    extern "C" fn _mock_get_record_value_handler(storage_handle: i32,
                                                 record_handle: i32,
                                                 record_value_p: *mut *const u8,
                                                 record_value_len_p: *mut usize) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::GetRecordValueHandler(
                storage_handle,
                record_handle,
            )
        );

        unsafe {
            let return_value = RETURN_VALUE.read().unwrap();
            *record_value_p = return_value.0.as_ptr();
            *record_value_len_p = return_value.0.len();
        }

        ErrorCode::Success
    }

    extern "C" fn _mock_get_record_tags_handler(storage_handle: i32,
                                                record_handle: i32,
                                                record_tags_p: *mut *const c_char) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::GetRecordTagsHandler(
                storage_handle,
                record_handle,
            )
        );

        unsafe { *record_tags_p = RETURN_TAGS.read().unwrap().0.as_ptr(); }

        ErrorCode::Success
    }

    extern "C" fn _mock_get_storage_metadata_handler(storage_handle: i32,
                                                     metadata_p: *mut *const c_char,
                                                     metadata_handle: *mut i32) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::GetStorageMetadataHandler(
                storage_handle,
            )
        );

        unsafe {
            *metadata_p = RETURN_METADATA.read().unwrap().0.as_ptr();
            *metadata_handle = RETURN_METADATA_HANDLE;
        }

        ErrorCode::Success
    }

    extern "C" fn _mock_set_storage_metadata_handler(storage_handle: i32,
                                                     metadata_p: *const c_char) -> ErrorCode {
        assert_ne!(metadata_p, ptr::null());

        DEBUG_VEC.write().unwrap().push(
            Call::SetStorageMetadataHandler(
                storage_handle,
                _convert_c_string(metadata_p),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_free_storage_metadata_handler(storage_handle: i32,
                                                      metadata_handle: i32) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::FreeStorageMetadataHandler(
                storage_handle,
                metadata_handle,
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_free_record_handler(storage_handle: i32,
                                            record_handle: i32) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::FreeRecordHandler(
                storage_handle,
                record_handle,
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_search_records_handler(storage_handle: i32,
                                               type_: *const c_char,
                                               query_json: *const c_char,
                                               options_json: *const c_char,
                                               search_handle_p: *mut i32) -> ErrorCode {
        assert_ne!(type_, ptr::null());
        assert_ne!(query_json, ptr::null());
        assert_ne!(options_json, ptr::null());

        unsafe { *search_handle_p = RETURN_SEARCH_HANDLE; }

        DEBUG_VEC.write().unwrap().push(
            Call::SearchRecordsHandler(
                storage_handle,
                _convert_c_string(type_),
                _convert_c_string(query_json),
                _convert_c_string(options_json),
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_search_all_records_handler(storage_handle: i32,
                                                   search_handle_p: *mut i32) -> ErrorCode {
        unsafe { *search_handle_p = RETURN_SEARCH_HANDLE; }

        DEBUG_VEC.write().unwrap().push(
            Call::SearchAllRecordsHandler(
                storage_handle,
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_get_search_total_count_handler(storage_handle: i32,
                                                       search_handle: i32,
                                                       total_count_p: *mut usize) -> ErrorCode {
        unsafe { *total_count_p = RETURN_SEARCH_TOTAL_COUNT; }

        DEBUG_VEC.write().unwrap().push(
            Call::GetSearchTotalCountHandler(
                storage_handle,
                search_handle,
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_fetch_search_next_record_handler(storage_handle: i32,
                                                         search_handle: i32,
                                                         record_handle_p: *mut i32) -> ErrorCode {
        unsafe { *record_handle_p = RETURN_RECORD_HANDLE; }

        DEBUG_VEC.write().unwrap().push(
            Call::FetchSearchNextRecordHandler(
                storage_handle,
                search_handle,
            )
        );

        ErrorCode::Success
    }

    extern "C" fn _mock_free_search_handler(storage_handle: i32,
                                            search_handle: i32) -> ErrorCode {
        DEBUG_VEC.write().unwrap().push(
            Call::FreeSearchHandler(
                storage_handle,
                search_handle,
            )
        );

        ErrorCode::Success
    }

    fn _create_storage_type() -> PluggedStorageType {
        PluggedStorageType::new(
            _mock_create_handler,
            _mock_open_handler,
            _mock_close_handler,
            _mock_delete_handler,
            _mock_add_record_handler,
            _mock_update_record_value_handler,
            _mock_update_record_tags_handler,
            _mock_add_record_tags_handler,
            _mock_delete_record_tags_handler,
            _mock_delete_record_handler,
            _mock_get_record_handler,
            _mock_get_record_id_handler,
            _mock_get_record_type_handler,
            _mock_get_record_value_handler,
            _mock_get_record_tags_handler,
            _mock_free_record_handler,
            _mock_get_storage_metadata_handler,
            _mock_set_storage_metadata_handler,
            _mock_free_storage_metadata_handler,
            _mock_search_records_handler,
            _mock_search_all_records_handler,
            _mock_get_search_total_count_handler,
            _mock_fetch_search_next_record_handler,
            _mock_free_search_handler,
        )
    }

    #[test]
    fn plugged_storage_type_new_works() {
        let _storage_type = _create_storage_type();
    }


    #[test]
    fn plugged_storage_type_create_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage_type = _create_storage_type();
        let storage_name = "wallet1";
        let credentials = "credentials";
        let metadata = vec![1, 2, 3];

        storage_type.create_storage(storage_name, None, Some(credentials), &metadata).unwrap();

        let expected_call = Call::CreateHandler(
            Some(storage_name.to_owned()),
            None,
            Some(credentials.to_owned()),
            Some(base64::encode(&metadata)),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_type_open_close_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage_type = _create_storage_type();
        let storage_name = "wallet1";
        let credentials = "credentials";

        let mut storage = storage_type.open_storage(storage_name, None, Some(credentials)).unwrap();
        storage.close().unwrap();

        let expected_open_call = Call::OpenHandler(
            Some(storage_name.to_owned()),
            None,
            Some(credentials.to_owned()),
        );

        let expected_close_call = Call::CloseHandler(
            RETURN_STORAGE_HANDLE
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 2);
        assert_eq!(&expected_open_call, debug.get(0).unwrap());
        assert_eq!(&expected_close_call, debug.get(1).unwrap());
    }

    #[test]
    fn plugged_storage_type_delete_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage_type = _create_storage_type();
        let storage_name = "wallet1";
        let credentials = "credentials";

        storage_type.delete_storage(storage_name, None, Some(credentials)).unwrap();

        let expected_call = Call::DeleteHandler(
            Some(storage_name.to_owned()),
            None,
            Some(credentials.to_owned()),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    fn _open_storage() -> Box<dyn WalletStorage> {
        // save the current index inside of DEBUG_VEC.
        let open_index = DEBUG_VEC.read().unwrap().len();

        let storage_type = _create_storage_type();
        let storage_name = "wallet1";
        let credentials = "credentials";

        let storage = storage_type.open_storage(storage_name, None, Some(credentials)).unwrap();

        let expected_call = Call::OpenHandler(
            Some(storage_name.to_owned()),
            None,
            Some(credentials.to_owned()),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), open_index + 1);
        assert_eq!(&expected_call, debug.get(open_index).unwrap());

        storage
    }

    fn _fetch_options(retrieve_value: bool, retrieve_tags: bool, retrieve_type: bool) -> String {
        let mut map = HashMap::with_capacity(3);

        map.insert("retrieveValue", retrieve_value);
        map.insert("retrieveTags", retrieve_tags);
        map.insert("retrieveType", retrieve_type);

        serde_json::to_string(&map).unwrap()
    }

    fn _search_options(retrieve_records: bool, retrieve_total_count: bool, retrieve_value: bool, retrieve_tags: bool, retrieve_type: bool) -> String {
        let mut map = HashMap::with_capacity(5);

        map.insert("retrieveRecords", retrieve_records);
        map.insert("retrieveTotalCount", retrieve_total_count);
        map.insert("retrieveValue", retrieve_value);
        map.insert("retrieveTags", retrieve_tags);
        map.insert("retrieveType", retrieve_type);

        serde_json::to_string(&map).unwrap()
    }

    #[test]
    fn plugged_storage_add_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let value = EncryptedValue { data: _random_vector(256), key: _random_vector(60) };
        let mut tags = Vec::new();
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));
        tags.push(Tag::PlainText(_random_vector(32), _random_string(64)));
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));

        storage.add(&type_, &id, &value, &tags).unwrap();

        let expected_call = Call::AddRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            value.to_bytes(),
            serde_json::from_str(&_tags_to_json(&tags).unwrap()).unwrap(),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_update_record_value_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let value = EncryptedValue { data: _random_vector(256), key: _random_vector(44) };

        storage.update(&type_, &id, &value).unwrap();

        let expected_call = Call::UpdateRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            value.to_bytes(),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_update_record_tags_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let mut tags = Vec::new();
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));
        tags.push(Tag::PlainText(_random_vector(32), _random_string(64)));
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));


        storage.update_tags(&type_, &id, &tags).unwrap();

        let expected_call = Call::UpdateRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            serde_json::from_str(&_tags_to_json(&tags).unwrap()).unwrap(),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_add_record_tags_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let mut tags = Vec::new();
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));
        tags.push(Tag::PlainText(_random_vector(32), _random_string(64)));
        tags.push(Tag::Encrypted(_random_vector(32), _random_vector(64)));


        storage.add_tags(&type_, &id, &tags).unwrap();

        let expected_call = Call::AddRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            serde_json::from_str(&_tags_to_json(&tags).unwrap()).unwrap(),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_get_record_type_value_tags_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let options = _fetch_options(true, true, true);

        let storage_entity = storage.get(&type_, &id, &options).unwrap();

        let expected_storage_entity = StorageRecord {
            type_: Some(type_.clone()),
            id: id.clone(),
            value: Some(RETURN_VALUE.read().unwrap().1.clone()),
            tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
        };

        assert_eq!(expected_storage_entity, storage_entity);

        let expected_get_record_call = Call::GetRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            Some(options.to_owned()),
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        // NOTE: it would not call get_record_type, because it can copy provided type.
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 4);
        assert_eq!(&expected_get_record_call, debug.get(0).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(1).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(2).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(3).unwrap());
    }

    #[test]
    fn plugged_storage_get_record_value_tags_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let options = _fetch_options(true, true, false);

        let storage_entity = storage.get(&type_, &id, &options).unwrap();

        let expected_storage_entity = StorageRecord {
            type_: None,
            id: id.clone(),
            value: Some(RETURN_VALUE.read().unwrap().1.clone()),
            tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
        };

        assert_eq!(expected_storage_entity, storage_entity);

        let expected_get_record_call = Call::GetRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            Some(options.to_owned()),
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 4);
        assert_eq!(&expected_get_record_call, debug.get(0).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(1).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(2).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(3).unwrap());
    }

    #[test]
    fn plugged_storage_get_record_value_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let options = _fetch_options(true, false, false);

        let storage_entity = storage.get(&type_, &id, &options).unwrap();

        let expected_storage_entity = StorageRecord {
            type_: None,
            id: id.clone(),
            value: Some(RETURN_VALUE.read().unwrap().1.clone()),
            tags: None,
        };

        assert_eq!(expected_storage_entity, storage_entity);

        let expected_get_record_call = Call::GetRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            Some(options.to_owned()),
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 3);
        assert_eq!(&expected_get_record_call, debug.get(0).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(1).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(2).unwrap());
    }

    #[test]
    fn plugged_storage_get_record_tags_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let options = _fetch_options(false, true, false);

        let storage_entity = storage.get(&type_, &id, &options).unwrap();

        let expected_storage_entity = StorageRecord {
            type_: None,
            id: id.clone(),
            value: None,
            tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
        };

        assert_eq!(expected_storage_entity, storage_entity);

        let expected_get_record_call = Call::GetRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            Some(options.to_owned()),
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        // NOTE: it would not call get_record_type, because it already have it.
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 3);
        assert_eq!(&expected_get_record_call, debug.get(0).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(1).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(2).unwrap());
    }

    #[test]
    fn plugged_storage_get_record_none_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);
        let id = _random_vector(32);
        let options = _fetch_options(false, false, false);

        let storage_entity = storage.get(&type_, &id, &options).unwrap();

        let expected_storage_entity = StorageRecord {
            type_: None,
            id: id.clone(),
            value: None,
            tags: None,
        };

        assert_eq!(expected_storage_entity, storage_entity);

        let expected_get_record_call = Call::GetRecordHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(base64::encode(&id)),
            Some(options.to_owned()),
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 2);
        assert_eq!(&expected_get_record_call, debug.get(0).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(1).unwrap());
    }

    #[test]
    fn plugged_storage_get_storage_metadata_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let metadata = storage.get_storage_metadata().unwrap();

        assert_eq!(RETURN_METADATA.read().unwrap().1, metadata);

        let expected_get_call = Call::GetStorageMetadataHandler(
            RETURN_STORAGE_HANDLE,
        );

        let expected_free_call = Call::FreeStorageMetadataHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_METADATA_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 2);
        assert_eq!(&expected_get_call, debug.get(0).unwrap());
        assert_eq!(&expected_free_call, debug.get(1).unwrap());
    }

    #[test]
    fn plugged_storage_set_storage_metadata_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let metadata = _random_vector(512);

        storage.set_storage_metadata(&metadata).unwrap();

        let expected_call = Call::SetStorageMetadataHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&metadata)),
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 1);
        assert_eq!(&expected_call, debug.get(0).unwrap());
    }

    #[test]
    fn plugged_storage_search_with_total_count_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);

        let tag_name = _random_vector(32);
        let tag_value = _random_vector(32);

        let query = language::Operator::Eq(
            language::TagName::EncryptedTagName(tag_name.clone()),
            language::TargetValue::Encrypted(tag_value.clone()),
        );
        let options = _search_options(true, true, true, true, false);

        {
            let mut storage_iterator = storage.search(&type_, &query, Some(&options)).unwrap();

            // TODO: solve how to get &PluggedStorage from Box<dyn WalletStorage>

            //        let expected_storage_iterator = PluggedStorageIterator{
            //            storage: &storage.downcast::<PluggedStorage>().unwrap(),
            //            search_handle: RETURN_SEARCH_HANDLE,
            //            options: FetchOptions{
            //                retrieveType: false,
            //                retrieveValue: true,
            //                retrieveTags: true,
            //            }
            //        };

            //        assert_eq!(*storage_iterator as PluggedStorageIterator, expected_storage_iterator);

            let total_count = storage_iterator.get_total_count().unwrap();

            assert_eq!(total_count, Some(RETURN_SEARCH_TOTAL_COUNT));

            let storage_entity = storage_iterator.next().unwrap();

            let expected_storage_entity = StorageRecord {
                type_: None,
                id: RETURN_ID.read().unwrap().1.clone(),
                value: Some(RETURN_VALUE.read().unwrap().1.clone()),
                tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
            };

            assert_eq!(expected_storage_entity, storage_entity.unwrap());
        }

        let expected_search_call = Call::SearchRecordsHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(query.to_string()),
            Some(options.to_string()),
        );
        let expected_total_count_call = Call::GetSearchTotalCountHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );
        let expected_fetch_next_call = Call::FetchSearchNextRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );
        let expected_get_id_call = Call::GetRecordIdHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_search_call = Call::FreeSearchHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 8);
        assert_eq!(&expected_search_call, debug.get(0).unwrap());
        assert_eq!(&expected_total_count_call, debug.get(1).unwrap());
        assert_eq!(&expected_fetch_next_call, debug.get(2).unwrap());
        assert_eq!(&expected_get_id_call, debug.get(3).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(4).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(5).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(6).unwrap());
        assert_eq!(&expected_free_search_call, debug.get(7).unwrap());
    }

    #[test]
    fn plugged_storage_search_without_total_count_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        let type_ = _random_vector(32);

        let tag_name = _random_vector(32);
        let tag_value = _random_vector(32);

        let query = language::Operator::Eq(
            language::TagName::EncryptedTagName(tag_name.clone()),
            language::TargetValue::Encrypted(tag_value.clone()),
        );
        let options = _search_options(true, false, true, true, false);

        {
            let mut storage_iterator = storage.search(&type_, &query, Some(&options)).unwrap();

            // TODO: solve how to get &PluggedStorage from Box<dyn WalletStorage>

            //        let expected_storage_iterator = PluggedStorageIterator{
            //            storage: &storage.downcast::<PluggedStorage>().unwrap(),
            //            search_handle: RETURN_SEARCH_HANDLE,
            //            options: FetchOptions{
            //                retrieveType: false,
            //                retrieveValue: true,
            //                retrieveTags: true,
            //            }
            //        };

            //        assert_eq!(*storage_iterator as PluggedStorageIterator, expected_storage_iterator);

            let total_count = storage_iterator.get_total_count().unwrap();

            assert_eq!(total_count, None);

            let storage_entity = storage_iterator.next().unwrap();

            let expected_storage_entity = StorageRecord {
                type_: None,
                id: RETURN_ID.read().unwrap().1.clone(),
                value: Some(RETURN_VALUE.read().unwrap().1.clone()),
                tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
            };

            assert_eq!(expected_storage_entity, storage_entity.unwrap());
        }

        let expected_search_call = Call::SearchRecordsHandler(
            RETURN_STORAGE_HANDLE,
            Some(base64::encode(&type_)),
            Some(query.to_string()),
            Some(options.to_string()),
        );
        // No total count call.
        let expected_fetch_next_call = Call::FetchSearchNextRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );
        let expected_get_id_call = Call::GetRecordIdHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_search_call = Call::FreeSearchHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 7);
        assert_eq!(&expected_search_call, debug.get(0).unwrap());
        assert_eq!(&expected_fetch_next_call, debug.get(1).unwrap());
        assert_eq!(&expected_get_id_call, debug.get(2).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(3).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(4).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(5).unwrap());
        assert_eq!(&expected_free_search_call, debug.get(6).unwrap());
    }

    #[test]
    fn plugged_storage_get_all_works() {
        DEBUG_VEC.write().unwrap().clear();

        let storage = _open_storage();

        DEBUG_VEC.write().unwrap().clear();

        {
            let mut storage_iterator = storage.get_all().unwrap();

            // TODO: solve how to get &PluggedStorage from Box<dyn WalletStorage>

            //        let expected_storage_iterator = PluggedStorageIterator{
            //            storage: &storage.downcast::<PluggedStorage>().unwrap(),
            //            search_handle: RETURN_SEARCH_HANDLE,
            //            options: FetchOptions{
            //                retrieveType: false,
            //                retrieveValue: true,
            //                retrieveTags: true,
            //            }
            //        };

            //        assert_eq!(*storage_iterator as PluggedStorageIterator, expected_storage_iterator);

            let storage_entity = storage_iterator.next().unwrap();

            let expected_storage_entity = StorageRecord {
                type_: Some(RETURN_TYPE.read().unwrap().1.clone()),
                id: RETURN_ID.read().unwrap().1.clone(),
                value: Some(RETURN_VALUE.read().unwrap().1.clone()),
                tags: Some(RETURN_TAGS.read().unwrap().1.clone()),
            };

            assert_eq!(expected_storage_entity, storage_entity.unwrap());

            let total_count = storage_iterator.get_total_count().unwrap();

            assert_eq!(None, total_count)
        }

        let expected_search_call = Call::SearchAllRecordsHandler(
            RETURN_STORAGE_HANDLE,
        );
        let expected_fetch_next_call = Call::FetchSearchNextRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );
        let expected_get_type_call = Call::GetRecordTypeHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_id_call = Call::GetRecordIdHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_value_call = Call::GetRecordValueHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_get_tags_call = Call::GetRecordTagsHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_record_call = Call::FreeRecordHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_RECORD_HANDLE,
        );
        let expected_free_search_call = Call::FreeSearchHandler(
            RETURN_STORAGE_HANDLE,
            RETURN_SEARCH_HANDLE,
        );

        let debug = DEBUG_VEC.read().unwrap();

        assert_eq!(debug.len(), 8);
        assert_eq!(&expected_search_call, debug.get(0).unwrap());
        assert_eq!(&expected_fetch_next_call, debug.get(1).unwrap());
        assert_eq!(&expected_get_type_call, debug.get(2).unwrap());
        assert_eq!(&expected_get_id_call, debug.get(3).unwrap());
        assert_eq!(&expected_get_value_call, debug.get(4).unwrap());
        assert_eq!(&expected_get_tags_call, debug.get(5).unwrap());
        assert_eq!(&expected_free_record_call, debug.get(6).unwrap());
        assert_eq!(&expected_free_search_call, debug.get(7).unwrap());
    }
}
