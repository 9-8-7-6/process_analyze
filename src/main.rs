mod process_analyze;

#[tokio::main]
async fn main() {
    process_analyze::analyze_process_status().await;
}
