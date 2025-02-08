use config::Config;
use tokio_postgres::{Error, NoTls};

#[derive(serde::Deserialize)]
struct DatabaseSettings {
    host: String,
    port: String,
    user: String,
    password: String,
    dbname: String,
}

pub async fn init_db() -> Result<(), Error> {
    // Завантаження конфігурації з файлу config.toml та змінного оточення
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let db_settings: DatabaseSettings = settings
        .get::<DatabaseSettings>("database")
        .expect("Database settings not found");

    // Формування рядка підключення із завантажених налаштувань
    let connection_str = format!(
        "host={} port={} user={} password={} dbname={}",
        db_settings.host, db_settings.port, db_settings.user, db_settings.password, db_settings.dbname
    );

    let (client, connection) = tokio_postgres::connect(&connection_str, NoTls).await?;

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
