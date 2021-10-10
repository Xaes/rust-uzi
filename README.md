# UZI

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

_Note: This is a personal project that I am developing to consolidate my learning in Rust, not a production ready tool / lib._

A load testing tool for API's made in Rust for measuring Web API's performance. It provides a flexible facility for generating various HTTP workloads. Web requests are made with the [Hyper](https://github.com/hyperium/hyper) (a low-level HTTP Client), so you can expect it to be blazing fast.

## Features

- Multi-threaded or Single-threaded HTTP Requests.
- Run by tests cases by time or by iterations.
- Metrics Analysis and comparation.
- Query Builder.
- Fake Data insertion.
- Test Cases.
- HTTP 1.1 / 2.0 Support.
- JSON Outputs.

## Tests

A simple concurrent test is already implemented. The test consists on creating a Warp Server, then, UZI makes GET requests to the three main endpoints. To execute the tests (with logs) use the following command:

```bash
RUST_LOG="rust_uzi::test_case=info,warp:test=info" cargo test multi_thread_api_test -- --nocapture
```

If you don't wanna to execute with logs you may want to use:

```bash
cargo test multi_thread_api_test -- --nocapture
```

There is a single threaded test as well. You can execute it with:

```bash
cargo test single_thread_api_test -- --nocapture
```
