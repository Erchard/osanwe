use crate::generated::{TransactionPb};
use hex::decode;


fn validate_hex_length(hex: &str, expected_bytes: usize) -> Result<(), String> {
    if hex.len() != expected_bytes * 2 {
        return Err(format!(
            "Invalid hex length for '{}': expected {} bytes ({} hex chars), got {} hex chars",
            hex,
            expected_bytes,
            expected_bytes * 2,
            hex.len()
        ));
    }
    Ok(())
}

pub fn parse_transaction_pb(
    transaction_hash: &str,
    transaction_type: &str,
    currency_id: &str,
    amount: &str,
    decimal: &str,
    timestamp: &str,
    sender_address: &str,
    sender_output_index: &str,
    recipient_address: &str,
    sender_signature: &str,
    source_transaction_hash: &str,
) -> Result<TransactionPb, Box<dyn std::error::Error>> {
    validate_hex_length(transaction_hash, 32)?;
    validate_hex_length(amount, 32)?;
    validate_hex_length(decimal, 1)?;
    validate_hex_length(sender_address, 20)?;
    validate_hex_length(recipient_address, 20)?;
    validate_hex_length(sender_signature, 65)?;
    validate_hex_length(source_transaction_hash, 32)?;

    Ok(TransactionPb {
        transaction_hash: decode(transaction_hash)?,
        transaction_type: transaction_type.parse()?,
        currency_id: currency_id.parse()?,
        amount: decode(amount)?,
        decimal: decode(decimal)?,
        timestamp: timestamp.parse()?,
        sender_address: decode(sender_address)?,
        sender_output_index: sender_output_index.parse()?,
        recipient_address: decode(recipient_address)?,
        sender_signature: decode(sender_signature)?,
        source_transaction_hash: decode(source_transaction_hash)?,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transaction_type1() {
        let transaction = parse_transaction_pb(
            "a3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde",
            "1",
            "100",
            "a3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde",
            "93",
            "1700000000",
            "abcdefabcdefabcdefabcdefabcdefabcdefabcd", // 40 hex chars (20 bytes)
            "5",
            "abcdefabcdefabcdefabcdefabcdefabcdefabcd", // 40 hex chars (20 bytes)
            "1234567890abcdef1234567890abcdef1234567890abcdef1f1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "a3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde"
        ).unwrap();

        assert_eq!(transaction.transaction_type, 1);
        assert_eq!(transaction.currency_id, 100);
        assert_eq!(transaction.amount.len(), 32);
        assert_eq!(transaction.decimal.len(), 1);
        assert_eq!(transaction.timestamp, 1700000000);
        assert_eq!(transaction.sender_output_index, 5);
        assert_eq!(transaction.transaction_hash.len(), 32);
        assert_eq!(transaction.sender_address.len(), 20);
        assert_eq!(transaction.recipient_address.len(), 20);
        assert_eq!(transaction.sender_signature.len(), 65);
        assert_eq!(transaction.source_transaction_hash.len(), 32);
    }

}
