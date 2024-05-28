#[tokio::main]
pub async fn main() {
    dotenv::dotenv().ok();
    data_intuitive::launch_server()
        .await
        .expect("Failed to launch server");
}
