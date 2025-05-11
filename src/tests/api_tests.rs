#[path = "../api.rs"]
mod api;
use api::{ send_codes, clear_board, blank_board, get_message };
// TODO: figure out how to test the api functions
#[cfg(test)]
#[tokio::test]
#[ignore]
async fn test_send_codes() {
    let message = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let result = send_codes(message);
    assert!(result.await.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_clear_board() {
    let result = clear_board();
    assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_blank_board() {
    let result = blank_board();
    assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_get_message() {
    let result = get_message();
    assert!(result.await.is_ok());
}
