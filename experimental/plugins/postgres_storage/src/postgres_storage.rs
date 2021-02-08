extern crate owning_ref;
extern crate sodiumoxide;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate openssl;
extern crate postgres_openssl;

use ::std::sync::RwLock;
use postgres;
use self::r2d2_postgres::{TlsMode, PostgresConnectionManager};
use serde_json;
use std::path::Path;

use self::postgres::tls::openssl::OpenSsl;
use postgres::tls::openssl::openssl::ssl::{SslMethod, SslConnectorBuilder};
use postgres::tls::openssl::openssl::x509::X509_FILETYPE_PEM;
use self::owning_ref::OwningHandle;
use std::rc::Rc;
use std::time::Duration;

use errors::wallet::WalletStorageError;
use errors::common::CommonError;
use wql::language;
use wql::query;
use wql::transaction;

use wql::storage::{StorageIterator, WalletStorage, StorageRecord, EncryptedValue, Tag, TagName};
use self::r2d2_postgres::r2d2::Pool;
use errors::wallet::WalletStorageError::{ConfigError};

fn default_true() -> bool { true }

fn default_false() -> bool { false }

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordOptions {
    #[serde(default = "default_false")]
    retrieve_type: bool,
    #[serde(default = "default_true")]
    retrieve_value: bool,
    #[serde(default = "default_false")]
    retrieve_tags: bool,
}

