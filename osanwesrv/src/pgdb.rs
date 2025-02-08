use config::Config;
use tokio_postgres::{Error, NoTls};

#[derive(serde::Deserialize)]
struct DatabaseSettings {
    url: String,
}

pub async fn init_db() -> Result<(), Error> {
    // Завантажуємо конфігурацію
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let db_settings: DatabaseSettings = settings
        .get::<DatabaseSettings>("database")
        .expect("Database settings not found");

    let (client, connection) = tokio_postgres::connect(&db_settings.url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS transactions (
            id SERIAL PRIMARY KEY,
            details TEXT NOT NULL
        );
        INSERT INTO transactions (details) VALUES
            ('Transaction 1'),
            ('Transaction 2');
        ",
    )
    .await?;

    println!("Database initialized successfully");
    Ok(())
}
