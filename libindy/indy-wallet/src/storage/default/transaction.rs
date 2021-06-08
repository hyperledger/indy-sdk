use rusqlite::{TransactionBehavior, Connection, DropBehavior, Result};
use std::ops::Deref;

pub struct Transaction<'conn> {
    conn: &'conn Connection,
    drop_behavior: DropBehavior,
    committed: bool,
}

impl<'conn> Transaction<'conn> {
    /// Begin a new transaction. Cannot be nested; see `savepoint` for nested transactions.
    pub fn new(conn: &Connection, behavior: TransactionBehavior) -> Result<Transaction> {
        let query = match behavior {
            TransactionBehavior::Deferred => "BEGIN DEFERRED",
            TransactionBehavior::Immediate => "BEGIN IMMEDIATE",
            TransactionBehavior::Exclusive => "BEGIN EXCLUSIVE",
            _ => ""
        };
        conn.execute_batch(query)
            .map(move |_| {
                Transaction {
                    conn,
                    drop_behavior: DropBehavior::Rollback,
                    committed: false,
                }
            })
    }

    /// Get the current setting for what happens to the transaction when it is dropped.
    pub fn drop_behavior(&self) -> DropBehavior {
        self.drop_behavior
    }

    /// Configure the transaction to perform the specified action when it is dropped.
    #[allow(dead_code)]
    pub fn set_drop_behavior(&mut self, drop_behavior: DropBehavior) {
        self.drop_behavior = drop_behavior
    }

    /// A convenience method which consumes and commits a transaction.
    pub fn commit(mut self) -> Result<()> {
        self.commit_()
    }

    fn commit_(&mut self) -> Result<()> {
        self.committed = true;
        self.conn.execute_batch("COMMIT")
    }

    /// A convenience method which consumes and rolls back a transaction.
    #[allow(dead_code)]
    pub fn rollback(mut self) -> Result<()> {
        self.rollback_()
    }

    fn rollback_(&mut self) -> Result<()> {
        self.committed = true;
        self.conn.execute_batch("ROLLBACK")
    }

    /// Consumes the transaction, committing or rolling back according to the current setting
    /// (see `drop_behavior`).
    ///
    /// Functionally equivalent to the `Drop` implementation, but allows callers to see any
    /// errors that occur.
    #[allow(dead_code)]
    pub fn finish(mut self) -> Result<()> {
        self.finish_()
    }

    fn finish_(&mut self) -> Result<()> {
        if self.committed {
            return Ok(());
        }
        match self.drop_behavior() {
            DropBehavior::Commit => self.commit_(),
            DropBehavior::Rollback => self.rollback_(),
            DropBehavior::Ignore => Ok(()),
            DropBehavior::Panic => {
                panic!("drop behaviour is set to panic".to_string());
            }
            _ => {
                panic!("internal error: unsupported drop behaviour");
            }
        }
    }
}

impl<'conn> Deref for Transaction<'conn> {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        self.conn
    }
}

#[allow(unused_must_use)]
impl<'conn> Drop for Transaction<'conn> {
    fn drop(&mut self) {
        self.finish_();
    }
}
