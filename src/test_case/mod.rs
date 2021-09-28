use crate::result::Result;
use hyper::{Client, Method, Uri};
use std::collections::HashMap;
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
    pub runs: u32,
    pub cases: HashMap<&'static str, Case>,
    pub result: Option<Result>,
}

impl TestCase {
    pub fn new(runs: Option<u32>) -> Self {
        Self {
            runs: runs.unwrap_or(100),
            cases: HashMap::new(),
            result: None,
        }
    }

    pub async fn run(self) {
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();
        let client = Client::new();

        // Spawning Tasks.

        self.cases.iter().for_each(|(_, case)| {
            (0..=self.runs).into_iter().for_each(|run_index| {
                let client_clone = client.clone();
                let endpoint_clone = case.clone().endpoint;

                let uri = Uri::builder()
                    .scheme("http")
                    .authority(case.host)
                    .path_and_query(case.endpoint)
                    .build()
                    .unwrap();

                println!("Spawning Request: ({:?})", uri);

                let task = tokio::spawn(async move {
                    match client_clone.get(uri).await {
                        Ok(_) => println!(
                            "Request #{} to {} was successful.",
                            run_index, endpoint_clone
                        ),
                        Err(_) => {
                            println!("Request #{} to {} failed.", run_index, endpoint_clone)
                        }
                    }
                });

                tasks.push(task);
            })
        });

        // Awaiting for spawned tasks.

        for task in tasks {
            task.await;
        }
    }
}
