use async_trait::async_trait;
use osanwelib::generated::{
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionPb, TransactionResponse,
};
use tokio::{signal, sync::oneshot};
use tokio_postgres::{Error, NoTls};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyTransactionService;

#[async_trait]
impl TransactionService for MyTransactionService {
    async fn submit_transaction(
        &self,
        request: Request<TransactionPb>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let transaction = request.into_inner();
        println!("Received transaction: {:?}", transaction);

        let response = TransactionResponse {
            status: "Transaction received".to_string(),
        };
        Ok(Response::new(response))
    }
}

async fn init_db() -> Result<(), Error> {
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
    "
        )
        .await?;

    println!("Database initialized successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_db().await?;

    let addr = "[::1]:50051".parse()?;
    let transaction_service = MyTransactionService::default();

    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .serve_with_shutdown(addr, async {
            signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
            println!("Received shutdown signal");
            let _ = shutdown_tx.send(());
        });

    println!("Osanwe gRPC Server running on {}. Press Ctrl+C to stop.", addr);
    server.await?;

    Ok(())
}
