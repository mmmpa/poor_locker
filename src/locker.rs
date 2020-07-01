use crate::{LockKey, LockerError, LockerResult};
use async_trait::async_trait;
use futures::Future;
use std::cmp::min;
use tokio::macros::support::Pin;
use tokio::time::Duration;

#[async_trait]
pub trait Locker: Clone + Send + Sync + 'static {
    type LockerStore: LockStore;

    fn store(&self) -> &Self::LockerStore;
    fn delay(&self) -> u64;

    /// Just an alias to lock.
    async fn allow_only_first_comer(&self, key: LockKey) -> LockerResult<()> {
        self.lock(key).await
    }

    async fn lock(&self, key: LockKey) -> LockerResult<()> {
        self.store().lock(key).await
    }

    async fn unlock(&self, key: LockKey) -> LockerResult<()> {
        self.store().unlock(key).await
    }

    async fn wait(&self, key: LockKey, mut ms: u64) -> LockerResult<()> {
        let mut rest = ms;
        let mut delay = self.delay();

        loop {
            if let Ok(_) = self.lock(key.clone()).await {
                return Ok(());
            }

            if rest == 0 {
                return Err(LockerError::Timeout);
            }

            delay = min(rest, delay);

            tokio::time::delay_for(Duration::from_millis(delay)).await;

            if rest <= delay {
                rest = 0;
            } else {
                rest -= delay;
            }
        }

        unreachable!()
    }

    async fn work_with_wait_lock<F, R, Fut>(&self, key: LockKey, secs: u64, f: F) -> LockerResult<R>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = R> + Send,
        R: Send,
    {
        self.wait(key.clone(), secs).await?;
        let re = f().await;
        self.unlock(key).await?;
        Ok(re)
    }
}

#[async_trait]
pub trait LockStore: Clone + Send + Sync + 'static {
    async fn lock(&self, key: LockKey) -> LockerResult<()>;
    async fn unlock(&self, key: LockKey) -> LockerResult<()>;
}
