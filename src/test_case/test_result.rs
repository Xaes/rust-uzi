use std::fmt::{Display, Formatter, Result};
use std::time::{Duration};
use time::{format_description, OffsetDateTime};

#[derive(Debug)]
pub struct TestResult {
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub duration: Duration,
    pub failed_requests: u32,
    pub successful_requests: u32,
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let format_description = format_description::parse("[hour]:[minute]:[second]").unwrap();
        let start_time = OffsetDateTime::from(self.start_time)
            .format(&format_description)
            .unwrap();

        let end_time = OffsetDateTime::from(self.end_time)
            .format(&format_description)
            .unwrap();

        write!(f, "[Test Results]: FAILED: {} / SUCCESSFULL: {} / Start Time: {} / End Time: {} / Duration: {:?}.",
        self.failed_requests, self.successful_requests, start_time, end_time, self.duration)
    }
}
