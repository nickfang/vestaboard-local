#[path = "../api.rs"]
mod api;
use api::{ Api, LocalApi, clear_board, blank_board };

// TODO: figure out how to test the api functions
#[cfg(test)]
#[tokio::test]
#[ignore]
async fn test_send_message() {
    let api = LocalApi::new();
    let message = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let result = api.send_message(message);
    assert!(result.await.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_clear_board() {
    let api = LocalApi::new();
    let result = clear_board(&api);
    assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_blank_board() {
    let api = LocalApi::new();
    let result = blank_board(&api);
    assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_get_message() {
    let api = LocalApi::new();
    let result = api.get_message();
    assert!(result.await.is_ok());
}
