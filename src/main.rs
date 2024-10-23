mod process_analyze;

#[tokio::main]
async fn main() {
    let (process_summary, _) = process_analyze::analyze_process_status().await;
    println!("{:#?}", process_summary);
}
