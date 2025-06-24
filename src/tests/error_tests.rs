#[cfg(test)]
mod tests {
    use crate::errors::VestaboardError;
    use std::error::Error;
    use std::io::{ Error as IoError, ErrorKind };

    #[test]
    fn test_io_error_constructor() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let context = "reading config file";
        let vb_error = VestaboardError::io_error(io_err, context);

        match vb_error {
            VestaboardError::IOError { context: ctx, .. } => {
                assert_eq!(ctx, "reading config file");
            }
            _ => panic!("Expected IOError variant"),
        }
    }

    #[test]
    fn test_json_error_constructor() {
        let json_str = r#"{"invalid": json syntax}"#;
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let context = "parsing schedule file";
        let vb_error = VestaboardError::json_error(json_err, context);

        match vb_error {
            VestaboardError::JsonError { context: ctx, .. } => {
                assert_eq!(ctx, "parsing schedule file");
            }
            _ => panic!("Expected JsonError variant"),
        }
    }

    #[test]
    fn test_widget_error_constructor() {
        let widget = "weather";
        let message = "API key not found";
        let vb_error = VestaboardError::widget_error(widget, message);

        match vb_error {
            VestaboardError::WidgetError { widget: w, message: m } => {
                assert_eq!(w, "weather");
                assert_eq!(m, "API key not found");
            }
            _ => panic!("Expected WidgetError variant"),
        }
    }

    #[test]
    fn test_schedule_error_constructor() {
        let operation = "save_schedule";
        let message = "disk full";
        let vb_error = VestaboardError::schedule_error(operation, message);

        match vb_error {
            VestaboardError::ScheduleError { operation: op, message: msg } => {
                assert_eq!(op, "save_schedule");
                assert_eq!(msg, "disk full");
            }
            _ => panic!("Expected ScheduleError variant"),
        }
    }

    #[test]
    fn test_api_error_constructor() {
        let code = Some(404);
        let message = "Not found";
        let vb_error = VestaboardError::api_error(code, message);

        match vb_error {
            VestaboardError::ApiError { code: c, message: msg } => {
                assert_eq!(c, Some(404));
                assert_eq!(msg, "Not found");
            }
            _ => panic!("Expected ApiError variant"),
        }
    }

    #[test]
    fn test_config_error_constructor() {
        let field = "api_key";
        let message = "missing required field";
        let vb_error = VestaboardError::config_error(field, message);

        match vb_error {
            VestaboardError::ConfigError { field: f, message: msg } => {
                assert_eq!(f, "api_key");
                assert_eq!(msg, "missing required field");
            }
            _ => panic!("Expected ConfigError variant"),
        }
    }

    #[test]
    fn test_other_error_constructor() {
        let message = "unexpected error";
        let vb_error = VestaboardError::other(message);

        match vb_error {
            VestaboardError::Other { message: msg } => {
                assert_eq!(msg, "unexpected error");
            }
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_display_formatting() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "permission denied");
        let vb_error = VestaboardError::io_error(io_err, "writing to file");
        let display_str = format!("{}", vb_error);
        assert!(display_str.contains("IO Error in writing to file"));
        assert!(display_str.contains("permission denied"));

        let widget_error = VestaboardError::widget_error("weather", "API timeout");
        let display_str = format!("{}", widget_error);
        assert_eq!(display_str, "Widget Error [weather]: API timeout");

        let api_error = VestaboardError::api_error(Some(500), "Internal server error");
        let display_str = format!("{}", api_error);
        assert_eq!(display_str, "API Error [500]: Internal server error");

        let api_error_no_code = VestaboardError::api_error(None, "Unknown error");
        let display_str = format!("{}", api_error_no_code);
        assert_eq!(display_str, "API Error: Unknown error");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let vb_error: VestaboardError = io_err.into();

        match vb_error {
            VestaboardError::IOError { context, .. } => {
                assert_eq!(context, "unknown context");
            }
            _ => panic!("Expected IOError variant"),
        }
    }

    #[test]
    fn test_from_json_error() {
        let json_str = r#"{"invalid": json}"#;
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let vb_error: VestaboardError = json_err.into();

        match vb_error {
            VestaboardError::JsonError { context, .. } => {
                assert_eq!(context, "unknown context");
            }
            _ => panic!("Expected JsonError variant"),
        }
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let vb_error = VestaboardError::io_error(io_err, "reading config");

        // Test that source() returns the wrapped error
        assert!(vb_error.source().is_some());
        assert_eq!(vb_error.source().unwrap().to_string(), "file not found");

        // Test that widget errors don't have a source
        let widget_error = VestaboardError::widget_error("test", "message");
        assert!(widget_error.source().is_none());
    }

    #[test]
    fn test_partial_eq() {
        // Test IO errors
        let io_err1 = IoError::new(ErrorKind::NotFound, "file1");
        let io_err2 = IoError::new(ErrorKind::NotFound, "file2");
        let vb_error1 = VestaboardError::io_error(io_err1, "context1");
        let vb_error2 = VestaboardError::io_error(io_err2, "context1");
        let vb_error3 = VestaboardError::io_error(
            IoError::new(ErrorKind::NotFound, "file1"),
            "context2"
        );

        assert_eq!(vb_error1, vb_error2); // Same context
        assert_ne!(vb_error1, vb_error3); // Different context

        // Test Widget errors
        let widget_err1 = VestaboardError::widget_error("weather", "timeout");
        let widget_err2 = VestaboardError::widget_error("weather", "timeout");
        let widget_err3 = VestaboardError::widget_error("weather", "different");
        let widget_err4 = VestaboardError::widget_error("text", "timeout");

        assert_eq!(widget_err1, widget_err2);
        assert_ne!(widget_err1, widget_err3); // Different message
        assert_ne!(widget_err1, widget_err4); // Different widget

        // Test cross-variant inequality
        let schedule_err = VestaboardError::schedule_error("save", "error");
        assert_ne!(widget_err1, schedule_err);
    }

    #[test]
    fn test_error_context_preservation() {
        let original_msg = "original error message";
        let context = "specific operation context";

        let io_err = IoError::new(ErrorKind::PermissionDenied, original_msg);
        let vb_error = VestaboardError::io_error(io_err, context);

        // Check that both original error and context are preserved
        let display_str = format!("{}", vb_error);
        assert!(display_str.contains(context));
        assert!(display_str.contains(original_msg));

        // Check that source chain works
        let source = vb_error.source().unwrap();
        assert_eq!(source.to_string(), original_msg);
    }

    #[test]
    fn test_debug_formatting() {
        let widget_error = VestaboardError::widget_error("test", "message");
        let debug_str = format!("{:?}", widget_error);
        assert!(debug_str.contains("WidgetError"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("message"));
    }
}