impl RecordOptions {
    pub fn id() -> String {
        let options = RecordOptions {
            retrieve_type: false,
            retrieve_value: false,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }

    pub fn id_value() -> String {
        let options = RecordOptions {
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }
}

impl Default for RecordOptions {
    fn default() -> RecordOptions {
        RecordOptions {
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    #[serde(default = "default_true")]
    retrieve_records: bool,
    #[serde(default = "default_false")]
    retrieve_total_count: bool,
    #[serde(default = "default_false")]
    retrieve_type: bool,
    #[serde(default = "default_true")]
    retrieve_value: bool,
    #[serde(default = "default_false")]
    retrieve_tags: bool,
}

impl SearchOptions {
    pub fn id_value() -> String {
        let options = SearchOptions {
            retrieve_records: true,
            retrieve_total_count: true,
            retrieve_type: true,
            retrieve_value: true,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }
}

impl Default for SearchOptions {
    fn default() -> SearchOptions {
        SearchOptions {
            retrieve_records: true,
            retrieve_total_count: false,
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        }
    }
}


const _POSTGRES_DB: &str = "postgres";
const _WALLETS_DB: &str = "wallets";
const _PLAIN_TAGS_QUERY: &str = "SELECT name, value from tags_plaintext where item_id = $1";
const _ENCRYPTED_TAGS_QUERY: &str = "SELECT name, value from tags_encrypted where item_id = $1";
const _PLAIN_TAGS_QUERY_MULTI: &str = "SELECT name, value from tags_plaintext where item_id = $1 and wallet_id = $2";
const _ENCRYPTED_TAGS_QUERY_MULTI: &str = "SELECT name, value from tags_encrypted where item_id = $1 and wallet_id = $2";
const _CREATE_WALLET_DATABASE: &str = "CREATE DATABASE \"$1\"";
const _CREATE_WALLETS_DATABASE: &str = "CREATE DATABASE wallets";
// Note: wallet id length was constrained before by postgres database name length to 64 characters, keeping the same restrictions
const _CREATE_SCHEMA: [&str; 12] = [
    "CREATE TABLE IF NOT EXISTS metadata (
        id BIGSERIAL PRIMARY KEY,
        value BYTEA NOT NULL
    )",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_metadata_values ON metadata(value)",
    "CREATE TABLE IF NOT EXISTS items(
        id BIGSERIAL PRIMARY KEY,
        type BYTEA NOT NULL,
        name BYTEA NOT NULL,
        value BYTEA NOT NULL,
        key BYTEA NOT NULL
    )",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_items_type_name ON items(type, name)",
    "CREATE TABLE IF NOT EXISTS tags_encrypted(
        name BYTEA NOT NULL,
        value BYTEA NOT NULL,
        item_id BIGINT NOT NULL,
        PRIMARY KEY(name, item_id),
        FOREIGN KEY(item_id)
            REFERENCES items(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_name ON tags_encrypted(name)",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_value ON tags_encrypted(md5(value))",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_item_id ON tags_encrypted(item_id)",
    "CREATE TABLE IF NOT EXISTS tags_plaintext(
        name BYTEA NOT NULL,
        value TEXT NOT NULL,
        item_id BIGINT NOT NULL,
        PRIMARY KEY(name, item_id),
        FOREIGN KEY(item_id)
            REFERENCES items(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_name ON tags_plaintext(name)",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_value ON tags_plaintext(value)",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_item_id ON tags_plaintext(item_id)"
];
const _CREATE_SCHEMA_MULTI: [&str; 14] = [
    "CREATE TABLE IF NOT EXISTS metadata (
        wallet_id VARCHAR(64) NOT NULL,
        value BYTEA NOT NULL,
        PRIMARY KEY(wallet_id)
    )",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_metadata_wallet_id_id ON metadata(wallet_id)",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_metadata_values ON metadata(wallet_id, value)",
    "CREATE TABLE IF NOT EXISTS items(
        wallet_id VARCHAR(64) NOT NULL,
        id BIGSERIAL NOT NULL,
        type BYTEA NOT NULL,
        name BYTEA NOT NULL,
        value BYTEA NOT NULL,
        key BYTEA NOT NULL,
        PRIMARY KEY(wallet_id, id)
    )",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_items_wallet_id_id ON items(wallet_id, id)",
    "CREATE UNIQUE INDEX IF NOT EXISTS ux_items_type_name ON items(wallet_id, type, name)",
    "CREATE TABLE IF NOT EXISTS tags_encrypted(
        wallet_id VARCHAR(64) NOT NULL,
        name BYTEA NOT NULL,
        value BYTEA NOT NULL,
        item_id BIGINT NOT NULL,
        PRIMARY KEY(wallet_id, name, item_id),
        FOREIGN KEY(wallet_id, item_id)
            REFERENCES items(wallet_id, id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_name ON tags_encrypted(wallet_id, name)",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_value ON tags_encrypted(wallet_id, value)",
    "CREATE INDEX IF NOT EXISTS ix_tags_encrypted_wallet_id_item_id ON tags_encrypted(wallet_id, item_id)",
    "CREATE TABLE IF NOT EXISTS tags_plaintext(
        wallet_id VARCHAR(64) NOT NULL,
        name BYTEA NOT NULL,
        value TEXT NOT NULL,
        item_id BIGINT NOT NULL,
        PRIMARY KEY(wallet_id, name, item_id),
        FOREIGN KEY(wallet_id, item_id)
            REFERENCES items(wallet_id, id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_name ON tags_plaintext(wallet_id, name)",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_value ON tags_plaintext(wallet_id, value)",
    "CREATE INDEX IF NOT EXISTS ix_tags_plaintext_wallet_id_item_id ON tags_plaintext(wallet_id, item_id)"
];
const _DROP_WALLET_DATABASE: &str = "DROP DATABASE \"$1\"";
const _DROP_SCHEMA: [&str; 4] = [
    "DROP TABLE tags_plaintext",
    "DROP TABLE tags_encrypted",
    "DROP TABLE items",
    "DROP TABLE metadata"
];
const _DELETE_WALLET_MULTI: [&str; 4] = [
    "DELETE FROM tags_plaintext WHERE wallet_id = $1",
    "DELETE FROM tags_encrypted WHERE wallet_id = $1",
    "DELETE FROM items WHERE wallet_id = $1",
    "DELETE FROM metadata WHERE wallet_id = $1"
];


#[derive(Debug)]
struct TagRetriever<'a> {
    plain_tags_stmt: postgres::stmt::Statement<'a>,
    encrypted_tags_stmt: postgres::stmt::Statement<'a>,
    wallet_id: Option<String>,
}

type TagRetrieverOwned = OwningHandle<Rc<r2d2::PooledConnection<PostgresConnectionManager>>, Box<TagRetriever<'static>>>;

impl<'a> TagRetriever<'a> {
    fn new_owned(conn: Rc<r2d2::PooledConnection<PostgresConnectionManager>>, wallet_id: Option<String>) -> Result<TagRetrieverOwned, WalletStorageError> {
        OwningHandle::try_new(conn.clone(), |conn| -> Result<_, postgres::Error> {
            let (plain_tags_stmt, encrypted_tags_stmt) = unsafe {
                match wallet_id {
                    Some(_) => ((*conn).prepare(_PLAIN_TAGS_QUERY_MULTI)?,
                                (*conn).prepare(_ENCRYPTED_TAGS_QUERY_MULTI)?),
                    None => ((*conn).prepare(_PLAIN_TAGS_QUERY)?,
                             (*conn).prepare(_ENCRYPTED_TAGS_QUERY)?)
                }
            };
            let tr = TagRetriever {
                plain_tags_stmt,
                encrypted_tags_stmt,
                wallet_id,
            };
            Ok(Box::new(tr))
        }).map_err(WalletStorageError::from)
    }

    fn retrieve(&mut self, id: i64) -> Result<Vec<Tag>, WalletStorageError> {
        let mut tags = Vec::new();

        let plain_results = match self.wallet_id {
            Some(ref w_id) => self.plain_tags_stmt.query(&[&id, &w_id])?,
            None => self.plain_tags_stmt.query(&[&id])?
        };
        let mut iter_plain = plain_results.iter();
        while let Some(res) = iter_plain.next() {
            let row = res;
            tags.push(Tag::PlainText(row.get(0), row.get(1)));
        }

        let encrypted_results = match self.wallet_id {
            Some(ref w_id) => self.encrypted_tags_stmt.query(&[&id, &w_id])?,
            None => self.encrypted_tags_stmt.query(&[&id])?
        };
        let mut iter_encrypted = encrypted_results.iter();
        while let Some(res) = iter_encrypted.next() {
            let row = res;
            tags.push(Tag::Encrypted(row.get(0), row.get(1)));
        }

        Ok(tags)
    }
}

struct PostgresStorageIterator {
    rows: Option<
        OwningHandle<
            OwningHandle<
                Rc<r2d2::PooledConnection<PostgresConnectionManager>>,
                Box<postgres::stmt::Statement<'static>>>,
            Box<postgres::rows::Rows<>>>>,
    tag_retriever: Option<TagRetrieverOwned>,
    options: RecordOptions,
    total_count: Option<usize>,
    iter_count: usize,
}

impl PostgresStorageIterator {
    fn new(stmt: Option<OwningHandle<Rc<r2d2::PooledConnection<PostgresConnectionManager>>, Box<postgres::stmt::Statement<'static>>>>,
           args: &[&dyn postgres::types::ToSql],
           options: RecordOptions,
           tag_retriever: Option<TagRetrieverOwned>,
           total_count: Option<usize>) -> Result<PostgresStorageIterator, WalletStorageError> {
        let mut iter = PostgresStorageIterator {
            rows: None,
            tag_retriever,
            options,
            total_count,
            iter_count: 0,
        };

        if let Some(stmt) = stmt {
            iter.rows = Some(OwningHandle::try_new(
                stmt, |stmt|
                    unsafe {
                        (*(stmt as *mut postgres::stmt::Statement)).query(args).map(Box::new)
                    },
            )?);
        }

        Ok(iter)
    }
}

impl StorageIterator for PostgresStorageIterator {
    fn next(&mut self) -> Result<Option<StorageRecord>, WalletStorageError> {
        // if records are not requested.
        if self.rows.is_none() {
            return Ok(None);
        }

        // TODO not sure if iter().nth() is the most efficient way to iterate through the result set
        // TODO investigate if the Iter object can be cached between calls to next()
        match self.rows.as_mut().unwrap().iter().nth(self.iter_count) {
            Some(row) => {
                self.iter_count = self.iter_count + 1;
                let name = row.get(1);
                let value = if self.options.retrieve_value {
                    Some(EncryptedValue::new(row.get(2), row.get(3)))
                } else {
                    None
                };
                let tags = if self.options.retrieve_tags {
                    match self.tag_retriever {
                        Some(ref mut tag_retriever) => Some(tag_retriever.retrieve(row.get(0))?),
                        None => return Err(WalletStorageError::CommonError(
                            CommonError::InvalidState("Fetch tags option set and tag retriever is None".to_string())
                        ))
                    }
                } else {
                    None
                };
                let type_ = if self.options.retrieve_type {
                    Some(row.get(4))
                } else {
                    None
                };
                Ok(Some(StorageRecord::new(name, value, type_, tags)))
            }
            //Some(Err(err)) => Err(WalletStorageError::from(err)),
            None => Ok(None)
        }
    }

    fn get_total_count(&self) -> Result<Option<usize>, WalletStorageError> {
        Ok(self.total_count)
    }
}

#[derive(Deserialize, Debug)]
pub struct PostgresConfig {
    url: String,
    tls: Option<String>,
    // default off
    max_connections: Option<u32>,
    // default 5
    min_idle_time: Option<u32>,
    // default 0, deprecated
    min_idle_count: Option<u32>,
    // default 0
    connection_timeout: Option<u64>,
    // default 5
    wallet_scheme: Option<WalletScheme>,   // default DatabasePerWallet
    database_name: Option<String>,   // default _WALLET_DB
    ca_file: Option<String>,
    certificate_chain_file: Option<String>,
    private_key_file: Option<String>
}

fn ssl_config(config: &PostgresConfig) -> OpenSsl {
    let mut builder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
    // TODO: Do some proper error handling
    if let Some(ca_file) = &config.ca_file {
        builder.set_ca_file(Path::new(&ca_file)).unwrap();
    }
    if let Some(certificate_chain_file) = &config.certificate_chain_file {
        builder.set_certificate_chain_file(Path::new(&certificate_chain_file)).unwrap();
    }
    if let Some(private_key_file) = &config.private_key_file {
        builder.set_private_key_file(Path::new(&private_key_file), X509_FILETYPE_PEM).unwrap();
    }

    let mut ssl = OpenSsl::new().unwrap();
    *ssl.connector_mut() = builder.build();
    ssl
}

impl PostgresConfig {
    fn tls<'a>(&'a self, ssl_conf: &'a Option<OpenSsl>) -> postgres::TlsMode {
        match &self.tls {
            Some(tls) => match tls.as_ref() {
                "None" => postgres::TlsMode::None,
                // TODO add tls support for connecting to postgres db
                "Prefer" => match ssl_conf {
                    Some(conf) => postgres::TlsMode::Prefer(conf),
                    None => postgres::TlsMode::None
                },
                "Require" => match ssl_conf {
                    Some(conf) => postgres::TlsMode::Require(conf),
                    None => postgres::TlsMode::None
                },
                _ => postgres::TlsMode::None
            },
            None => postgres::TlsMode::None
        }
    }

    fn r2d2_tls(&self) -> TlsMode {
        let ssl_conf = Some(ssl_config(&self));
        match &self.tls {
            Some(tls) => match tls.as_ref() {
                "None" => TlsMode::None,
                // TODO add tls support for connecting to postgres db
                "Prefer" => match ssl_conf {
                    Some(conf) => TlsMode::Prefer(Box::new(conf)),
                    None => TlsMode::None
                },
                "Require" => match ssl_conf {
                    Some(conf) => TlsMode::Require(Box::new(conf)),
                    None => TlsMode::None
                },
                _ => TlsMode::None
            },
            None => TlsMode::None
        }
    }

    /// Sets the maximum number of connections managed by the pool.
    fn max_connections(&self) -> u32 {
        match &self.max_connections {
            Some(conn) => *conn,
            None => 5
        }
    }
    /// Sets the minimum idle connection count maintained by the pool.
    fn min_idle_count(&self) -> u32 {
        match self.min_idle_count {
            Some(idle_count) => idle_count,
            None => match self.min_idle_time {
                Some(idle_count_deprecated) => {
                    warn!("Configuration option min_idle_time is deprecated. Use min_idle_count instead.");
                    idle_count_deprecated
                }
                None => 0
            }
        }
    }
    /// Sets the idle timeout used by the pool.
    fn connection_timeout(&self) -> u64 {
        match &self.connection_timeout {
            Some(timeout) => *timeout,
            None => 5
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PostgresCredentials {
    account: String,
    password: String,
    admin_account: Option<String>,
    admin_password: Option<String>,
}

#[derive(Debug)]
pub struct PostgresStorage {
    pool: r2d2::Pool<PostgresConnectionManager>,
    wallet_id: String,
}

pub trait WalletStorageType {
    fn init_storage(&self, config: Option<&str>, credentials: Option<&str>) -> Result<(), WalletStorageError>;
    fn create_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>, metadata: &[u8]) -> Result<(), WalletStorageError>;
    fn open_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> Result<Box<PostgresStorage>, WalletStorageError>;
    fn delete_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> Result<(), WalletStorageError>;
}

#[derive(Deserialize, Debug)]
#[derive(Copy, Clone)]
enum WalletScheme {
    DatabasePerWallet,
    MultiWalletSingleTable,
    MultiWalletSingleTableSharedPool,
    MultiWalletMultiTable,
}

trait WalletStrategy {
    // initialize storage based on wallet storage strategy
    fn init_storage(&self, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError>;
    // initialize a single wallet based on wallet storage strategy
    fn create_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials, metadata: &[u8]) -> Result<(), WalletStorageError>;
    // open a wallet based on wallet storage strategy
    fn open_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<Box<PostgresStorage>, WalletStorageError>;
    // delete a single wallet based on wallet storage strategy
    fn delete_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError>;
    // determine physical table name based on wallet strategy
    fn table_name(&self, id: &str, base_name: &str) -> String;
    // determine additional query parameters based on wallet strategy
    fn query_qualifier(&self) -> Option<String>;
}


pub struct PostgresStorageType {}

struct DatabasePerWalletStrategy {
    ssl_config: Option<OpenSsl>
}

struct MultiWalletSingleTableStrategy {
    ssl_config: Option<OpenSsl>
}

struct MultiWalletMultiTableStrategy {}

struct MultiWalletSingleTableStrategySharedPool {
    pool: r2d2::Pool<PostgresConnectionManager>,
    ssl_config: Option<OpenSsl>
}


impl WalletStrategy for MultiWalletSingleTableStrategySharedPool {

    fn init_storage(&self, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // create database and tables for storage
        // if admin user and password aren't provided then bail
        if credentials.admin_account == None || credentials.admin_password == None {
            return Ok(());
        }

        let url_base = PostgresStorageType::_admin_postgres_url(&config, &credentials);
        let url = PostgresStorageType::_postgres_url(_WALLETS_DB, &config, &credentials);

        let conn = postgres::Connection::connect(&url_base[..], config.tls(&self.ssl_config))?;

        if let Err(error) = conn.execute(&_CREATE_WALLETS_DATABASE, &[]) {
            if error.code() != Some(&postgres::error::DUPLICATE_DATABASE) {
                conn.finish()?;
                return Err(WalletStorageError::IOError(format!("Error occurred while creating the database: {}", error)));
            } else {
                // if database already exists, assume tables are created already and return
                conn.finish()?;
                return Ok(());
            }
        }
        conn.finish()?;

        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        for sql in &_CREATE_SCHEMA_MULTI {
            if let Err(error) = conn.execute(sql, &[]) {
                conn.finish()?;
                return Err(WalletStorageError::IOError(format!("Error occurred while creating wallet schema: {}", error)));
            }
        }
        conn.finish()?;
        Ok(())
    }

    fn create_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials, metadata: &[u8]) -> Result<(), WalletStorageError> {
        // insert metadata
        let url = PostgresStorageType::_postgres_url(_WALLETS_DB, &config, &credentials);

        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        // We allow error on conflict since this indicates AlreadyExists error
        let ret = match conn.execute("INSERT INTO metadata(wallet_id, value) VALUES($1, $2)", &[&id, &metadata]) {
            Ok(_) => Ok(()),
            Err(error) => {
                if error.code() == Some(&postgres::error::UNIQUE_VIOLATION) {
                    Err(WalletStorageError::AlreadyExists)
                } else {
                    Err(WalletStorageError::IOError(format!("Error occurred while inserting into metadata: {}", error)))
                }
            }
        };
        conn.finish()?;
        ret
    }

    fn open_wallet(&self, id: &str, _config: &PostgresConfig, _credentials: &PostgresCredentials) -> Result<Box<PostgresStorage>, WalletStorageError> {
        let connection = match self.pool.get() {
            Ok(connection) => connection,
            Err(error) => {
                error!("Error retrieving connection from connection pool. {:?}", error);
                return Err(WalletStorageError::NotFound)
            }
        };
        // select metadata for this wallet to ensure it exists
        let res: Result<Vec<u8>, WalletStorageError> = {
            let mut rows = connection.query(
                "SELECT value FROM metadata WHERE wallet_id = $1",
                &[&id]);
            match rows.as_mut().unwrap().iter().next() {
                Some(row) => Ok(row.get(0)),
                None => {
                    error!("No metadata was found for wallet id '{}' which indicates this wallet does not exist.", id);
                    Err(WalletStorageError::ItemNotFound)
                }
            }
        };
        match res {
            Ok(_entity) => (),
            Err(error) => {
                error!("Error retrieving metadata for wallet id '{}' which indicates this wallet does not exist. Error details: {:?}", id, error);
                return Err(WalletStorageError::NotFound)
            }
        };

        Ok(Box::new(PostgresStorage {
            pool: self.pool.clone(),
            wallet_id: id.to_string(),
        }))
    }

    fn delete_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        let url = PostgresStorageType::_postgres_url(&_WALLETS_DB, &config, &credentials);

        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        let mut ret = Ok(());
        for sql in &_DELETE_WALLET_MULTI {
            ret = match conn.execute(sql, &[&id]) {
                Ok(row_count) => {
                    if row_count == 0 {
                        Err(WalletStorageError::NotFound)
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    Err(WalletStorageError::IOError(format!("Error occurred while deleting wallet: {}", error)))
                }
            }
        };
        conn.finish()?;
        return ret;
    }

    fn table_name(&self, _id: &str, base_name: &str) -> String {
        // TODO
        base_name.to_owned()
    }

    fn query_qualifier(&self) -> Option<String> {
        // TODO
        Some("AND wallet_id = $$".to_owned())
    }
}

impl WalletStrategy for DatabasePerWalletStrategy {
    // initialize storage based on wallet storage strategy
    fn init_storage(&self, _config: &PostgresConfig, _credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // no-op
        debug!("Initializing storage strategy DatabasePerWalletStrategy.");
        Ok(())
    }
    // initialize a single wallet based on wallet storage strategy
    fn create_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials, metadata: &[u8]) -> Result<(), WalletStorageError> {
        // create database for wallet
        // if admin user and password aren't provided then bail
        if credentials.admin_account == None || credentials.admin_password == None {
            return Ok(());
        }

        debug!("Entering create_wallet");
        debug!("setting up the admin_postgres_url");
        let url_base = PostgresStorageType::_admin_postgres_url(&config, &credentials);
        let url = PostgresStorageType::_postgres_url(id, &config, &credentials);

        debug!("connecting to postgres, url_base: {:?}", url_base);
        let conn = postgres::Connection::connect(&url_base[..], config.tls(&self.ssl_config))?;

        debug!("creating wallets DB");
        let create_db_sql = str::replace(_CREATE_WALLET_DATABASE, "$1", id);
        let mut schema_result = match conn.execute(&create_db_sql, &[]) {
            Ok(_) => Ok(()),
            Err(_error) => {
                Err(WalletStorageError::AlreadyExists)
            }
        };
        conn.finish()?;

        debug!("connecting to wallet as user");
        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        debug!("setting up multi schema");
        for sql in &_CREATE_SCHEMA {
            match schema_result {
                Ok(_) => schema_result = match conn.execute(sql, &[]) {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        Err(WalletStorageError::IOError(format!("Error occurred while creating wallet schema: {}", error)))
                    }
                },
                _ => ()
            }
        };
        debug!("inserting the keys");
        let ret = match schema_result {
            Ok(_) => {
                match conn.execute("INSERT INTO metadata(value) VALUES($1)
                                    ON CONFLICT (value) DO UPDATE SET value = excluded.value",
                                   &[&metadata]) {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        //std::fs::remove_file(db_path)?;
                        Err(WalletStorageError::IOError(format!("Error occurred while inserting the keys: {}", error)))
                    }
                }
            }
            Err(error) => Err(error)
        };
        conn.finish()?;
        ret
    }
    // open a wallet based on wallet storage strategy
    fn open_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<Box<PostgresStorage>, WalletStorageError> {
        let url = PostgresStorageType::_postgres_url(id, &config, &credentials);

        // don't need a connection, but connect just to verify we can
        let _conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(_) => return Err(WalletStorageError::NotFound)
        };

        // TODO close _conn
        let manager = match PostgresConnectionManager::new(&url[..], config.r2d2_tls()) {
            Ok(manager) => manager,
            Err(_) => return Err(WalletStorageError::NotFound)
        };
        let pool = match r2d2::Pool::builder()
            .min_idle(Some(config.min_idle_count()))
            .max_size(config.max_connections())
            .idle_timeout(Some(Duration::new(config.connection_timeout(), 0)))
            .build(manager) {
            Ok(pool) => pool,
            Err(_) => return Err(WalletStorageError::NotFound)
        };
        Ok(Box::new(PostgresStorage {
            pool: pool,
            wallet_id: id.to_string(),
        }))
    }
    // delete a single wallet based on wallet storage strategy
    fn delete_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // if admin user and password aren't provided then bail
        if credentials.admin_account == None || credentials.admin_password == None {
            return Ok(());
        }

