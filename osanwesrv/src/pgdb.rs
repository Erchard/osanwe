use config::Config;
use std::env;
use tokio_postgres::{Client, Error, NoTls};
use osanwelib::generated::TransactionPb; 

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


/// Функція для отримання клієнта БД (повторно використовувана логіка)
pub async fn get_db_client() -> Result<Client, Error> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let db_settings: DatabaseSettings = settings
        .get::<DatabaseSettings>("database")
        .expect("Database settings not found");

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

    Ok(client)
}

/// Збереження транзакції в базі даних.
/// Функція приймає об'єкт TransactionPb і вставляє його в таблицю transactions.
/// Використовується ON CONFLICT для уникнення помилки при повторному збереженні транзакції з однаковим хешем.
pub async fn save_transaction(tx: TransactionPb) -> Result<(), Box<dyn std::error::Error>> {
    let client = get_db_client().await?;

    let stmt = client
        .prepare(
            "INSERT INTO transactions (
            transaction_hash,
            transaction_type,
            currency_id,
            amount,
            timestamp,
            sender_address,
            sender_output_index,
            recipient_address,
            sender_signature,
            source_transaction_hash
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (transaction_hash) DO NOTHING",
        )
        .await?;

    client
        .execute(
            &stmt,
            &[
                &tx.transaction_hash,
                // Приведення типів: PostgreSQL очікує SMALLINT для transaction_type та INTEGER для currency_id
                &(tx.transaction_type as i16),
                &(tx.currency_id as i32),
                &tx.amount,
                &(tx.timestamp as i64),
                &tx.sender_address,
                &(tx.sender_output_index as i32),
                &tx.recipient_address,
                &tx.sender_signature,
                &tx.source_transaction_hash,
            ],
        )
        .await?;

    println!("Transaction saved successfully.");
    Ok(())
}

fn print_current_directory() {
    match env::current_dir() {
        Ok(path) => println!("Поточна робоча директорія: {}", path.display()),
        Err(e) => eprintln!("Не вдалося отримати поточну директорію: {}", e),
    }
}
