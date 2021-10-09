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
use pretty_env_logger::env_logger;
use rust_uzi::test_case::{Case, TestCase};
use std::boxed::Box;

#[tokio::main]
async fn main() {
    let clap = clap_app!(UZI =>
        (name: "UZI".bold().red().to_string())
        (about: crate_description!())
        (version: crate_version!())
        (author: crate_authors!())
        (@arg log: -l --log ... "Sets the level of log information")
        (@arg iters: -i --iters ... "Sets quantity of requests that will be made per endpoint")
        (@arg host: +required "Address of the API you wish to test")
        (@arg endpoints: +required "Endpoints of the API separated by commas")
    )
    .get_matches();

    let cli: &'static mut clap::ArgMatches = Box::leak(Box::new(clap));

    if let Some(log_level) = cli.value_of("log") {
        let level: LevelFilter = log_level.parse().unwrap_or(LevelFilter::Off);
        env_logger::builder().filter_level(level);
    }

    let endpoints = cli.value_of("endpoints").unwrap();
    let hostname = cli.value_of("host").unwrap();
    let iterations: u32 = cli
        .value_of("iters")
        .map_or(500, |i| i.parse().unwrap_or(500));

    let mut test_builder = TestCase::builder("test_case").iters(iterations);

    for (index, endpoint) in endpoints.split(",").into_iter().enumerate() {
        test_builder = test_builder.case(Case::new(
            Box::leak(Box::new(format!("case_{}", index))),
            hostname,
            endpoint,
        ));
    }

    let results = test_builder.build().run().await;
    println!("{}", results);
}
