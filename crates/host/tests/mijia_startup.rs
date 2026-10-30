use homeverb_core::Operation;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn explicit_mijia_core_startup_is_empty_local_and_read_only() {
    let directory =
        std::env::temp_dir().join(format!("homeverb-mijia-startup-{}", uuid::Uuid::new_v4()));
    let core = homeverb_host::open_mijia_read_only(&directory).unwrap();
    assert!(core.list().unwrap().is_empty());
    assert!(
        core.discover(&Operation::new(1000, CancellationToken::new()).unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    drop(core);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn explicit_mijia_event_startup_remains_empty_without_account_or_discovery() {
    let directory =
        std::env::temp_dir().join(format!("homeverb-event-startup-{}", uuid::Uuid::new_v4()));
    let core = homeverb_host::open_mijia_read_only_with_events(&directory).unwrap();
    assert!(core.list().unwrap().is_empty());
    assert!(core.recent_executions().unwrap().is_empty());
    assert_eq!(
        core.inspect_event_history_options(
            "fictional",
            "cloud",
            &Operation::new(1000, CancellationToken::new()).unwrap()
        )
        .await
        .unwrap_err()
        .code,
        "DEVICE_NOT_FOUND"
    );
    drop(core);
    std::fs::remove_dir_all(directory).unwrap();
}
