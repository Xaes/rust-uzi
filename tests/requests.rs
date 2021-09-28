extern crate rust_uzi;

use hyper::Method;
use rand::Rng;
use rust_uzi::test_case::{Case, TestCase};
use std::net::SocketAddr;
use warp::Filter;

fn get_case() -> TestCase {
    let mut case = TestCase::new(None);

    // Defining Cases.

    case.cases.insert(
        "first_case",
        Case::new("0.0.0.0:8000", "/all-success", Some(Method::GET)),
    );
    case.cases.insert(
        "second_case",
        Case::new("0.0.0.0:8000", "/all-denied", Some(Method::GET)),
    );
    case.cases.insert(
        "third_case",
        Case::new("0.0.0.0:8000", "/mixed", Some(Method::GET)),
    );

    case
}

fn get_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let all_success = warp::path!("all-success").and(warp::get()).map(warp::reply);

    let all_denied = warp::path!("all-denied")
        .and(warp::get())
        .map(warp::reply)
        .map(|reply| warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST));

    let mixed_responses = warp::path!("mixed")
        .and(warp::get())
        .map(warp::reply)
        .map(|reply| {
            let chance_result = rand::thread_rng().gen_range(0..=100);
            match chance_result < 50 {
                false => warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST),
                true => warp::reply::with_status(reply, warp::http::StatusCode::OK),
            }
        });

    all_success
        .or(all_denied)
        .or(mixed_responses)
        .with(warp::filters::log::log("api-test"))
}

#[tokio::test]
async fn single_thread_api_test() {
    let address: SocketAddr = "0.0.0.0:8000".parse().unwrap();

    tokio::spawn(async move {
        warp::serve(get_routes()).run(address).await;
    });

    get_case().run().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_thread_api_test() {
    let address: SocketAddr = "0.0.0.0:8001".parse().unwrap();

    tokio::spawn(async move {
        warp::serve(get_routes()).run(address).await;
    });

    get_case().run().await;
}