        let url_base = PostgresStorageType::_admin_postgres_url(&config, &credentials);
        let url = PostgresStorageType::_postgres_url(id, &config, &credentials);

        match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => {
                for sql in &_DROP_SCHEMA {
                    match conn.execute(sql, &[]) {
                        Ok(_) => (),
                        Err(_) => ()
                    };
                }
                let _ret = conn.finish();
                ()
            }
            Err(_) => return Err(WalletStorageError::NotFound)
        };

        let conn = postgres::Connection::connect(url_base, config.tls(&self.ssl_config))?;
        let drop_db_sql = str::replace(_DROP_WALLET_DATABASE, "$1", id);
        let ret = match conn.execute(&drop_db_sql, &[]) {
            Ok(_) => Ok(()),
            Err(_) => Ok(())
        };
        conn.finish()?;
        ret
    }
    // determine phyisical table name based on wallet strategy
    fn table_name(&self, _id: &str, base_name: &str) -> String {
        // TODO
        base_name.to_owned()
    }
    // determine additional query parameters based on wallet strategy
    fn query_qualifier(&self) -> Option<String> {
        // TODO
        None
    }
}




// determine additional query parameters based on wallet strategy
fn get_multi_database_name(config: &PostgresConfig) -> &str {
    // look to see if there is a specified db to use.  If not, use the default name
    let name: &str = match config.database_name {
        Some(ref database_name) => {
            if database_name.is_empty() {
                _WALLETS_DB
            } else {
                database_name
            }
        },
        None => _WALLETS_DB,
    };

    return name;
}

impl WalletStrategy for MultiWalletSingleTableStrategy {
    // initialize storage based on wallet storage strategy
    fn init_storage(&self, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        debug!("Entering init_storage");
        // create database and tables for storage
        // if admin user and password aren't provided then bail
        debug!("Initializing storage strategy MultiWalletSingleTableStrategy.");
        if credentials.admin_account == None || credentials.admin_password == None {
            return Ok(());
        }

        debug!("setting up the admin_postgres_url");
        // look to see if there is a specified db to use.  If not, use the default name
        let wallet_db_name: &str = get_multi_database_name(config);
        debug!("wallet_db_name: {:?}", wallet_db_name);
        let url_base = PostgresStorageType::_admin_postgres_url(&config, &credentials);
        let url = PostgresStorageType::_postgres_url(wallet_db_name, &config, &credentials);
        debug!("postgres_url: {:?}", url);
        debug!("connecting to postgres, url_base: {:?}", url_base);
        let conn = postgres::Connection::connect(&url_base[..], config.tls(&self.ssl_config))?;

        debug!("creating wallets DB");
        let create_db_sql: String = str::replace(_CREATE_WALLET_DATABASE, "$1", wallet_db_name);
        debug!("create_db_sql: {:?}", create_db_sql);
        if let Err(error) = conn.execute(&create_db_sql, &[]) {
            if error.code() != Some(&postgres::error::DUPLICATE_DATABASE) {
                debug!("error creating database, Error: {}", error);
                conn.finish()?;
                return Err(WalletStorageError::IOError(format!("Error occurred while creating the database: {}", error)));
            } else {
                // if database already exists, assume tables are created already and return
                debug!("database already exists");
                conn.finish()?;
                return Ok(());
            }
        }
        conn.finish()?;

        debug!("connecting to wallet storage");
        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                debug!("error connecting to wallet storage, Error: {}", error);
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        debug!("setting up multi schema");
        for sql in &_CREATE_SCHEMA_MULTI {
            if let Err(error) = conn.execute(sql, &[]) {
                debug!("error creating wallet schema, Error: {}", error);
                conn.finish()?;
                return Err(WalletStorageError::IOError(format!("Error occurred while creating wallet schema: {}", error)));
            }
        }
        conn.finish()?;
        Ok(())
    }
    // initialize a single wallet based on wallet storage strategy
    fn create_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials, metadata: &[u8]) -> Result<(), WalletStorageError> {
        // look to see if there is a specified db to use.  If not, use the default name
        let wallet_db_name: &str = get_multi_database_name(config);
        debug!("wallet_db_name: {:?}", wallet_db_name);

        // insert metadata
        let url = PostgresStorageType::_postgres_url(wallet_db_name, &config, &credentials);

        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        // We allow error on conflict since this indicates AlreadyExists error
        let ret = match conn.execute("INSERT INTO metadata(wallet_id, value) VALUES($1, $2)", &[&id, &metadata]) {
            Ok(_) => Ok(()),
            Err(error) => {
                if error.code() == Some(&postgres::error::UNIQUE_VIOLATION) {
                    Err(WalletStorageError::AlreadyExists)
                } else {
                    Err(WalletStorageError::IOError(format!("Error occurred while inserting into metadata: {}", error)))
                }
            }
        };
        conn.finish()?;
        ret
    }
    // open a wallet based on wallet storage strategy
    fn open_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<Box<PostgresStorage>, WalletStorageError> {
        // look to see if there is a specified db to use.  If not, use the default name
        let wallet_db_name: &str = get_multi_database_name(config);
        debug!("wallet_db_name: {:?}", wallet_db_name);

        let url = PostgresStorageType::_postgres_url(wallet_db_name, &config, &credentials);

        // don't need a connection, but connect just to verify we can
        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(_) => return Err(WalletStorageError::NotFound)
        };

        // select metadata for this wallet to ensure it exists
        let res: Result<Vec<u8>, WalletStorageError> = {
            let mut rows = conn.query(
                "SELECT value FROM metadata WHERE wallet_id = $1",
                &[&id]);
            match rows.as_mut().unwrap().iter().next() {
                Some(row) => Ok(row.get(0)),
                None => Err(WalletStorageError::ItemNotFound)
            }
        };

        match res {
            Ok(_entity) => (),
            Err(_) => return Err(WalletStorageError::NotFound)
        };

        // TODO close conn

        let manager = match PostgresConnectionManager::new(&url[..], config.r2d2_tls()) {
            Ok(manager) => manager,
            Err(_) => return Err(WalletStorageError::NotFound)
        };
        let pool = match r2d2::Pool::builder()
            .min_idle(Some(config.min_idle_count()))
            .max_size(config.max_connections())
            .idle_timeout(Some(Duration::new(config.connection_timeout(), 0)))
            .build(manager) {
            Ok(pool) => pool,
            Err(_) => return Err(WalletStorageError::NotFound)
        };

        Ok(Box::new(PostgresStorage {
            pool: pool,
            wallet_id: id.to_string(),
        }))
    }
    // delete a single wallet based on wallet storage strategy
    fn delete_wallet(&self, id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // look to see if there is a specified db to use.  If not, use the default name
        let wallet_db_name: &str = get_multi_database_name(config);
        debug!("wallet_db_name: {:?}", wallet_db_name);
        let url = PostgresStorageType::_postgres_url(wallet_db_name, &config, &credentials);

        let conn = match postgres::Connection::connect(&url[..], config.tls(&self.ssl_config)) {
            Ok(conn) => conn,
            Err(error) => {
                return Err(WalletStorageError::IOError(format!("Error occurred while connecting to wallet schema: {}", error)));
            }
        };

        let mut ret = Ok(());
        for sql in &_DELETE_WALLET_MULTI {
            ret = match conn.execute(sql, &[&id]) {
                Ok(row_count) => {
                    if row_count == 0 {
                        Err(WalletStorageError::NotFound)
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    Err(WalletStorageError::IOError(format!("Error occurred while deleting wallet: {}", error)))
                }
            }
        };
        conn.finish()?;
        return ret;
    }
    // determine phyisical table name based on wallet strategy
    fn table_name(&self, _id: &str, base_name: &str) -> String {
        // TODO
        base_name.to_owned()
    }
    // determine additional query parameters based on wallet strategy
    fn query_qualifier(&self) -> Option<String> {
        // TODO
        Some("AND wallet_id = $$".to_owned())
    }
}

