use config::Config;
use std::env;
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
    print_current_directory();

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
        db_settings.host,
        db_settings.port,
        db_settings.user,
        db_settings.password,
        db_settings.dbname
    );

    let (client, connection) = tokio_postgres::connect(&connection_str, NoTls).await?;

    // Запуск окремої задачі для підтримки з'єднання
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Вбудовуємо SQL-скрипт на етапі компіляції.
    // Шлях вказується відносно розташування цього файлу (pgdb.rs)
    const SQL_SCRIPT: &str = include_str!("../sql/init_db.sql");

    // Виконання SQL-скрипту
    client.batch_execute(SQL_SCRIPT).await?;

    println!("Database initialized successfully");
    Ok(())
}

fn print_current_directory() {
    match env::current_dir() {
        Ok(path) => println!("Поточна робоча директорія: {}", path.display()),
        Err(e) => eprintln!("Не вдалося отримати поточну директорію: {}", e),
    }
}
