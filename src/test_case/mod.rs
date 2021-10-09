mod test_result;

use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use std::default::Default;
use std::time::Duration;
use test_result::{TestResult, TestResultBuilder};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub endpoint: String,
    pub iters: u32,
}

impl Case {
    pub fn new(id: String, endpoint: String, iters: u32) -> Self {
        Self {
            id,
            endpoint,
            iters,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub hostname: String,
    pub cases: HashMap<String, Case>,
    pub timeout: Duration,
}

impl<'a> TestCase {
    pub fn builder(name: String) -> TestCaseBuilder {
        TestCaseBuilder::new(name)
    }

    pub async fn run(self) -> TestResult {
        let https = self.hostname.starts_with("https");
        let https_connector = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);
        let mut test_builder = TestResultBuilder::new(self.hostname.clone());
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();

        // Initializing Test Benchmark.

        test_builder.start();

        // Spawning Tasks.

        self.cases.iter().for_each(|(_, case)| {
            (0..case.iters).into_iter().for_each(|run_index| {
                let client_clone = client.clone();
                let case_clone = case.clone();
                let endpoint = case.endpoint.clone();
                let timeout_clone = self.timeout;
                let hostname = self.hostname.clone();
                let mut builder_clone = test_builder.clone();
                let authority =
                    hostname[hostname.find("//").map(|v| v + 2).unwrap_or(0)..].to_string();

                let uri = Uri::builder()
                    .scheme(if https { "https" } else { "http" })
                    .authority(authority)
                    .path_and_query(endpoint)
                    .build()
                    .unwrap();

                info!("Spawning Request: ({:?})", uri);

                let task = tokio::spawn(async move {
                    match tokio::time::timeout(timeout_clone, client_clone.get(uri)).await {
                        Ok(result) => match result {
                            Ok(_) => match result.unwrap().status().is_success() {
                                true => {
                                    builder_clone.register_success(&case_clone);
                                    info!(
                                        "Request #{} to {} was successful.",
                                        run_index, case_clone.endpoint
                                    );
                                }
                                false => {
                                    builder_clone.register_error(&case_clone);
                                    info!(
                                        "Request #{} to {} failed.",
                                        run_index, case_clone.endpoint
                                    );
                                }
                            },
                            Err(_) => {
                                builder_clone.register_error(&case_clone);
                                info!("Request #{} to {} failed.", run_index, case_clone.endpoint)
                            }
                        },
                        Err(_) => {
                            builder_clone.register_error(&case_clone);
                            info!(
                                "Request #{} to {} failed (timeout).",
                                run_index, case_clone.endpoint
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

        test_builder.build()
    }
}

#[derive(Debug, Default)]
pub struct TestCaseBuilder {
    pub name: String,
    pub cases: HashMap<String, Case>,
    pub timeout: Duration,
    pub hostname: String,
}

impl TestCaseBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            timeout: Duration::from_secs(15),
            ..Default::default()
        }
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    pub fn case(mut self, case: Case) -> Self {
        self.cases.insert(case.id.clone(), case);
        self
    }

    pub fn host(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn build(self) -> TestCase {
        TestCase {
            name: self.name,
            timeout: self.timeout,
            cases: self.cases,
            hostname: self.hostname,
        }
    }
}