impl WalletStrategy for MultiWalletMultiTableStrategy {
    // initialize storage based on wallet storage strategy
    fn init_storage(&self, _config: &PostgresConfig, _credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // create database for storage
        // TODO
        Ok(())
    }
    // initialize a single wallet based on wallet storage strategy
    fn create_wallet(&self, _id: &str, _config: &PostgresConfig, _credentials: &PostgresCredentials, _metadata: &[u8]) -> Result<(), WalletStorageError> {
        // create tables for wallet storage
        // TODO
        Ok(())
    }
    // open a wallet based on wallet storage strategy
    fn open_wallet(&self, _id: &str, _config: &PostgresConfig, _credentials: &PostgresCredentials) -> Result<Box<PostgresStorage>, WalletStorageError> {
        // TODO
        Err(WalletStorageError::NotFound)
    }
    // delete a single wallet based on wallet storage strategy
    fn delete_wallet(&self, _id: &str, _config: &PostgresConfig, _credentials: &PostgresCredentials) -> Result<(), WalletStorageError> {
        // TODO
        Ok(())
    }
    // determine phyisical table name based on wallet strategy
    fn table_name(&self, _id: &str, base_name: &str) -> String {
        // TODO
        base_name.to_owned()
    }
    // determine additional query parameters based on wallet strategy
    fn query_qualifier(&self) -> Option<String> {
        // TODO
        None
    }
}

lazy_static! {
    static ref SELECTED_STRATEGY: RwLock<Box<dyn WalletStrategy + Send + Sync>> = RwLock::new(Box::new(DatabasePerWalletStrategy{ ssl_config: None }));
    static ref INITIALIZED: RwLock<bool>= RwLock::new(false);
}

impl PostgresStorageType {
    pub fn new() -> PostgresStorageType {
        PostgresStorageType {}
    }

    fn _admin_postgres_url(config: &PostgresConfig, credentials: &PostgresCredentials) -> String {
        let mut url_base = "postgresql://".to_owned();

        match credentials.admin_account {
            Some(ref account) => url_base.push_str(&account[..]),
            None => ()
        }
        url_base.push_str(":");
        match credentials.admin_password {
            Some(ref password) => url_base.push_str(&password[..]),
            None => ()
        }
        url_base.push_str("@");
        url_base.push_str(&config.url[..]);
        url_base
    }

    fn _base_postgres_url(config: &PostgresConfig, credentials: &PostgresCredentials) -> String {
        let mut url_base = "postgresql://".to_owned();
        url_base.push_str(&credentials.account[..]);
        url_base.push_str(":");
        url_base.push_str(&credentials.password[..]);
        url_base.push_str("@");
        url_base.push_str(&config.url[..]);
        url_base
    }

    fn _postgres_url(id: &str, config: &PostgresConfig, credentials: &PostgresCredentials) -> String {
        let mut url_base = PostgresStorageType::_base_postgres_url(config, credentials);
        url_base.push_str("/");
        url_base.push_str(id);
        url_base
    }
}


