use crate::{LockKey, LockStore, Locker, LockerError, LockerResult};
use async_trait::async_trait;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeValue, DeleteItemError, DeleteItemInput, DeleteItemOutput, DynamoDb, DynamoDbClient,
    PutItemError, PutItemInput, PutItemOutput,
};
use serde::export::Formatter;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

#[derive(Clone)]
pub struct DynamoLockStoreClient {
    pub cli: DynamoDbClient,
    pub table_name: String,
    pub id_attribute_name: String,
    pub status_attribute_name: String,
}

impl From<(DynamoDbClient, String)> for DynamoLockStoreClient {
    fn from((cli, table_name): (DynamoDbClient, String)) -> Self {
        Self {
            cli,
            table_name,
            id_attribute_name: "hash_key".to_string(),
            status_attribute_name: "locked".to_string(),
        }
    }
}

impl std::fmt::Debug for DynamoLockStoreClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynamoLockStoreClient {{ MASKED }}");
        Ok(())
    }
}

impl DynamoLockStoreClient {
    fn expression(&self) -> Option<String> {
        let ex = format!("attribute_not_exists({})", self.status_attribute_name);
        Some(ex)
    }

    fn table(&self) -> String {
        self.table_name.clone()
    }

    fn id_attr(&self) -> String {
        self.id_attribute_name.clone()
    }

    fn status_attr(&self) -> String {
        self.status_attribute_name.clone()
    }

    fn create_key_item(&self, key: LockKey) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert(
            self.id_attr(),
            AttributeValue {
                s: key.as_str().to_string().into(),
                ..Default::default()
            },
        );
        item
    }

    fn create_lock_item(&self, key: LockKey) -> HashMap<String, AttributeValue> {
        let mut item = self.create_key_item(key);
        item.insert(
            self.status_attr(),
            AttributeValue {
                bool: true.into(),
                ..Default::default()
            },
        );
        item
    }

    fn create_put_item(&self, key: LockKey) -> PutItemInput {
        PutItemInput {
            condition_expression: self.expression(),
            item: self.create_lock_item(key),
            table_name: self.table(),
            ..Default::default()
        }
    }
}

#[async_trait]
impl LockStore for DynamoLockStoreClient {
    async fn lock(&self, key: LockKey) -> LockerResult<()> {
        match self.cli.put_item(self.create_put_item(key.clone())).await {
            Ok(_) => Ok(()),
            Err(RusotoError::Service(PutItemError::ConditionalCheckFailed(_))) => {
                Err(LockerError::AlreadyLocked(key))
            }
            Err(e) => Err(LockerError::AccessError(e.to_string())),
        }
    }

    async fn unlock(&self, key: LockKey) -> LockerResult<()> {
        match self
            .cli
            .delete_item(DeleteItemInput {
                condition_expression: None,
                table_name: self.table(),
                return_values: "ALL_OLD".to_string().into(),
                key: self.create_key_item(key.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(o) => match o.attributes {
                None => Err(LockerError::AlreadyUnlocked(key)),
                Some(_) => Ok(()),
            },
            Err(e) => Err(LockerError::AccessError(e.to_string())),
        }
    }
}
