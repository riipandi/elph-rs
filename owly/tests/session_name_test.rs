use owly::checkpoint::TursoCheckpointSaver;
use tempfile::TempDir;

#[tokio::test]
async fn thread_metadata_roundtrip() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("meta.sqlite");
    let saver = TursoCheckpointSaver::open(Some(path)).await.expect("open");

    let thread_id = "owly-test-thread";
    assert!(
        saver
            .get_thread_metadata(thread_id)
            .await
            .expect("read")
            .display_name
            .is_none()
    );

    saver
        .set_thread_display_name(thread_id, "Fix auth middleware", true)
        .await
        .expect("write");
    let meta = saver.get_thread_metadata(thread_id).await.expect("read");
    assert_eq!(meta.display_name.as_deref(), Some("Fix auth middleware"));
    assert!(meta.auto_named);

    saver.delete_thread(thread_id).await.expect("delete");
    let meta = saver.get_thread_metadata(thread_id).await.expect("read after delete");
    assert!(meta.display_name.is_none());
}
