mod dynamo_client;

pub use dynamo_client::*;

#[cfg(test)]
mod tests {
    use crate::{DynamoLockStoreClient, DynamoLockerClient, LockKey, Locker, LockerError};
    use rs_ttb;
    use rusoto_core::{HttpClient, Region};
    use rusoto_credential::StaticProvider;
    use rusoto_dynamodb::DynamoDbClient;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::Duration;

    pub fn gen_dynamo_client() -> DynamoDbClient {
        let region = Region::Custom {
            name: "us-west-2".to_owned(),
            endpoint: "http://localhost:8000".to_owned(),
        };
        let credential = StaticProvider::new(
            "poor_locker".to_string(),
            "poor_locker".to_string(),
            None,
            None,
        );
        DynamoDbClient::new_with(HttpClient::new().unwrap(), credential, region)
    }

    fn create_cli() -> DynamoLockerClient {
        let store = DynamoLockStoreClient::from((
            gen_dynamo_client(),
            "poor-locker-test-lock-table".to_string(),
        ));
        (store, 10).into()
    }

    #[tokio::test]
    async fn test_lock() {
        let cli = create_cli();
        let lock_key = LockKey::from(rs_ttb::random_string(12));

        let re = cli.lock(lock_key.clone()).await;
        assert!(re.is_ok(), "{:?}", re);

        let re = cli.lock(lock_key.clone()).await;
        assert!(re.is_err(), "{:?}", re);

        let re = cli.unlock(lock_key.clone()).await;
        assert!(re.is_ok(), "{:?}", re);

        let re = cli.lock(lock_key.clone()).await;
        assert!(re.is_ok(), "{:?}", re);

        let re = cli.unlock(lock_key.clone()).await;
        assert!(re.is_ok(), "{:?}", re);

        let re = cli.unlock(lock_key.clone()).await;
        assert!(re.is_err(), "{:?}", re);
    }

    #[tokio::test]
    async fn test_wait() {
        let cli = create_cli();
        let a = Arc::new(RwLock::new(Vec::<usize>::new()));

        let lock_key = LockKey::from(rs_ttb::random_string(12));
        cli.lock(lock_key.clone()).await.unwrap();

        let thread = {
            let cli = cli.clone();
            let key = lock_key.clone();
            let a = a.clone();
            tokio::spawn(async move {
                cli.wait(key, 100).await.unwrap();
                a.write().await.push(24);
            })
        };

        a.write().await.push(11);
        cli.unlock(lock_key).await.unwrap();

        futures::future::join_all(vec![thread]).await;

        assert_eq!(*a.read().await, vec![11, 24]);
    }

    #[tokio::test]
    async fn test_wait_abort() {
        let cli = create_cli();
        let a = Arc::new(RwLock::new(Vec::<usize>::new()));

        let lock_key = LockKey::from(rs_ttb::random_string(12));
        cli.lock(lock_key.clone()).await.unwrap();

        let thread = {
            let cli = cli.clone();
            let key = lock_key.clone();
            let a = a.clone();
            tokio::spawn(async move {
                cli.wait(key, 1).await?;
                a.write().await.push(24);

                Ok::<(), LockerError>(())
            })
        };

        futures::future::join_all(vec![thread]).await;

        assert_eq!(*a.read().await, Vec::<usize>::new());
    }

    #[tokio::test]
    async fn test_wait_work() {
        let cli = create_cli();
        let a = Arc::new(RwLock::new(Vec::<usize>::new()));

        let lock_key = LockKey::from(rs_ttb::random_string(12));

        let thread_a = {
            let cli = cli.clone();
            let key = lock_key.clone();
            let a = a.clone();
            tokio::spawn(async move {
                cli.work_with_wait_lock(key, 10, || async {
                    tokio::time::delay_for(Duration::from_millis(20)).await;
                    a.write().await.push(11);
                })
                .await
                .unwrap();
            })
        };

        let thread_b = {
            let cli = cli.clone();
            let key = lock_key.clone();
            let a = a.clone();
            tokio::spawn(async move {
                tokio::time::delay_for(Duration::from_millis(10)).await;
                cli.work_with_wait_lock(key, 100, || async {
                    a.write().await.push(24);
                })
                .await
                .unwrap();
            })
        };

        futures::future::join_all(vec![thread_a, thread_b]).await;

        assert_eq!(*a.read().await, vec![11, 24]);
    }
}
