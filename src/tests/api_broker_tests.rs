#[path = "../api_broker.rs"]
mod api_broker;
#[path = "../api.rs"]
mod api;

#[cfg(test)]
mod tests {
    use std::sync::{ Arc, Mutex };

    // use super::*;
    use crate::api::Api;
    use crate::api_broker::{ ApiBroker, LocalApiBroker };

    struct MockApi {
        sent_message: Arc<Mutex<Option<[[u8; 22]; 6]>>>,
    }

    impl MockApi {
        fn new() -> Self {
            MockApi { sent_message: Arc::new(Mutex::new(None)) }
        }
    }

    impl Api for MockApi {
        async fn send_message(&self, message: [[u8; 22]; 6]) -> Result<(), reqwest::Error> {
            *self.sent_message.lock().unwrap() = Some(message);
            Ok(())
        }
        async fn get_message(&self) -> Result<(), reqwest::Error> {
            Ok(())
        }
    }
    #[test]
    fn test_valid_message() {
        let api_broker = LocalApiBroker::new();
        let message = "hello";
        let expected_codes = Some(vec![8, 5, 12, 12, 15]);
        assert_eq!(api_broker.to_codes(message), expected_codes);
    }

    #[test]
    fn test_invalid_message() {
        let api_broker = LocalApiBroker::new();
        let message = "Hello!";
        assert_eq!(api_broker.to_codes(message), None);
    }

    #[test]
    fn test_empty_message() {
        let api_broker = LocalApiBroker::new();
        let message = "";
        let expected_codes: Option<Vec<u8>> = Some(vec![]);
        assert_eq!(api_broker.to_codes(message), expected_codes);
    }

    #[test]
    fn test_message_with_spaces() {
        let api_broker = LocalApiBroker::new();
        let message = "hello world";
        let expected_codes = Some(vec![8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4]);
        assert_eq!(api_broker.to_codes(message), expected_codes);
    }

    #[test]
    fn test_message_with_numbers() {
        let api_broker = LocalApiBroker::new();
        let message = "1234567890";
        let expected_codes = Some(vec![27, 28, 29, 30, 31, 32, 33, 34, 35, 36]);
        assert_eq!(api_broker.to_codes(message), expected_codes);
    }

    #[test]
    fn test_message_with_colors() {
        let api_broker = LocalApiBroker::new();
        let message = "ROYGBVWK";
        let expected_codes = Some(vec![63, 64, 65, 66, 67, 68, 69, 70]);
        assert_eq!(api_broker.to_codes(message), expected_codes);
    }

    #[tokio::test]
    async fn test_display_message() {
        let mock_api = MockApi::new();
        let sent_message = Arc::clone(&mock_api.sent_message);
        let api_broker = LocalApiBroker::new_with_api(mock_api);
        let message = vec!["hello".to_string(), "world".to_string()];
        api_broker.display_message(message, false).await;
        let expected = [
            [8, 5, 12, 12, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0; 22],
            [0; 22],
            [0; 22],
            [0; 22],
        ];
        assert_eq!(sent_message.lock().unwrap().unwrap(), expected);
    }

    #[tokio::test]
    async fn test_display_message_all_characters() {
        let mock_api = MockApi::new();
        let sent_message = Arc::clone(&mock_api.sent_message);
        let api_broker = LocalApiBroker::new_with_api(mock_api);
        let message = vec![
            "ROYGBVKW".to_string(),
            "abcdefghijklmnopqrstuv".to_string(),
            "wxyz1234567890".to_string(),
            "!@#$()-+&=;:'\"%,./?D".to_string()
        ];
        api_broker.display_message(message, false).await;
        let expected = [
            [63, 64, 65, 66, 67, 68, 70, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
            [23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 0, 0, 0, 0, 0, 0, 0, 0],
            [37, 38, 39, 40, 41, 42, 44, 46, 47, 48, 49, 50, 52, 53, 54, 55, 56, 59, 60, 62, 0, 0],
            [0; 22],
            [0; 22],
        ];
        assert_eq!(sent_message.lock().unwrap().unwrap(), expected);
    }
}
