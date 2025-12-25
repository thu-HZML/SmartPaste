use super::*;
use crate::db::cleanup::{
    notify_cleanup, reset_cleanup_sender, set_cleanup_sender, trigger_cleanup,
};
use std::sync::mpsc;
use std::time::Duration;

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::db::TEST_RUN_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

#[test]
fn test_cleanup_notification_flow() {
    let _g = test_lock();

    // 1. Create a channel
    let (tx, rx) = mpsc::channel();

    // 2. Set the sender
    set_cleanup_sender(tx);

    // 3. Trigger cleanup manually
    let result = trigger_cleanup();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cleanup triggered");

    // 4. Verify message received
    let msg = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg.is_ok());

    // 5. Test notify_cleanup (internal function)
    notify_cleanup();
    let msg2 = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg2.is_ok());
}

#[test]
fn test_trigger_cleanup_without_sender() {
    let _g = test_lock();

    // Reset sender to None
    reset_cleanup_sender();

    // Trigger cleanup should fail
    let result = trigger_cleanup();
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "cleanup worker not started");
}
