[![CircleCI](https://circleci.com/gh/mmmpa/poor_locker.svg?style=shield)](https://circleci.com/gh/mmmpa/poor_locker)

# poor_locker

We use to avoid processing same event from Slack.

Slack sends same event multiple times when we don't or not immediately return 200.

Our responses often delay enough for Slack to re-send events due to Cold Starting.

```rs
use crate::{DynamoLockStoreClient, DynamoLockerClient, LockKey, Locker};
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;

#[tokio::test]
async fn test_lock() {
    // need DynamoDB.
    // schema sample - terraform/.mods/dynamo_db.tf
    let store = DynamoLockStoreClient::from((
        DynamoDbClient::new(Region::UsEast1),
        "poor-locker-test-lock-table".to_string(),
    ));
    let cli = DynamoLockerClient::from(store);

    // generate unique key from coming event in App
    let lock_key = LockKey::from(rs_ttb::random_string(12));

    let re = cli.allow_only_first_comer(lock_key.clone()).await;
    assert!(re.is_ok(), "{:?}", re);

    let re = cli.allow_only_first_comer(lock_key.clone()).await;
    assert!(re.is_err(), "{:?}", re);
}
```