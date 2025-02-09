mod pgdb;

use async_trait::async_trait;
use osanwelib::generated::{
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionPb, TransactionResponse,
};
use tokio::{signal, sync::oneshot};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyTransactionService;

#[async_trait]
impl TransactionService for MyTransactionService {
    async fn submit_transaction(
        &self,
        request: Request<TransactionPb>,
    ) -> Result<Response<TransactionResponse>, Status> {
        // Отримання транзакції з запиту
        let transaction = request.into_inner();
        println!("Received transaction: {:?}", transaction);

        // Спроба збереження транзакції в базі даних
        match pgdb::save_transaction(transaction).await {
            Ok(()) => {
                let response = TransactionResponse {
                    status: "Transaction received and saved".to_string(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                eprintln!("Failed to save transaction: {:?}", e);
                Err(Status::internal("Failed to save transaction"))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ініціалізація бази даних (створення таблиць, індексів тощо)
    pgdb::init_db().await?;

    let addr = "[::1]:50051".parse()?;
    let transaction_service = MyTransactionService::default();

    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .serve_with_shutdown(addr, async {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for shutdown signal");
            println!("Received shutdown signal");
            let _ = shutdown_tx.send(());
        });

    println!(
        "Osanwe gRPC Server running on {}. Press Ctrl+C to stop.",
        addr
    );
    server.await?;

    Ok(())
}
