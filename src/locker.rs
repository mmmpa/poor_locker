use crate::{LockKey, LockerError, LockerResult};
use async_trait::async_trait;
use futures::Future;
use tokio::macros::support::Pin;
use tokio::time::Duration;

#[async_trait]
pub trait Locker: Clone + Send + Sync + 'static {
    type LockerStore: LockStore;

    fn store(&self) -> &Self::LockerStore;
    fn delay(&self) -> u64;

    async fn lock(&self, key: LockKey) -> LockerResult<()> {
        self.store().lock(key).await
    }

    async fn unlock(&self, key: LockKey) -> LockerResult<()> {
        self.store().unlock(key).await
    }

    async fn wait(&self, key: LockKey, mut ms: u64) -> LockerResult<()> {
        let mut rest = ms;

        loop {
            if let Ok(_) = self.lock(key.clone()).await {
                return Ok(());
            }

            tokio::time::delay_for(Duration::from_millis(self.delay())).await;

            if rest < self.delay() {
                return Err(LockerError::Timeout);
            }

            rest -= self.delay();
        }
    }

    async fn wait_and_work<F, R, Fut>(&self, key: LockKey, secs: u64, f: F) -> LockerResult<R>
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
