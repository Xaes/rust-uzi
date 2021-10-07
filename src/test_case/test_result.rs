use super::Case;
use std::collections::HashMap;

use colored::*;
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
    pub result_cases: HashMap<&'static str, CaseResult>,
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let format_description = format_description::parse("[hour]:[minute]:[second]").unwrap();
        let total_requests = self.failed_requests + self.successful_requests;
        let duration = self.duration.as_secs_f32();
        let requests_per_sec = (total_requests as f32) / self.duration.as_secs_f32();
        let start_time = self.start_time.format(&format_description).unwrap();
        let end_time = self.end_time.format(&format_description).unwrap();

        // Printing Results.

        writeln!(f, "{}\n", "TEST RESULTS".bold().yellow())?;
        writeln!(
            f,
            "{}: {}.",
            "Total Requests Sent".bold(),
            total_requests.to_string().yellow()
        )?;
        writeln!(
            f,
            "{}: {}.",
            "Number of Fails".bold(),
            self.failed_requests.to_string().red()
        )?;
        writeln!(
            f,
            "{}: {}.",
            "Number of Successes".bold(),
            self.successful_requests.to_string().green()
        )?;
        writeln!(
            f,
            "{}: {} req/sec.",
            "Requests per Second".bold(),
            requests_per_sec.to_string().yellow()
        )?;
        writeln!(
            f,
            "{}: {:.4} secs.",
            "Total Time Elapsed".bold(),
            duration.to_string().yellow()
        )?;
        writeln!(
            f,
            "{}: {}.",
            "Start Time".bold(),
            start_time.yellow()
        )?;
        writeln!(
            f,
            "{}: {}.",
            "End Time".bold(),
            end_time.yellow()
        )?;

        // Printing Results per endpoint.

        let mut table = table!([
            "Endpoint".bold(),
            "Error Count".bold().red(),
            "Success Count".bold().green()
        ]);

        self.result_cases.iter().for_each(|(_, case)| {
            table.add_row(row![
                case.case.endpoint,
                case.error_count.to_string().red(),
                case.success_count.to_string().green()
            ]);
        });

        writeln!(f, "\n{}\n", "ENDPOINTS".bold().yellow())?;
        writeln!(f, "{}", table)
    }
}

#[derive(Debug, Clone)]
pub struct TestResultBuilder {
    start_instant: Option<Instant>,
    start_time: Option<OffsetDateTime>,
    failed_requests: Arc<Mutex<u32>>,
    successful_requests: Arc<Mutex<u32>>,
    result_cases: Arc<Mutex<HashMap<&'static str, CaseResult>>>,
}

impl Default for TestResultBuilder {
    fn default() -> Self {
        TestResultBuilder {
            failed_requests: Arc::new(Mutex::new(0)),
            successful_requests: Arc::new(Mutex::new(0)),
            start_instant: None,
            start_time: None,
            result_cases: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl TestResultBuilder {
    pub fn new() -> Self {
        TestResultBuilder::default()
    }

    pub fn start(&mut self) {
        self.start_instant = Some(Instant::now());
        self.start_time = Some(OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()));
    }

    pub fn register_success(&mut self, case: &Case) {
        let mut success_number = self.successful_requests.lock().unwrap();
        let mut result_map = self.result_cases.lock().unwrap();
        *success_number += 1;
        result_map
            .entry(case.id)
            .and_modify(|case| {
                (*case).success_count += 1;
            })
            .or_insert(CaseResult {
                case: *case,
                error_count: 0,
                success_count: 1,
            });
    }

    pub fn register_error(&mut self, case: &Case) {
        let mut error_number = self.failed_requests.lock().unwrap();
        let mut result_map = self.result_cases.lock().unwrap();
        *error_number += 1;
        result_map
            .entry(case.id)
            .and_modify(|case| {
                (*case).error_count += 1;
            })
            .or_insert(CaseResult {
                case: *case,
                error_count: 1,
                success_count: 0,
            });
    }

    pub fn build(self) -> TestResult {
        let failed_requests_count = *self.failed_requests.lock().unwrap();
        let successful_requests_count = *self.successful_requests.lock().unwrap();
        let result_map = &*self.result_cases.lock().unwrap();

        TestResult {
            start_time: self.start_time.unwrap(),
            end_time: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            duration: self.start_instant.unwrap().elapsed(),
            failed_requests: failed_requests_count,
            successful_requests: successful_requests_count,
            result_cases: result_map.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CaseResult {
    case: Case,
    error_count: u32,
    success_count: u32,
}

impl std::convert::From<&Case> for CaseResult {
    fn from(case: &Case) -> CaseResult {
        CaseResult {
            case: *case,
            error_count: 0,
            success_count: 0,
        }
    }
}
