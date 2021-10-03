use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let format_description = format_description::parse("[hour]:[minute]:[second]").unwrap();
        let start_time = OffsetDateTime::from(self.start_time)
            .format(&format_description)
            .unwrap();
        let end_time = OffsetDateTime::from(self.end_time)
            .format(&format_description)
            .unwrap();

        writeln!(f, "[---- TEST RESULTS ----]\n")?;
        writeln!(
            f,
            "[Total Requests Sent]: {}.",
            self.failed_requests + self.successful_requests
        )?;
        writeln!(f, "[Number of Fails]: {}.", self.failed_requests)?;
        writeln!(f, "[Number of Successes]: {}.", self.successful_requests)?;
        writeln!(
            f,
            "[Total Time Elapsed]: {:.4} secs.",
            self.duration.as_secs_f32()
        )?;
        writeln!(f, "[Start Time]: {}.", start_time)?;
        writeln!(f, "[End Time]: {}.", end_time)
    }
}

#[derive(Debug, Clone)]
pub struct TestResultBuilder {
    start_instant: Option<Instant>,
    start_time: Option<OffsetDateTime>,
    failed_requests: Arc<Mutex<u32>>,
    successful_requests: Arc<Mutex<u32>>,
}

impl Default for TestResultBuilder {
    fn default() -> Self {
        TestResultBuilder {
            failed_requests: Arc::new(Mutex::new(0)),
            successful_requests: Arc::new(Mutex::new(0)),
            start_instant: None,
            start_time: None,
        }
    }
}

impl TestResultBuilder {
    pub fn new() -> Self {
        TestResultBuilder::default()
    }

    pub fn start(&mut self) {
        self.start_instant = Some(Instant::now());
        self.start_time = Some(OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc()));
    }

    pub fn increment_err_count(&mut self) {
        let mut number = self.failed_requests.lock().unwrap();
        *number += 1;
    }

    pub fn increment_ok_count(&mut self) {
        let mut number = self.successful_requests.lock().unwrap();
        *number += 1;
    }

    pub fn build(self) -> TestResult {
        let failed_requests_count = *self.failed_requests.lock().unwrap();
        let successful_requests_count = *self.successful_requests.lock().unwrap();

        TestResult {
            start_time: self.start_time.unwrap(),
            end_time: OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc()),
            duration: self.start_instant.unwrap().elapsed(),
            failed_requests: failed_requests_count,
            successful_requests: successful_requests_count,
        }
    }
}
