use tonic::{transport::Server, Request, Response, Status};
use osanwelib::generated::{
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionPb, TransactionResponse,
};
use async_trait::async_trait;

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
    let addr = "[::1]:50051".parse()?;
    let transaction_service = MyTransactionService::default();

    println!("Osanwe gRPC Server running on {}", addr);

    Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .serve(addr)
        .await?;

    Ok(())
}