impl WalletStorage for PostgresStorage {
    ///
    /// Tries to fetch values and/or tags from the storage.
    /// Returns Result with StorageEntity object which holds requested data in case of success or
    /// Result with WalletStorageError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type_ of the item in storage
    ///  * `id` - id of the item in storage
    ///  * `options` - JSon containing what needs to be fetched.
    ///  Example: {"retrieveValue": true, "retrieveTags": true}
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `StorageEntity` - Contains name, optional value and optional tags
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::Closed` - Storage is closed
    ///  * `WalletStorageError::ItemNotFound` - Item is not found in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    ///
    fn get(&self, type_: &[u8], id: &[u8], options: &str) -> Result<StorageRecord, WalletStorageError> {
        let options: RecordOptions = if options == "{}" { // FIXME:
            RecordOptions::default()
        } else {
            serde_json::from_str(options)?
        };
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let res: Result<(i64, Vec<u8>, Vec<u8>), WalletStorageError> = {
            let mut rows = match query_qualifier {
                Some(_) => conn.query(
                    "SELECT id, value, key FROM items where type = $1 AND name = $2 AND wallet_id = $3",
                    &[&type_.to_vec(), &id.to_vec(), &self.wallet_id]),
                None => conn.query(
                    "SELECT id, value, key FROM items where type = $1 AND name = $2",
                    &[&type_.to_vec(), &id.to_vec()])
            };
            match rows.as_mut().unwrap().iter().next() {
                Some(row) => Ok((row.get(0), row.get(1), row.get(2))),
                None => Err(WalletStorageError::ItemNotFound)
            }
        };
        let item = match res {
            Ok(entity) => entity,
            Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
            Err(err) => return Err(WalletStorageError::from(err))
        };
        let value = if options.retrieve_value
        { Some(EncryptedValue::new(item.1, item.2)) } else { None };
        let type_ = if options.retrieve_type { Some(type_.clone()) } else { None };
        let tags = if options.retrieve_tags {
            let mut tags = Vec::new();

            // get all encrypted.
            let rows = match query_qualifier {
                Some(_) => {
                    let stmt = conn.prepare_cached("SELECT name, value FROM tags_encrypted WHERE item_id = $1 AND wallet_id = $2")?;
                    stmt.query(&[&item.0, &self.wallet_id])?
                }
                None => {
                    let stmt = conn.prepare_cached("SELECT name, value FROM tags_encrypted WHERE item_id = $1")?;
                    stmt.query(&[&item.0])?
                }
            };

            let mut iter = rows.iter();
            while let Some(res) = iter.next() {
                let row = res;
                //let tag_name: Vec<u8> = row.get(0);
                //let tag_value: Vec<u8> = row.get(1);
                tags.push(Tag::Encrypted(row.get(0), row.get(1)));
            }

            // get all plain
            let rows = match query_qualifier {
                Some(_) => {
                    let stmt = conn.prepare_cached("SELECT name, value FROM tags_plaintext WHERE item_id = $1 AND wallet_id = $2")?;
                    stmt.query(&[&item.0, &self.wallet_id])?
                }
                None => {
                    let stmt = conn.prepare_cached("SELECT name, value FROM tags_plaintext WHERE item_id = $1")?;
                    stmt.query(&[&item.0])?
                }
            };

            let mut iter = rows.iter();
            while let Some(res) = iter.next() {
                let row = res;
                //let tag_name: Vec<u8> = row.get(0);
                //let tag_value: String = row.get(1);
                tags.push(Tag::PlainText(row.get(0), row.get(1)));
            }
            Some(tags)
        } else { None };
        Ok(StorageRecord::new(id.to_vec(), value, type_.map(|val| val.to_vec()), tags))
    }

    ///
    /// inserts value and tags into storage.
    /// Returns Result with () on success or
    /// Result with WalletStorageError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type of the item in storage
    ///  * `id` - id of the item in storage
    ///  * `value` - value of the item in storage
    ///  * `value_key` - key used to encrypt the value
    ///  * `tags` - tags assigned to the value
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` class of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::Closed` - Storage is closed
    ///  * `WalletStorageError::ItemAlreadyExists` - Item is already present in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    ///
    fn add(&self, type_: &[u8], id: &[u8], value: &EncryptedValue, tags: &[Tag]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let tx: transaction::Transaction = transaction::Transaction::new(&conn)?;
        let res = match query_qualifier {
            Some(_) => tx.prepare_cached("INSERT INTO items (type, name, value, key, wallet_id) VALUES ($1, $2, $3, $4, $5) RETURNING id")?
                .query(&[&type_.to_vec(), &id.to_vec(), &value.data, &value.key, &self.wallet_id]),
            None => tx.prepare_cached("INSERT INTO items (type, name, value, key) VALUES ($1, $2, $3, $4) RETURNING id")?
                .query(&[&type_.to_vec(), &id.to_vec(), &value.data, &value.key])
        };

        let item_id = match res {
            Ok(rows) => {
                let res = match rows.iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                };
                let item_id: i64 = match res {
                    Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
                    Err(err) => return Err(WalletStorageError::from(err)),
                    Ok(id) => id
                };
                item_id
            }
            Err(err) => {
                if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) ||
                    err.code() == Some(&postgres::error::INTEGRITY_CONSTRAINT_VIOLATION) {
                    return Err(WalletStorageError::ItemAlreadyExists);
                } else {
                    return Err(WalletStorageError::from(err));
                }
            }
        };

        let item_id = item_id as i64;

        if !tags.is_empty() {
            let stmt_e = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)")?,
                None => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value) VALUES ($1, $2, $3)")?
            };
            let stmt_p = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)")?,
                None => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value) VALUES ($1, $2, $3)")?
            };

            for tag in tags {
                match tag {
                    &Tag::Encrypted(ref tag_name, ref tag_data) => {
                        let res = match query_qualifier {
                            Some(_) => stmt_e.execute(&[&item_id, tag_name, tag_data, &self.wallet_id]),
                            None => stmt_e.execute(&[&item_id, tag_name, tag_data])
                        };
                        match res {
                            Ok(_) => (),
                            Err(err) => {
                                if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) ||
                                    err.code() == Some(&postgres::error::INTEGRITY_CONSTRAINT_VIOLATION) {
                                    return Err(WalletStorageError::ItemAlreadyExists);
                                } else {
                                    return Err(WalletStorageError::from(err));
                                }
                            }
                        }
                    }
                    &Tag::PlainText(ref tag_name, ref tag_data) => {
                        let res = match query_qualifier {
                            Some(_) => stmt_p.execute(&[&item_id, tag_name, tag_data, &self.wallet_id]),
                            None => stmt_p.execute(&[&item_id, tag_name, tag_data])
                        };
                        match res {
                            Ok(_) => (),
                            Err(err) => {
                                if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) ||
                                    err.code() == Some(&postgres::error::INTEGRITY_CONSTRAINT_VIOLATION) {
                                    return Err(WalletStorageError::ItemAlreadyExists);
                                } else {
                                    return Err(WalletStorageError::from(err));
                                }
                            }
                        }
                    }
                };
            }
        }

        tx.commit()?;

        Ok(())
    }

    fn update(&self, type_: &[u8], id: &[u8], value: &EncryptedValue) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let res = match query_qualifier {
            Some(_) => conn.prepare_cached("UPDATE items SET value = $1, key = $2 WHERE type = $3 AND name = $4 AND wallet_id = $5")?
                .execute(&[&value.data, &value.key, &type_.to_vec(), &id.to_vec(), &self.wallet_id]),
            None => conn.prepare_cached("UPDATE items SET value = $1, key = $2 WHERE type = $3 AND name = $4")?
                .execute(&[&value.data, &value.key, &type_.to_vec(), &id.to_vec()])
        };

        match res {
            Ok(1) => Ok(()),
            Ok(0) => Err(WalletStorageError::ItemNotFound),
            Ok(count) => Err(WalletStorageError::CommonError(CommonError::InvalidState(format!("Postgres returned update row count: {}", count)))),
            Err(err) => Err(WalletStorageError::from(err)),
        }
    }

    fn add_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let tx: transaction::Transaction = transaction::Transaction::new(&conn)?;

        let res = match query_qualifier {
            Some(_) => {
                let mut rows = tx.prepare_cached("SELECT id FROM items WHERE type = $1 AND name = $2")?
                    .query(&[&type_.to_vec(), &id.to_vec()]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
            None => {
                let mut rows = tx.prepare_cached("SELECT id FROM items WHERE type = $1 AND name = $2")?
                    .query(&[&type_.to_vec(), &id.to_vec()]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
        };

        let item_id: i64 = match res {
            Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
            Err(err) => return Err(WalletStorageError::from(err)),
            Ok(id) => id
        };

        if !tags.is_empty() {
            let enc_tag_insert_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)
                                        ON CONFLICT (name, item_id, wallet_id) DO UPDATE SET value = excluded.value")?,
                None => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value) VALUES ($1, $2, $3)
                                        ON CONFLICT (name, item_id) DO UPDATE SET value = excluded.value")?
            };
            let plain_tag_insert_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)
                                            ON CONFLICT (name, item_id, wallet_id) DO UPDATE SET value = excluded.value")?,
                None => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value) VALUES ($1, $2, $3)
                                            ON CONFLICT (name, item_id) DO UPDATE SET value = excluded.value")?
            };

            for tag in tags {
                match tag {
                    &Tag::Encrypted(ref tag_name, ref tag_data) => {
                        let res = match query_qualifier {
                            Some(_) => enc_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data, &self.wallet_id]),
                            None => enc_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data])
                        };
                        match res {
                            Ok(_) => (),
                            Err(err) => {
                                if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) ||
                                    err.code() == Some(&postgres::error::INTEGRITY_CONSTRAINT_VIOLATION) {
                                    return Err(WalletStorageError::ItemAlreadyExists);
                                } else {
                                    return Err(WalletStorageError::from(err));
                                }
                            }
                        }
                    }
                    &Tag::PlainText(ref tag_name, ref tag_data) => {
                        let res = match query_qualifier {
                            Some(_) => plain_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data, &self.wallet_id]),
                            None => plain_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data])
                        };
                        match res {
                            Ok(_) => (),
                            Err(err) => {
                                if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) ||
                                    err.code() == Some(&postgres::error::INTEGRITY_CONSTRAINT_VIOLATION) {
                                    return Err(WalletStorageError::ItemAlreadyExists);
                                } else {
                                    return Err(WalletStorageError::from(err));
                                }
                            }
                        }
                    }
                };
            }
        }
        tx.commit()?;

        Ok(())
    }

    fn update_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let tx: transaction::Transaction = transaction::Transaction::new(&conn)?;

        let res = match query_qualifier {
            Some(_) => {
                let mut rows = tx.prepare_cached("SELECT id FROM items WHERE type = $1 AND name = $2 AND wallet_id = $3")?
                    .query(&[&type_.to_vec(), &id.to_vec(), &self.wallet_id]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
            None => {
                let mut rows = tx.prepare_cached("SELECT id FROM items WHERE type = $1 AND name = $2")?
                    .query(&[&type_.to_vec(), &id.to_vec()]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
        };

        let item_id: i64 = match res {
            Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
            Err(err) => return Err(WalletStorageError::from(err)),
            Ok(id) => id
        };

        match query_qualifier {
            Some(_) => {
                tx.execute("DELETE FROM tags_encrypted WHERE item_id = $1 AND wallet_id = $2", &[&item_id, &self.wallet_id])?;
                tx.execute("DELETE FROM tags_plaintext WHERE item_id = $1 AND wallet_id = $2", &[&item_id, &self.wallet_id])?;
            }
            None => {
                tx.execute("DELETE FROM tags_encrypted WHERE item_id = $1", &[&item_id])?;
                tx.execute("DELETE FROM tags_plaintext WHERE item_id = $1", &[&item_id])?;
            }
        };

        if !tags.is_empty() {
            let enc_tag_insert_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)")?,
                None => tx.prepare_cached("INSERT INTO tags_encrypted (item_id, name, value) VALUES ($1, $2, $3)")?
            };
            let plain_tag_insert_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value, wallet_id) VALUES ($1, $2, $3, $4)")?,
                None => tx.prepare_cached("INSERT INTO tags_plaintext (item_id, name, value) VALUES ($1, $2, $3)")?
            };

            for tag in tags {
                match query_qualifier {
                    Some(_) => {
                        match tag {
                            &Tag::Encrypted(ref tag_name, ref tag_data) => enc_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data, &self.wallet_id])?,
                            &Tag::PlainText(ref tag_name, ref tag_data) => plain_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data, &self.wallet_id])?
                        }
                    }
                    None => {
                        match tag {
                            &Tag::Encrypted(ref tag_name, ref tag_data) => enc_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data])?,
                            &Tag::PlainText(ref tag_name, ref tag_data) => plain_tag_insert_stmt.execute(&[&item_id, tag_name, tag_data])?
                        }
                    }
                };
            }
        }
        tx.commit()?;

        Ok(())
    }

    fn delete_tags(&self, type_: &[u8], id: &[u8], tag_names: &[TagName]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let res = match query_qualifier {
            Some(_) => {
                let mut rows = conn.prepare_cached("SELECT id FROM items WHERE type =$1 AND name = $2 AND wallet_id = $3")?
                    .query(&[&type_.to_vec(), &id.to_vec(), &self.wallet_id]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
            None => {
                let mut rows = conn.prepare_cached("SELECT id FROM items WHERE type =$1 AND name = $2")?
                    .query(&[&type_.to_vec(), &id.to_vec()]);
                match rows.as_mut().unwrap().iter().next() {
                    Some(row) => Ok(row.get(0)),
                    None => Err(WalletStorageError::ItemNotFound)
                }
            }
        };

        let item_id: i64 = match res {
            Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
            Err(err) => return Err(WalletStorageError::from(err)),
            Ok(id) => id
        };

        let tx: transaction::Transaction = transaction::Transaction::new(&conn)?;
        {
            let enc_tag_delete_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("DELETE FROM tags_encrypted WHERE item_id = $1 AND name = $2 AND wallet_id = $3")?,
                None => tx.prepare_cached("DELETE FROM tags_encrypted WHERE item_id = $1 AND name = $2")?
            };
            let plain_tag_delete_stmt = match query_qualifier {
                Some(_) => tx.prepare_cached("DELETE FROM tags_plaintext WHERE item_id = $1 AND name = $2 AND wallet_id = $3")?,
                None => tx.prepare_cached("DELETE FROM tags_plaintext WHERE item_id = $1 AND name = $2")?
            };

            for tag_name in tag_names {
                match query_qualifier {
                    Some(_) =>
                        match tag_name {
                            &TagName::OfEncrypted(ref tag_name) => enc_tag_delete_stmt.execute(&[&item_id, tag_name, &self.wallet_id])?,
                            &TagName::OfPlain(ref tag_name) => plain_tag_delete_stmt.execute(&[&item_id, tag_name, &self.wallet_id])?,
                        },
                    None =>
                        match tag_name {
                            &TagName::OfEncrypted(ref tag_name) => enc_tag_delete_stmt.execute(&[&item_id, tag_name])?,
                            &TagName::OfPlain(ref tag_name) => plain_tag_delete_stmt.execute(&[&item_id, tag_name])?,
                        }
                };
            }
        }
        tx.commit()?;

        Ok(())
    }

    ///
    /// deletes value and tags into storage.
    /// Returns Result with () on success or
    /// Result with WalletStorageError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type of the item in storage
    ///  * `id` - id of the item in storage
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::Closed` - Storage is closed
    ///  * `WalletStorageError::ItemNotFound` - Item is not found in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    ///
    fn delete(&self, type_: &[u8], id: &[u8]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let row_count = match query_qualifier {
            Some(_) => conn.execute(
                "DELETE FROM items where type = $1 AND name = $2 AND wallet_id = $3",
                &[&type_.to_vec(), &id.to_vec(), &self.wallet_id],
            )?,
            None => conn.execute(
                "DELETE FROM items where type = $1 AND name = $2",
                &[&type_.to_vec(), &id.to_vec()],
            )?
        };
        if row_count == 1 {
            Ok(())
        } else {
            Err(WalletStorageError::ItemNotFound)
        }
    }

