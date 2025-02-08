

use tokio_postgres::{Error, NoTls};


pub async fn init_db() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=osanwe_admin password=123456 dbname=osanwe_dev",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    client
        .batch_execute(
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
