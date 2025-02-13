use std::{
    path::Path,
    sync::{Arc, PoisonError, RwLock},
};

use redb::{
    Database, Key, ReadTransaction, ReadableTable, TableDefinition, TableError, Value,
    WriteTransaction,
};

const STARTUP_APPS_TABLE: TableDefinition<String, Vec<u16>> = TableDefinition::new("startup_apps");

const BRIDGE_TABLE: TableDefinition<String, Vec<u8>> = TableDefinition::new("bridge");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Redb(redb::Error),
}

impl<T: Into<redb::Error>> From<T> for Error {
    fn from(err: T) -> Self {
        Error::Redb(err.into())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct Client {
    db: Arc<RwLock<Database>>,
}

impl Client {
    pub fn new(file: impl AsRef<Path>) -> Result<Self> {
        let file = file.as_ref();

        let db = if file.exists() {
            Database::open(file)?
        } else {
            Database::create(file)?
        };

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
        })
    }

    pub fn get_startup_apps(&self) -> Result<Vec<(String, Vec<u16>)>> {
        self.get_all_cloned(STARTUP_APPS_TABLE)
    }

    pub fn insert_startup_app(
        &self,
        image: impl Into<String>,
        ports: Vec<u16>,
    ) -> Result<Option<Vec<u16>>> {
        let write = self.write()?;

        let old_ports = write
            .open_table(STARTUP_APPS_TABLE)?
            .insert(image.into(), ports)?
            .map(|v| v.value().clone());
        write.commit()?;

        Ok(old_ports)
    }

    pub fn remove_startup_app(&self, image: impl Into<String>) -> Result<Option<Vec<u16>>> {
        let write = self.write()?;
        let ports = write
            .open_table(STARTUP_APPS_TABLE)?
            .remove(image.into())?
            .map(|v| v.value().clone());
        write.commit()?;
        Ok(ports)
    }

    fn get_all_cloned<K, V>(&self, table: TableDefinition<K, V>) -> Result<Vec<(K, V)>>
    where
        K: Key + Clone + 'static,
        V: Clone + 'static,
        for<'a> K: Value<SelfType<'a> = K>,
        for<'a> V: Value<SelfType<'a> = V>,
    {
        let read = self.read()?;
        match read.open_table(table) {
            Ok(table) => table
                .range::<K>(..)?
                .map(|res| {
                    res.map(|(k, v)| (k.value().clone(), v.value().clone()))
                        .map_err(Into::into)
                })
                .collect(),
            Err(TableError::TableDoesNotExist(_)) => {
                let write = self.write()?;
                let res = write
                    .open_table(table)?
                    .range::<K>(..)?
                    .map(|res| {
                        res.map(|(k, v)| (k.value().clone(), v.value().clone()))
                            .map_err(Into::into)
                    })
                    .collect();
                write.commit()?;
                res
            }
            Err(e) => Err(e.into()),
        }
    }

    fn write(&self) -> Result<WriteTransaction> {
        self.db
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .begin_write()
            .map_err(Into::into)
    }

    fn read(&self) -> Result<ReadTransaction> {
        self.db
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .begin_read()
            .map_err(Into::into)
    }
}

impl hyveos_bridge::DbClient for Client {
    type Error = Error;

    async fn put(&self, key: String, value: impl Into<Vec<u8>> + Send) -> Result<Option<Vec<u8>>> {
        let write = self.write()?;

        let old_value = write
            .open_table(BRIDGE_TABLE)?
            .insert(key, value.into())?
            .map(|v| v.value().clone());
        write.commit()?;

        Ok(old_value)
    }

    async fn get(&self, key: String) -> Result<Option<Vec<u8>>> {
        let read = self.read()?;
        match read.open_table(BRIDGE_TABLE) {
            Ok(table) => {
                let value = table.get(key)?.map(|v| v.value().clone());
                Ok(value)
            }
            Err(TableError::TableDoesNotExist(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
