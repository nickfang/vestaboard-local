#[derive(Debug)]
pub enum VestaboardError {
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    WidgetError(String),
    ScheduleError(String),
}

impl PartialEq for VestaboardError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VestaboardError::IOError(e1), VestaboardError::IOError(e2)) =>
                e1.to_string() == e2.to_string(),
            (VestaboardError::JsonError(e1), VestaboardError::JsonError(e2)) =>
                e1.to_string() == e2.to_string(),
            (VestaboardError::WidgetError(e1), VestaboardError::WidgetError(e2)) => e1 == e2,
            (VestaboardError::ScheduleError(e1), VestaboardError::ScheduleError(e2)) => e1 == e2,
            _ => false,
        }
    }
}
