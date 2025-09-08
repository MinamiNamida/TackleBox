use std::{error::Error, time::Duration};

use crate::api::app::App;

use migration::{Migrator, MigratorTrait};
use sea_orm::ConnectOptions;

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let mut opt = ConnectOptions::new("postgres://postgres:password@localhost/tacklebox");
    opt.max_connections(100)
        .min_connections(5)
        // .connect_timeout(Duration::from_secs(8))
        // .acquire_timeout(Duration::from_secs(8))
        // .idle_timeout(Duration::from_secs(8))
        // .max_lifetime(Duration::from_secs(8))
        // .sqlx_logging(true)
        .set_schema_search_path("public"); // Setting default PostgreSQL schema
        let connection = sea_orm::Database::connect(opt).await?;
    Migrator::down(&connection, None).await?;
    Migrator::up(&connection, None).await?;

    let mut app = App::new().await?;
    app.run().await;
    Ok(())
}