    fn get_storage_metadata(&self) -> Result<Vec<u8>, WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let res: Result<Vec<u8>, WalletStorageError> = {
            let mut rows = match query_qualifier {
                Some(_) => conn.query(
                    "SELECT value FROM metadata WHERE wallet_id = $1",
                    &[&self.wallet_id]),
                None => conn.query(
                    "SELECT value FROM metadata",
                    &[])
            };
            match rows.as_mut().unwrap().iter().next() {
                Some(row) => Ok(row.get(0)),
                None => Err(WalletStorageError::ItemNotFound)
            }
        };

        match res {
            Ok(entity) => Ok(entity),
            Err(WalletStorageError::ItemNotFound) => return Err(WalletStorageError::ItemNotFound),
            Err(err) => return Err(WalletStorageError::from(err))
        }
    }

    fn set_storage_metadata(&self, metadata: &[u8]) -> Result<(), WalletStorageError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let res = match query_qualifier {
            Some(_) => conn.execute("UPDATE metadata SET value = $1 WHERE wallet_id = $2", &[&metadata.to_vec(), &self.wallet_id]),
            None => conn.execute("UPDATE metadata SET value = $1", &[&metadata.to_vec()])
        };
        match res {
            Ok(_) => Ok(()),
            Err(error) => {
                Err(WalletStorageError::IOError(format!("Error occurred while inserting the keys: {}", error)))
            }
        }
    }

    fn get_all(&self) -> Result<Box<dyn StorageIterator>, WalletStorageError> {
        let query_qualifier = get_wallet_strategy_qualifier();
        let statement = match query_qualifier {
            Some(_) => self._prepare_statement("SELECT id, name, value, key, type FROM items WHERE wallet_id = $1")?,
            None => self._prepare_statement("SELECT id, name, value, key, type FROM items")?
        };
        let fetch_options = RecordOptions {
            retrieve_type: true,
            retrieve_value: true,
            retrieve_tags: true,
        };
        let pool = self.pool.clone();
        let tag_retriever = match query_qualifier {
            Some(_) => Some(TagRetriever::new_owned(Rc::new(pool.get().unwrap()).clone(), Some(self.wallet_id.clone()))?),
            None => Some(TagRetriever::new_owned(Rc::new(pool.get().unwrap()).clone(), None)?)
        };

        let storage_iterator = match query_qualifier {
            Some(_) => PostgresStorageIterator::new(Some(statement), &[&self.wallet_id], fetch_options, tag_retriever, None)?,
            None => PostgresStorageIterator::new(Some(statement), &[], fetch_options, tag_retriever, None)?
        };
        Ok(Box::new(storage_iterator))
    }

    fn search(&self, type_: &[u8], query: &language::Operator, options: Option<&str>) -> Result<Box<dyn StorageIterator>, WalletStorageError> {
        let type_ = type_.to_vec(); // FIXME

        let search_options = match options {
            None => SearchOptions::default(),
            Some(option_str) => serde_json::from_str(option_str)?
        };

        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        let query_qualifier = get_wallet_strategy_qualifier();
        let wallet_id_arg = self.wallet_id.to_owned();
        let total_count: Option<usize> = if search_options.retrieve_total_count {
            let (query_string, query_arguments) = match query_qualifier {
                Some(_) => {
                    let (mut query_string, mut query_arguments) = query::wql_to_sql_count(&type_, query)?;
                    query_arguments.push(&wallet_id_arg);
                    let arg_str = format!(" AND i.wallet_id = ${}", query_arguments.len());
                    query_string.push_str(&arg_str);
                    let mut with_clause = false;
                    if query_string.contains("tags_plaintext") {
                        query_arguments.push(&wallet_id_arg);
                        query_string = format!("tags_plaintext as (select * from tags_plaintext where wallet_id = ${}) {}", query_arguments.len(), query_string);
                        with_clause = true;
                    }
                    if query_string.contains("tags_encrypted") {
                        if with_clause {
                            query_string = format!(", {}", query_string);
                        }
                        query_arguments.push(&wallet_id_arg);
                        query_string = format!("tags_encrypted as (select * from tags_encrypted where wallet_id = ${}) {}", query_arguments.len(), query_string);
                        with_clause = true;
                    }
                    if with_clause {
                        query_string = format!("WITH {}", query_string);
                    }
                    (query_string, query_arguments)
                }
                None => query::wql_to_sql_count(&type_, query)?
            };

            let mut rows = conn.query(
                &query_string,
                &query_arguments[..]);
            match rows.as_mut().unwrap().iter().next() {
                Some(row) => {
                    let x: i64 = row.get(0);
                    Some(x as usize)
                }
                None => None
            }
        } else { None };

        if search_options.retrieve_records {
            let fetch_options = RecordOptions {
                retrieve_value: search_options.retrieve_value,
                retrieve_tags: search_options.retrieve_tags,
                retrieve_type: search_options.retrieve_type,
            };

            let (query_string, query_arguments) = match query_qualifier {
                Some(_) => {
                    let (mut query_string, mut query_arguments) = query::wql_to_sql(&type_, query, options)?;
                    query_arguments.push(&wallet_id_arg);
                    let arg_str = format!(" AND i.wallet_id = ${}", query_arguments.len());
                    query_string.push_str(&arg_str);
                    let mut with_clause = false;
                    if query_string.contains("tags_plaintext") {
                        query_arguments.push(&wallet_id_arg);
                        query_string = format!("tags_plaintext as (select * from tags_plaintext where wallet_id = ${}) {}", query_arguments.len(), query_string);
                        with_clause = true;
                    }
                    if query_string.contains("tags_encrypted") {
                        if with_clause {
                            query_string = format!(", {}", query_string);
                        }
                        query_arguments.push(&wallet_id_arg);
                        query_string = format!("tags_encrypted as (select * from tags_encrypted where wallet_id = ${}) {}", query_arguments.len(), query_string);
                        with_clause = true;
                    }
                    if with_clause {
                        query_string = format!("WITH {}", query_string);
                    }
                    (query_string, query_arguments)
                }
                None => query::wql_to_sql(&type_, query, options)?
            };

            let statement = self._prepare_statement(&query_string)?;
            let tag_retriever = if fetch_options.retrieve_tags {
                let pool = self.pool.clone();
                match query_qualifier {
                    Some(_) => Some(TagRetriever::new_owned(Rc::new(pool.get().unwrap()).clone(), Some(self.wallet_id.clone()))?),
                    None => Some(TagRetriever::new_owned(Rc::new(pool.get().unwrap()).clone(), None)?)
                }
            } else {
                None
            };
            let storage_iterator = PostgresStorageIterator::new(Some(statement), &query_arguments[..], fetch_options, tag_retriever, total_count)?;
            Ok(Box::new(storage_iterator))
        } else {
            let storage_iterator = PostgresStorageIterator::new(None, &[], RecordOptions::default(), None, total_count)?;
            Ok(Box::new(storage_iterator))
        }
    }

    fn close(&mut self) -> Result<(), WalletStorageError> {
        // TODO throws a borrow error if we try to close the connection here; temporary workaround is to rely on idle connection timeout
        Ok(())
    }
}

impl PostgresStorage {
    fn _prepare_statement(&self, sql: &str) -> Result<
        OwningHandle<Rc<r2d2::PooledConnection<PostgresConnectionManager>>, Box<postgres::stmt::Statement<'static>>>,
        WalletStorageError> {
        let pool = self.pool.clone();
        OwningHandle::try_new(Rc::new(pool.get().unwrap()).clone(), |conn| {
            unsafe { (*conn).prepare(sql) }.map(Box::new).map_err(WalletStorageError::from)
        })
    }
}

fn create_connection_pool(config: &PostgresConfig, credentials: &PostgresCredentials) -> Result<Pool<PostgresConnectionManager>, WalletStorageError> {
    let _url_base = PostgresStorageType::_admin_postgres_url(&config, &credentials);
    let url = PostgresStorageType::_postgres_url(_WALLETS_DB, &config, &credentials);

    let manager = match PostgresConnectionManager::new(&url[..], config.r2d2_tls()) {
        Ok(manager) => manager,
        Err(e) => { return Err(WalletStorageError::GenericError(format!("Problem creating PostgresConnectionManager. Details {:?}", e))); }
    };

    debug!("MultiWalletSingleTableStrategySharedPool open >> building connection pool");
    match r2d2::Pool::builder()
        .min_idle(Some(config.min_idle_count()))
        .max_size(config.max_connections())
        .idle_timeout(Some(Duration::new(config.connection_timeout(), 0)))
        .build(manager) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(WalletStorageError::GenericError(format!("Problem creating PostgresConnectionManager. Details {:?}", e)))
    }
}

fn set_wallet_strategy(strategy: Box<dyn WalletStrategy + Send + Sync>) {
    let mut write_strategy = SELECTED_STRATEGY.write().unwrap();
    *write_strategy = strategy;
}

fn get_wallet_strategy_qualifier() -> Option<String> {
    let read_strategy = SELECTED_STRATEGY.read().unwrap();
    read_strategy.as_ref().query_qualifier()
}

fn set_initialized() {
    if !get_initialized() {
        let mut write_init = INITIALIZED.write().unwrap();
        *write_init = true;
    }
}

fn get_initialized() -> bool {
    *INITIALIZED.read().unwrap()
}


