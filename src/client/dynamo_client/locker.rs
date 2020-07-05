use crate::{DynamoLockStoreClient, Locker};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamoLockerClient(Arc<DynamoLockerClientInner>);

#[derive(Clone, Debug)]
struct DynamoLockerClientInner {
    store: DynamoLockStoreClient,
    delay: u64,
}

impl DynamoLockerClient {
    pub fn new(store: DynamoLockStoreClient, delay: u64) -> Self {
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
