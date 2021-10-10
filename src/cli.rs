extern crate colored;
extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate clap;

pub mod test_case;

use colored::*;
use log::LevelFilter;
use rust_uzi::test_case::{Case, TestCase};
use tokio::runtime::Builder as RuntimeBuilder;

fn main() {
    let clap = clap_app!(UZI =>
        (name: "UZI".bold().red().to_string())
        (about: crate_description!())
        (version: crate_version!())
        (author: crate_authors!())
        (@arg host: +required "Address of the API you wish to test.")
        (@arg endpoints: +required "Endpoints of the API separated by commas. For example: '/todos, '/users'.")
        (@arg log: -l --log [LOG_LEVEL] "Sets the level of log information.")
        (@arg iters: -i --iters [ITERS] "Sets quantity of requests that will be made per endpoint. If no iterations are specified 500 iterations will be used instead.")
        (@arg threads: -t --threads [THREADS] "Sets the quantity of threads that the loading test will run with. If the thread parameter is equal to one, then the task will be executed on a single thread.")
    )
    .get_matches();

    if let Some(log_level) = clap.value_of("log") {
        let level: LevelFilter = log_level.parse().unwrap_or(LevelFilter::Off);
        pretty_env_logger::env_logger::builder()
            .filter_module("rust_uzi::test_case", level)
            .init();
    }

    let endpoints = clap.value_of("endpoints").unwrap();
    let hostname = clap.value_of("host").unwrap();

    let iterations: u32 = clap
        .value_of("iters")
        .map_or(500, |i| i.parse().unwrap_or(500));

    let threads: usize = clap
        .value_of("threads")
        .map_or(1, |i| i.parse().unwrap_or(1));

    // Building Test Case.

    let mut test_builder = TestCase::builder("test_case".to_string()).host(hostname.to_string());

    for (index, endpoint) in endpoints.split(',').into_iter().enumerate() {
        test_builder = test_builder.case(Case::new(
            format!("case_{}", index),
            endpoint.trim().to_string(),
            iterations,
        ));
    }

    // Creating executing environment.

    let runtime = match threads {
        1 => RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
        _ => RuntimeBuilder::new_multi_thread()
            .worker_threads(threads)
            .enable_all()
            .build()
            .unwrap(),
    };

    println!(
        "{} {} {}",
        "Running test with: ".bold(),
        threads.to_string().yellow().bold(),
        " thread(s).\n".bold()
    );

    runtime.block_on(async move {
        let results = test_builder.build().run().await;
        println!("{}", results);
    });
}