impl WalletStorageType for PostgresStorageType {
    ///
    /// Initializes the wallets database and creates the necessary tables for all wallets
    /// This needs to be called once at the very beginning, I'm not entirely sure the best way to enforce it
    ///
    /// # Arguments
    ///
    ///  * `storage_config` - config containing the location of Postgres DB files
    ///  * `storage_credentials` - DB credentials
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::NotFound` - File with the provided id not found
    ///  * `IOError(..)` - Deletion of the file form the file-system failed
    ///
    fn init_storage(&self, config: Option<&str>, credentials: Option<&str>) -> Result<(), WalletStorageError> {
        if get_initialized() {
            return Ok(());
        }
        let config = config
            .map(serde_json::from_str::<PostgresConfig>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize config: {:?}", err)))?;
        let credentials = credentials
            .map(serde_json::from_str::<PostgresCredentials>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize credentials: {:?}", err)))?;

        let config = match config {
            Some(config) => config,
            None => return Err(WalletStorageError::ConfigError)
        };
        let credentials = match credentials {
            Some(credentials) => credentials,
            None => return Err(WalletStorageError::ConfigError)
        };

        let ssl_config = Some(ssl_config(&config));
        match config.wallet_scheme {
            Some(scheme) => match scheme {
                WalletScheme::DatabasePerWallet => {
                    debug!("Initialising postgresql using DatabasePerWallet strategy.");
                    set_wallet_strategy(Box::new(DatabasePerWalletStrategy { ssl_config: ssl_config }));
                }
                WalletScheme::MultiWalletSingleTable => {
                    debug!("Initialising postgresql using MultiWalletSingleTable strategy.");
                    set_wallet_strategy(Box::new(MultiWalletSingleTableStrategy { ssl_config: ssl_config }));
                }
                WalletScheme::MultiWalletMultiTable => {
                    debug!("Initialising postgresql using MultiWalletMultiTable strategy.");
                    set_wallet_strategy(Box::new(MultiWalletMultiTableStrategy {}));
                }
                WalletScheme::MultiWalletSingleTableSharedPool => {
                    if (&config as &PostgresConfig).min_idle_count() > 0 {
                        // TODO: This restriction can be removed  but we would have make sure that
                        // we don't attempt to construct connection pool before we have constructed
                        // database. Currently we are creating database (if doesn't exists) in
                        // strategy.init_storage(&config, &credentials) at the end of this function.
                        error!("MultiWalletSingleTableSharedPool does not support 'min_idle_count' \
                               (or its deprecated equivalent 'min_idle_time') to be > 0");
                        return Err(ConfigError)
                    }
                    debug!("Initialising postgresql using MultiWalletSingleTableSharedPool strategy.");
                    let pool = create_connection_pool(&config, &credentials)?;
                    set_wallet_strategy(Box::new(MultiWalletSingleTableStrategySharedPool { ssl_config: ssl_config, pool }));
                }
            },
            None => {
                debug!("Initialising postgresql but strategy was not specified in storage \
                        configuration. Using DatabasePerWallet strategy by default.");
                set_wallet_strategy(Box::new(DatabasePerWalletStrategy { ssl_config: ssl_config }));
            }
        };
        let r1 = SELECTED_STRATEGY.read().unwrap();
        let res = r1.as_ref().init_storage(&config, &credentials);
        set_initialized();
        res
    }

    ///
    /// Deletes the Postgres database file with the provided id from the path specified in the
    /// config file.
    ///
    /// # Arguments
    ///
    ///  * `id` - the wallet id
    ///  * `storage_config` - Postgres DB connection config
    ///  * `storage_credentials` - DB credentials
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::NotFound` - File with the provided id not found
    ///  * `IOError(..)` - Deletion of the file form the file-system failed
    ///
    fn delete_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> Result<(), WalletStorageError> {
        let config = config
            .map(serde_json::from_str::<PostgresConfig>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize config: {:?}", err)))?;
        let credentials = credentials
            .map(serde_json::from_str::<PostgresCredentials>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize credentials: {:?}", err)))?;

        let config = match config {
            Some(config) => config,
            None => return Err(WalletStorageError::ConfigError)
        };
        let credentials = match credentials {
            Some(credentials) => credentials,
            None => return Err(WalletStorageError::ConfigError)
        };

        let strategy_read_lock = SELECTED_STRATEGY.read().unwrap();
        strategy_read_lock.as_ref().delete_wallet(id, &config, &credentials)
    }

    ///
    /// Creates the Postgres DB schema with the provided name in the id specified in the config file,
    /// and initializes the encryption keys needed for encryption and decryption of data.
    ///
    /// # Arguments
    ///
    ///  * `id` - name of the Postgres DB schema
    ///  * `config` - config containing the location of postgres db
    ///  * `credentials` - DB credentials
    ///  * `metadata` - encryption keys that need to be stored in the newly created DB
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `AlreadyExists` - Schema with a given name already exists in the database
    ///  * `IOError("IO error during storage operation:...")` - Connection to the DB failed
    ///  * `IOError("Error occurred while creating wallet file:..)"` - Creation of schema failed
    ///  * `IOError("Error occurred while inserting the keys...")` - Insertion of keys failed
    ///
    fn create_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>, metadata: &[u8]) -> Result<(), WalletStorageError> {
        self.init_storage(config, credentials)?;
        let config = config
            .map(serde_json::from_str::<PostgresConfig>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize config: {:?}", err)))?;
        let credentials = credentials
            .map(serde_json::from_str::<PostgresCredentials>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize credentials: {:?}", err)))?;

        let config = match config {
            Some(config) => config,
            None => return Err(WalletStorageError::ConfigError)
        };
        let credentials = match credentials {
            Some(credentials) => credentials,
            None => return Err(WalletStorageError::ConfigError)
        };

        // initialize using the global selected_strategy object
        let r1 = SELECTED_STRATEGY.read().unwrap();
        r1.as_ref().create_wallet(id, &config, &credentials, metadata)
    }

    ///
    /// Establishes a connection to the SQLite DB with the provided id located in the path
    /// specified in the config. In case of a successful onection returns a Storage object
    /// embedding the connection and the encryption keys that will be used for encryption and
    /// decryption operations.
    ///
    ///
    /// # Arguments
    ///
    ///  * `id` - id of the SQLite DB file
    ///  * `config` - config containing the location of SQLite DB files
    ///  * `credentials` - DB credentials
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `(Box<Storage>, Vec<u8>)` - Tuple of `SQLiteStorage` and `encryption keys`
    ///  * `WalletStorageError`
    ///
    /// # Errors
    ///
    /// Any of the following `WalletStorageError` type_ of errors can be throw by this method:
    ///
    ///  * `WalletStorageError::NotFound` - File with the provided id not found
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    ///
    fn open_storage(&self, id: &str, config: Option<&str>, credentials: Option<&str>) -> Result<Box<PostgresStorage>, WalletStorageError> {
        self.init_storage(config, credentials)?;
        let config = config
            .map(serde_json::from_str::<PostgresConfig>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize config: {:?}", err)))?;
        let credentials = credentials
            .map(serde_json::from_str::<PostgresCredentials>)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize credentials: {:?}", err)))?;

        let config = match config
        {
            Some(config) => config,
            None => return Err(WalletStorageError::ConfigError)
        };
        let credentials = match credentials {
            Some(credentials) => credentials,
            None => return Err(WalletStorageError::ConfigError)
        };

        // initialize using the global selected_strategy object
        let r1 = SELECTED_STRATEGY.read().unwrap();
        r1.as_ref().open_wallet(id, &config, &credentials)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use utils::test;

    #[test]
    fn postgres_storage_type_create_works() {
        _cleanup();

        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();
    }

    #[test]
    fn postgres_storage_type_create_works_for_twice() {
        _cleanup();

        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();

        let res = storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata());
        assert_match!(Err(WalletStorageError::AlreadyExists), res);
    }

    #[test]
    fn postgres_storage_get_storage_metadata_works() {
        _cleanup();

        let storage = _storage();
        let metadata = storage.get_storage_metadata().unwrap();

        assert_eq!(metadata, _metadata());
    }

    #[test]
    fn postgres_storage_type_delete_works() {
        _cleanup();

        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();

        storage_type.delete_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..])).unwrap();
    }


    #[test]
    fn postgres_storage_type_delete_works_for_non_existing() {
        _cleanup();

        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();

        let res = storage_type.delete_storage("unknown", Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]));
        assert_match!(Err(WalletStorageError::NotFound), res);

