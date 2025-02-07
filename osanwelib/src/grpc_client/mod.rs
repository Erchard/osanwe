// osanwelib/src/grpc_client.rs
use crate::generated::{
    transaction_service_client::TransactionServiceClient, TransactionPb,
};

pub async fn send_transaction_to_server(tx: TransactionPb) -> Result<(), Box<dyn std::error::Error>> {
    // Підключення до gRPC сервера
    let mut client = TransactionServiceClient::connect("http://[::1]:50051").await?;
    
    // Відправка транзакції
    let response = client.submit_transaction(tx).await?;
    println!("Server response: {:?}", response.into_inner());
    Ok(())
}
