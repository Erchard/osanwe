use crate::generated::{
    transaction_service_client::TransactionServiceClient, TransactionPb,
};

pub async fn send_transaction_to_server(tx: TransactionPb) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Підключення до gRPC сервера за адресою http://[::1]:50051");
    let mut client = TransactionServiceClient::connect("http://[::1]:50051").await?;
    
    log::info!("Відправка транзакції на сервер");
    let response = client.submit_transaction(tx).await?;
    
    log::info!("Відповідь сервера: {:?}", response.into_inner());
    Ok(())
}
