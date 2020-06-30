use crate::{DynamoLockStoreClient, LockKey, LockStore, Locker, LockerError, LockerResult};
use async_trait::async_trait;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeValue, DeleteItemError, DeleteItemInput, DeleteItemOutput, DynamoDb, DynamoDbClient,
    PutItemError, PutItemInput, PutItemOutput,
};
use serde::export::Formatter;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamoLockerClient(Arc<DynamoLockerClientInner>);

#[derive(Clone, Debug)]
struct DynamoLockerClientInner {
    store: DynamoLockStoreClient,
    delay: u64,
}

impl From<DynamoLockStoreClient> for DynamoLockerClient {
    fn from(store: DynamoLockStoreClient) -> Self {
        Self(Arc::new(DynamoLockerClientInner { store, delay: 500 }))
    }
}

impl From<(DynamoLockStoreClient, u64)> for DynamoLockerClient {
    fn from((store, delay): (DynamoLockStoreClient, u64)) -> Self {
        Self(Arc::new(DynamoLockerClientInner { store, delay }))
    }
}

impl Locker for DynamoLockerClient {
    type LockerStore = DynamoLockStoreClient;

    fn store(&self) -> &Self::LockerStore {
        &self.0.store
    }

    fn delay(&self) -> u64 {
        self.0.delay
    }
}
