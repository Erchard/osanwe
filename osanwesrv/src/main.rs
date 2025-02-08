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
        let transaction = request.into_inner();
        println!("Received transaction: {:?}", transaction);

        let response = TransactionResponse {
            status: "Transaction received".to_string(),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
