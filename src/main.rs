use crate::api::app::App;

mod api;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut app = App::new();
    app.run().await;
}
