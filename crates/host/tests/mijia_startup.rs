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
