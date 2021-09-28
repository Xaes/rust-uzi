#[derive(Debug)]
pub struct Result {
    output_filepath: &'static str,
    duration: u32,
    failed_requests: u32,
    successful_requests: u32,
}