        storage_type.delete_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..])).unwrap();
    }

    #[test]
    fn postgres_storage_type_open_works() {
        _cleanup();
        _storage();
    }

    #[test]
    fn postgres_storage_type_open_works_for_not_created() {
        _cleanup();

        let storage_type = PostgresStorageType::new();

        let res = storage_type.open_storage("unknown", Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]));
        assert_match!(Err(WalletStorageError::NotFound), res);
    }

    #[test]
    fn postgres_storage_add_works_with_config() {
        _cleanup();

        let storage = _storage_db_pool();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
    }

    #[test]
    fn postgres_storage_add_works_for_is_802() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.add(&_type1(), &_id1(), &_value1(), &_tags());
        assert_match!(Err(WalletStorageError::ItemAlreadyExists), res);

        let res = storage.add(&_type1(), &_id1(), &_value1(), &_tags());
        assert_match!(Err(WalletStorageError::ItemAlreadyExists), res);
    }

    #[test]
    fn postgres_storage_set_get_works() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();

        assert_eq!(record.value.unwrap(), _value1());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_tags()));
    }

    #[test]
    fn postgres_storage_set_get_works_for_twice() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.add(&_type1(), &_id1(), &_value2(), &_tags());
        assert_match!(Err(WalletStorageError::ItemAlreadyExists), res);
    }

    #[test]
    fn postgres_storage_set_get_works_for_reopen() {
        _cleanup();

        {
            _storage().add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        }

        let storage_type = PostgresStorageType::new();
        let storage = storage_type.open_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..])).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();

        assert_eq!(record.value.unwrap(), _value1());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_tags()));
    }

    #[test]
    fn postgres_storage_get_works_for_wrong_key() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.get(&_type1(), &_id2(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##);
        assert_match!(Err(WalletStorageError::ItemNotFound), res)
    }

    #[test]
    fn postgres_storage_delete_works() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_tags()));

        storage.delete(&_type1(), &_id1()).unwrap();
        let res = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##);
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    #[test]
    fn postgres_storage_delete_works_for_non_existing() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.delete(&_type1(), &_id2());
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    #[test]
    fn postgres_storage_create_and_find_multiple_works() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        let record1 = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record1.value.unwrap(), _value1());
        assert_eq!(_sort(record1.tags.unwrap()), _sort(_tags()));

        storage.add(&_type2(), &_id2(), &_value2(), &_tags()).unwrap();
        let record2 = storage.get(&_type2(), &_id2(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record2.value.unwrap(), _value2());
        assert_eq!(_sort(record2.tags.unwrap()), _sort(_tags()));
    }

    #[test]
    fn postgres_storage_get_all_workss() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        storage.add(&_type2(), &_id2(), &_value2(), &_tags()).unwrap();

        let mut storage_iterator = storage.get_all().unwrap();

        let record = storage_iterator.next().unwrap().unwrap();
        assert_eq!(record.type_.unwrap(), _type1());
        assert_eq!(record.value.unwrap(), _value1());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_tags()));

        let record = storage_iterator.next().unwrap().unwrap();
        assert_eq!(record.type_.unwrap(), _type2());
        assert_eq!(record.value.unwrap(), _value2());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_tags()));

        let record = storage_iterator.next().unwrap();
        assert!(record.is_none());
    }

    #[test]
    fn postgres_storage_get_all_works_for_empty() {
        _cleanup();

        let storage = _storage();
        let mut storage_iterator = storage.get_all().unwrap();

        let record = storage_iterator.next().unwrap();
        assert!(record.is_none());
    }

    #[test]
    fn postgres_storage_update_works() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        storage.update(&_type1(), &_id1(), &_value2()).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value2());
    }

    #[test]
    fn postgres_storage_update_works_for_non_existing_id() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        let res = storage.update(&_type1(), &_id2(), &_value2());
        assert_match!(Err(WalletStorageError::ItemNotFound), res)
    }

    #[test]
    fn postgres_storage_update_works_for_non_existing_type() {
        _cleanup();

        let storage = _storage();

        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();
        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        let res = storage.update(&_type2(), &_id1(), &_value2());
        assert_match!(Err(WalletStorageError::ItemNotFound), res)
    }

    #[test]
    fn postgres_storage_add_tags_works() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        storage.add_tags(&_type1(), &_id1(), &_new_tags()).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        let expected_tags = {
            let mut tags = _tags();
            tags.extend(_new_tags());
            _sort(tags)
        };

        assert_eq!(_sort(record.tags.unwrap()), expected_tags);
    }

    #[test]
    fn postgres_storage_add_tags_works_for_non_existing_id() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.add_tags(&_type1(), &_id2(), &_new_tags());
        assert_match!(Err(WalletStorageError::ItemNotFound), res)
    }

    #[test]
    fn postgres_storage_add_tags_works_for_non_existing_type() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.add_tags(&_type2(), &_id1(), &_new_tags());
        assert_match!(Err(WalletStorageError::ItemNotFound), res)
    }

    #[test]
    fn postgres_storage_add_tags_works_for_already_existing() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let tags_with_existing = {
            let mut tags = _tags();
            tags.extend(_new_tags());
            tags
        };

        storage.add_tags(&_type1(), &_id1(), &tags_with_existing).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        let expected_tags = {
            let mut tags = _tags();
            tags.extend(_new_tags());
            _sort(tags)
        };

        assert_eq!(_sort(record.tags.unwrap()), expected_tags);
    }

    #[test]
    fn postgres_storage_update_tags_works() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        storage.update_tags(&_type1(), &_id1(), &_new_tags()).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());
        assert_eq!(_sort(record.tags.unwrap()), _sort(_new_tags()));
    }

    #[test]
    fn postgres_storage_update_tags_works_for_non_existing_id() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.update_tags(&_type1(), &_id2(), &_new_tags());
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    #[test]
    fn postgres_storage_update_tags_works_for_non_existing_type() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let res = storage.update_tags(&_type1(), &_id2(), &_new_tags());
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    #[test]
    fn postgres_storage_update_tags_works_for_already_existing() {
        _cleanup();

        let storage = _storage();
        storage.add(&_type1(), &_id1(), &_value1(), &_tags()).unwrap();

        let tags_with_existing = {
            let mut tags = _tags();
            tags.extend(_new_tags());
            tags
        };

        storage.update_tags(&_type1(), &_id1(), &tags_with_existing).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.value.unwrap(), _value1());

        let expected_tags = {
            let mut tags = _tags();
            tags.extend(_new_tags());
            _sort(tags)
        };

        assert_eq!(_sort(record.tags.unwrap()), expected_tags);
    }

    #[test]
    fn postgres_storage_delete_tags_works() {
        _cleanup();

        let storage = _storage();

        let tag_name1 = vec![0, 0, 0];
        let tag_name2 = vec![1, 1, 1];
        let tag_name3 = vec![2, 2, 2];
        let tag1 = Tag::Encrypted(tag_name1.clone(), vec![0, 0, 0]);
        let tag2 = Tag::PlainText(tag_name2.clone(), "tag_value_2".to_string());
        let tag3 = Tag::Encrypted(tag_name3.clone(), vec![2, 2, 2]);
        let tags = vec![tag1.clone(), tag2.clone(), tag3.clone()];

        storage.add(&_type1(), &_id1(), &_value1(), &tags).unwrap();

        let tag_names = vec![TagName::OfEncrypted(tag_name1.clone()), TagName::OfPlain(tag_name2.clone())];
        storage.delete_tags(&_type1(), &_id1(), &tag_names).unwrap();

        let record = storage.get(&_type1(), &_id1(), r##"{"retrieveType": false, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        assert_eq!(record.tags.unwrap(), vec![tag3]);
    }

    #[test]
    fn postgres_storage_delete_tags_works_for_non_existing_type() {
        _cleanup();

        let storage = _storage();

        let tag_name1 = vec![0, 0, 0];
        let tag_name2 = vec![1, 1, 1];
        let tag_name3 = vec![2, 2, 2];
        let tag1 = Tag::Encrypted(tag_name1.clone(), vec![0, 0, 0]);
        let tag2 = Tag::PlainText(tag_name2.clone(), "tag_value_2".to_string());
        let tag3 = Tag::Encrypted(tag_name3.clone(), vec![2, 2, 2]);
        let tags = vec![tag1.clone(), tag2.clone(), tag3.clone()];

        storage.add(&_type1(), &_id1(), &_value1(), &tags).unwrap();

        let tag_names = vec![TagName::OfEncrypted(tag_name1.clone()), TagName::OfPlain(tag_name2.clone())];
        let res = storage.delete_tags(&_type2(), &_id1(), &tag_names);
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    #[test]
    fn postgres_storage_delete_tags_works_for_non_existing_id() {
        _cleanup();

        let storage = _storage();

        let tag_name1 = vec![0, 0, 0];
        let tag_name2 = vec![1, 1, 1];
        let tag_name3 = vec![2, 2, 2];
        let tag1 = Tag::Encrypted(tag_name1.clone(), vec![0, 0, 0]);
        let tag2 = Tag::PlainText(tag_name2.clone(), "tag_value_2".to_string());
        let tag3 = Tag::Encrypted(tag_name3.clone(), vec![2, 2, 2]);
        let tags = vec![tag1.clone(), tag2.clone(), tag3.clone()];

        storage.add(&_type1(), &_id1(), &_value1(), &tags).unwrap();

        let tag_names = vec![TagName::OfEncrypted(tag_name1.clone()), TagName::OfPlain(tag_name2.clone())];
        let res = storage.delete_tags(&_type1(), &_id2(), &tag_names);
        assert_match!(Err(WalletStorageError::ItemNotFound), res);
    }

    fn _cleanup() {
        let storage_type = PostgresStorageType::new();
        let _res = storage_type.init_storage(Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..])).unwrap();
        let _ret = storage_type.delete_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]));
        let res = test::cleanup_storage();
        res
    }

    fn _wallet_id() -> &'static str {
        "walle1"
    }

    fn _storage() -> Box<dyn WalletStorage> {
        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();
        let res = storage_type.open_storage(_wallet_id(), Some(&_wallet_config()[..]), Some(&_wallet_credentials()[..])).unwrap();
        res
    }

    fn _storage_db_pool() -> Box<dyn WalletStorage> {
        let storage_type = PostgresStorageType::new();
        storage_type.create_storage(_wallet_id(), Some(&_wallet_config_db_pool()[..]), Some(&_wallet_credentials()[..]), &_metadata()).unwrap();
        let res = storage_type.open_storage(_wallet_id(), Some(&_wallet_config_db_pool()[..]), Some(&_wallet_credentials()[..])).unwrap();
        res
    }

    fn _wallet_config() -> String {
        let wallet_scheme = env::var("WALLET_SCHEME");
        match wallet_scheme {
            Ok(scheme) => {
                if scheme == "MultiWalletSingleTable" {
                    return _wallet_config_multi();
                }
                if scheme == "MultiWalletSingleTableSharedPool" {
                    return _wallet_config_multi_with_shared_pool();
                }
            }
            Err(_) => ()
        };
        let config = json!({
            "url": "localhost:5432".to_owned()
        }).to_string();
        config
    }

    fn _wallet_config_multi() -> String {
        let config = json!({
            "url": "localhost:5432".to_owned(),
            "wallet_scheme": "MultiWalletSingleTable".to_owned()
        }).to_string();
        config
    }

    fn _wallet_config_multi_with_shared_pool() -> String {
        let config = json!({
            "url": "localhost:5432".to_owned(),
            "wallet_scheme": "MultiWalletSingleTableSharedPool".to_owned()
        }).to_string();
        config
    }

    fn _wallet_config_db_pool() -> String {
        let config = json!({
            "url": "localhost:5432".to_owned(),
            "tls": "None",
            "max_connections": 4,
            "min_idle_count": 0,
            "connection_timeout": 10
        }).to_string();
        config
    }

    fn _wallet_credentials() -> String {
        let creds = json!({
            "account": "postgres".to_owned(),
            "password": "mysecretpassword".to_owned(),
            "admin_account": Some("postgres".to_owned()),
            "admin_password": Some("mysecretpassword".to_owned())
        }).to_string();
        creds
    }

    fn _metadata() -> Vec<u8> {
        return vec![
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8
        ];
    }

    fn _type(i: u8) -> Vec<u8> {
        vec![i, 1 + i, 2 + i]
    }

    fn _type1() -> Vec<u8> {
        _type(1)
    }

    fn _type2() -> Vec<u8> {
        _type(2)
    }

    fn _id(i: u8) -> Vec<u8> {
        vec![3 + i, 4 + i, 5 + i]
    }

    fn _id1() -> Vec<u8> {
        _id(1)
    }

    fn _id2() -> Vec<u8> {
        _id(2)
    }

    fn _value(i: u8) -> EncryptedValue {
        EncryptedValue { data: vec![6 + i, 7 + i, 8 + i], key: vec![9 + i, 10 + i, 11 + i] }
    }

    fn _value1() -> EncryptedValue {
        _value(1)
    }

    fn _value2() -> EncryptedValue {
        _value(2)
    }

    fn _tags() -> Vec<Tag> {
        let mut tags: Vec<Tag> = Vec::new();
        tags.push(Tag::Encrypted(vec![1, 5, 8], vec![3, 5, 6]));
        tags.push(Tag::PlainText(vec![1, 5, 8, 1], "Plain value 1".to_string()));
        tags.push(Tag::Encrypted(vec![2, 5, 8], vec![3, 5, 7]));
        tags.push(Tag::PlainText(vec![2, 5, 8, 1], "Plain value 2".to_string()));
        tags
    }

    fn _new_tags() -> Vec<Tag> {
        vec![
            Tag::Encrypted(vec![1, 1, 1], vec![2, 2, 2]),
            Tag::PlainText(vec![1, 1, 1], String::from("tag_value_3"))
        ]
    }

    fn _sort(mut v: Vec<Tag>) -> Vec<Tag> {
        v.sort();
        v
    }
}
