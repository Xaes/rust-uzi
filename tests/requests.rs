extern crate rust_uzi;

use futures::channel::oneshot::channel;
use rand::Rng;
use rust_uzi::test_case::{Case, TestCase};
use std::net::SocketAddr;
use warp::Filter;

fn get_case() -> TestCase {
    let default_iters = 1000;
    TestCase::builder("test_case".to_string())
        .host("localhost:8000".to_string())
        .case(Case::new(
            "first_case".to_string(),
            "/all-success".to_string(),
            default_iters,
        ))
        .case(Case::new(
            "second_case".to_string(),
            "/all-denied".to_string(),
            default_iters,
        ))
        .case(Case::new(
            "third_case".to_string(),
            "/mixed".to_string(),
            default_iters,
        ))
        .build()
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
        .with(warp::filters::log::log("warp:test"))
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_thread_api_test() {
    pretty_env_logger::init();
    let (shutdown_sender, shutdown_receiver) = channel::<()>();
    let address: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let (_, server_future) =
        warp::serve(get_routes()).bind_with_graceful_shutdown(address, async {
            shutdown_receiver.await.ok();
        });

    tokio::spawn(server_future);

    let result = get_case().run().await;
    shutdown_sender.send(()).unwrap();
    println!("{}", result);
}
