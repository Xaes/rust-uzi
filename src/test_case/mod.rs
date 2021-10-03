mod test_result;

use hyper::{Client, Method, Uri};
use std::collections::HashMap;
use std::default::Default;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use test_result::TestResult;
use time::OffsetDateTime;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct Case {
    pub host: &'static str,
    pub endpoint: &'static str,
    pub method: Method,
}

impl Case {
    pub fn new(host: &'static str, endpoint: &'static str, method: Option<Method>) -> Self {
        Self {
            host,
            endpoint,
            method: method.unwrap_or(Method::GET),
        }
    }
}

#[derive(Debug)]
pub struct TestCase {
    pub name: &'static str,
    pub iterations: u32,
    pub cases: HashMap<&'static str, Case>,
    pub timeout: Duration,
}

impl TestCase {
    pub fn builder(name: &'static str) -> TestCaseBuilder {
        TestCaseBuilder::new(name)
    }

    pub async fn run(self) -> TestResult {
        let duration = Instant::now();
        let start_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let successful_requests = Arc::new(Mutex::new(0));
        let failed_requests = Arc::new(Mutex::new(0));
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();
        let client = Client::new();

        // Spawning Tasks.

        self.cases.iter().for_each(|(_, case)| {
            (0..self.iterations).into_iter().for_each(|run_index| {
                let client_clone = client.clone();
                let endpoint_clone = case.clone().endpoint;
                let timeout_clone = self.timeout.clone();
                let successful_requests_count = successful_requests.clone();
                let failed_requests_count = failed_requests.clone();

                let uri = Uri::builder()
                    .scheme("http")
                    .authority(case.host)
                    .path_and_query(case.endpoint)
                    .build()
                    .unwrap();

                info!("Spawning Request: ({:?})", uri);

                let task = tokio::spawn(async move {
                    match tokio::time::timeout(timeout_clone, client_clone.get(uri)).await {
                        Ok(result) => match result {
                            Ok(_) => match result.unwrap().status().is_success() {
                                true => {
                                    let mut num = successful_requests_count.lock().unwrap();
                                    *num += 1;

                                    info!(
                                        "Request #{} to {} was successful.",
                                        run_index, endpoint_clone
                                    );
                                }
                                false => {
                                    let mut num = failed_requests_count.lock().unwrap();
                                    *num += 1;
                                    error!("Request #{} to {} failed.", run_index, endpoint_clone);
                                }
                            },
                            Err(_) => {
                                let mut num = failed_requests_count.lock().unwrap();
                                *num += 1;
                                error!("Request #{} to {} failed.", run_index, endpoint_clone)
                            }
                        },
                        Err(_) => {
                            let mut num = failed_requests_count.lock().unwrap();
                            *num += 1;
                            error!(
                                "Request #{} to {} failed (timeout).",
                                run_index, endpoint_clone
                            );
                        }
                    }
                });

                tasks.push(task);
            })
        });

        // Awaiting for spawned tasks.

        for task in tasks {
            task.await.unwrap();
        }

        let end_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let duration = duration.elapsed();

        // Building a Test Result.

        let failed_requests_count = *failed_requests.lock().unwrap();
        let successful_requests_count = *successful_requests.lock().unwrap();

        TestResult {
            start_time,
            end_time,
            duration,
            failed_requests: failed_requests_count,
            successful_requests: successful_requests_count,
        }
    }
}

#[derive(Debug, Default)]
pub struct TestCaseBuilder {
    pub name: &'static str,
    pub cases: HashMap<&'static str, Case>,
    pub timeout: Duration,
    pub iterations: u32,
}

impl TestCaseBuilder {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            cases: HashMap::new(),
            timeout: Duration::from_secs(15),
            iterations: 300,
        }
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    pub fn case(mut self, name: &'static str, case: Case) -> Self {
        self.cases.insert(name, case);
        self
    }

    pub fn iters(mut self, iters: u32) -> Self {
        self.iterations = iters;
        self
    }

    pub fn build(self) -> TestCase {
        TestCase {
            name: self.name,
            timeout: self.timeout,
            cases: self.cases,
            iterations: self.iterations,
        }
    }
}
