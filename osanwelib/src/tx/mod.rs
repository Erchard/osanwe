use crate::generated::TransactionPb;
use hex::decode;

/// Структура, підготовлена до збереження в БД
#[derive(Debug, Clone)]
pub struct TransactionDb {
    pub transaction_hash: String,        // 0x...
    pub transaction_type: u32,           // число
    pub currency_id: u32,                // число
    pub amount: String,                  // 0x...
    pub decimal: String,                 // 0x...
    pub timestamp: u64,                  // число
    pub sender_address: String,          // 0x...
    pub sender_output_index: u32,        // число
    pub recipient_address: String,       // 0x...
    pub sender_signature: String,        // 0x...
    pub source_transaction_hash: String, // 0x...
}

/// Допоміжна функція для конвертації байтових полів у формат 0x...
fn to_hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Конвертація TransactionPb у TransactionDb
pub fn to_transaction_db(tx: &TransactionPb) -> TransactionDb {
    TransactionDb {
        transaction_hash: to_hex_string(&tx.transaction_hash),
        transaction_type: tx.transaction_type,
        currency_id: tx.currency_id,
        amount: to_hex_string(&tx.amount),
        decimal: to_hex_string(&tx.decimal),
        timestamp: tx.timestamp,
        sender_address: to_hex_string(&tx.sender_address),
        sender_output_index: tx.sender_output_index,
        recipient_address: to_hex_string(&tx.recipient_address),
        sender_signature: to_hex_string(&tx.sender_signature),
        source_transaction_hash: to_hex_string(&tx.source_transaction_hash),
    }
}

/// Конвертація TransactionDb у TransactionPb
pub fn from_transaction_db(
    tx_db: &TransactionDb,
) -> Result<TransactionPb, Box<dyn std::error::Error>> {
    // Перевірка та декодування кожного шістнадцяткового поля
    validate_hex_length_with_prefix(&tx_db.transaction_hash, 32)?;
    validate_hex_length_with_prefix(&tx_db.amount, 32)?;
    validate_hex_length_with_prefix(&tx_db.decimal, 1)?;
    validate_hex_length_with_prefix(&tx_db.sender_address, 20)?;
    validate_hex_length_with_prefix(&tx_db.recipient_address, 20)?;
    validate_hex_length_with_prefix(&tx_db.sender_signature, 65)?;
    validate_hex_length_with_prefix(&tx_db.source_transaction_hash, 32)?;

    // Декодування шістнадцяткових рядків у Vec<u8>
    let transaction_hash = decode(&tx_db.transaction_hash[2..])?; // Видаляємо "0x"
    let amount = decode(&tx_db.amount[2..])?;
    let decimal = decode(&tx_db.decimal[2..])?;
    let sender_address = decode(&tx_db.sender_address[2..])?;
    let recipient_address = decode(&tx_db.recipient_address[2..])?;
    let sender_signature = decode(&tx_db.sender_signature[2..])?;
    let source_transaction_hash = decode(&tx_db.source_transaction_hash[2..])?;

    // Створення TransactionPb
    Ok(TransactionPb {
        transaction_hash,
        transaction_type: tx_db.transaction_type,
        currency_id: tx_db.currency_id,
        amount,
        decimal,
        timestamp: tx_db.timestamp,
        sender_address,
        sender_output_index: tx_db.sender_output_index,
        recipient_address,
        sender_signature,
        source_transaction_hash,
    })
}

/// Допоміжна функція для перевірки довжини шістнадцяткового рядка з префіксом 0x
fn validate_hex_length_with_prefix(hex: &str, expected_bytes: usize) -> Result<(), String> {
    let expected_length = expected_bytes * 2;
    if !hex.starts_with("0x") {
        return Err(format!("Hex string '{}' does not start with '0x'", hex));
    }
    if hex.len() != expected_length + 2 {
        return Err(format!(
            "Invalid hex length for '{}': expected {} bytes ({} hex chars + 2 for '0x'), got {} hex chars",
            hex,
            expected_bytes,
            expected_bytes * 2,
            hex.len() - 2
        ));
    }
    Ok(())
}

/// Конвертація TransactionDb у TransactionPb, очікуючи наявність префікса 0x
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
    // Використовуємо validate_hex_length_with_prefix
    validate_hex_length_with_prefix(transaction_hash, 32)?;
    validate_hex_length_with_prefix(amount, 32)?;
    validate_hex_length_with_prefix(decimal, 1)?;
    validate_hex_length_with_prefix(sender_address, 20)?;
    validate_hex_length_with_prefix(recipient_address, 20)?;
    validate_hex_length_with_prefix(sender_signature, 65)?;
    validate_hex_length_with_prefix(source_transaction_hash, 32)?;

    // Видаляємо "0x" перед декодуванням
    Ok(TransactionPb {
        transaction_hash: decode(&transaction_hash[2..])?,
        transaction_type: transaction_type.parse()?,
        currency_id: currency_id.parse()?,
        amount: decode(&amount[2..])?,
        decimal: decode(&decimal[2..])?,
        timestamp: timestamp.parse()?,
        sender_address: decode(&sender_address[2..])?,
        sender_output_index: sender_output_index.parse()?,
        recipient_address: decode(&recipient_address[2..])?,
        sender_signature: decode(&sender_signature[2..])?,
        source_transaction_hash: decode(&source_transaction_hash[2..])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transaction_pb_with_prefix_only() {
        let transaction = parse_transaction_pb(
        "0xa3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde",
        "1",
        "100",
        "0xa3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde",
        "0x93",
        "1700000000",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd", // 40 hex chars (20 bytes) з префіксом
        "5",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd", // 40 hex chars (20 bytes) з префіксом
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1f1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "0xa3b1c2d3e4f5678901234567890abcdef1234567890abcdef1234567890abcde"
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

    #[test]
    fn test_to_transaction_db() {
        // Створимо "пустий" TransactionPb для прикладу
        let pb = TransactionPb {
            transaction_hash: vec![0xAA; 32],
            transaction_type: 1,
            currency_id: 100,
            amount: vec![0xBB; 32],
            decimal: vec![0x01],
            timestamp: 1700000000,
            sender_address: vec![0xCC; 20],
            sender_output_index: 99,
            recipient_address: vec![0xDD; 20],
            sender_signature: vec![0xEE; 65],
            source_transaction_hash: vec![0xFF; 32],
        };

        let db_record = to_transaction_db(&pb);

        // Перевіримо довжини полів, де це актуально
        // (для прикладу — transaction_hash має бути 0x + 64 hex-символи => 66 довжина)
        assert_eq!(db_record.transaction_hash.len(), 66);
        assert_eq!(db_record.amount.len(), 66);
        assert_eq!(db_record.decimal.len(), 4); // "0x" + 2 hex-символи
        assert_eq!(db_record.sender_address.len(), 42); // "0x" + 40 hex-символів
        assert_eq!(db_record.recipient_address.len(), 42);
        assert_eq!(db_record.sender_signature.len(), 132); // "0x" + 130 hex-символів
        assert_eq!(db_record.source_transaction_hash.len(), 66);

        assert_eq!(db_record.transaction_type, 1);
        assert_eq!(db_record.currency_id, 100);
        assert_eq!(db_record.sender_output_index, 99);

        // Перевіримо timestamp
        assert_eq!(db_record.timestamp, 1700000000);
    }

    #[test]
    fn test_from_transaction_db() {
        // Створимо приклад TransactionDb
        let db = TransactionDb {
            transaction_hash: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            transaction_type: 1,
            currency_id: 100,
            amount: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            decimal: "0x01".to_string(),
            timestamp: 1700000000,
            sender_address: "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
            sender_output_index: 99,
            recipient_address: "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
            sender_signature: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            source_transaction_hash: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        };

        let pb = from_transaction_db(&db).unwrap();

        // Перевіримо поля
        assert_eq!(pb.transaction_type, 1);
        assert_eq!(pb.currency_id, 100);
        assert_eq!(pb.amount.len(), 32);
        assert_eq!(pb.decimal.len(), 1);
        assert_eq!(pb.timestamp, 1700000000);
        assert_eq!(pb.sender_output_index, 99);
        assert_eq!(pb.transaction_hash.len(), 32);
        assert_eq!(pb.sender_address.len(), 20);
        assert_eq!(pb.recipient_address.len(), 20);
        assert_eq!(pb.sender_signature.len(), 65);
        assert_eq!(pb.source_transaction_hash.len(), 32);

        // Перевіримо зміст байтових полів
        assert!(pb.transaction_hash.iter().all(|&b| b == 0xAA));
        assert!(pb.amount.iter().all(|&b| b == 0xBB));
        assert!(pb.decimal == vec![0x01]);
        assert!(pb.sender_address.iter().all(|&b| b == 0xCC));
        assert!(pb.recipient_address.iter().all(|&b| b == 0xDD));
        assert!(pb.sender_signature.iter().all(|&b| b == 0xEE));
        assert!(pb.source_transaction_hash.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_round_trip_conversion() {
        // Створимо TransactionPb
        let original_pb = TransactionPb {
            transaction_hash: vec![0xAA; 32],
            transaction_type: 2,
            currency_id: 200,
            amount: vec![0xBB; 32],
            decimal: vec![0x02],
            timestamp: 1800000000,
            sender_address: vec![0xCC; 20],
            sender_output_index: 100,
            recipient_address: vec![0xDD; 20],
            sender_signature: vec![0xEE; 65],
            source_transaction_hash: vec![0xFF; 32],
        };

        // Конвертуємо в TransactionDb
        let db = to_transaction_db(&original_pb);

        // Конвертуємо назад у TransactionPb
        let converted_pb = from_transaction_db(&db).unwrap();

        // Перевіримо, що початковий та конвертований TransactionPb збігаються
        assert_eq!(original_pb.transaction_hash, converted_pb.transaction_hash);
        assert_eq!(original_pb.transaction_type, converted_pb.transaction_type);
        assert_eq!(original_pb.currency_id, converted_pb.currency_id);
        assert_eq!(original_pb.amount, converted_pb.amount);
        assert_eq!(original_pb.decimal, converted_pb.decimal);
        assert_eq!(original_pb.timestamp, converted_pb.timestamp);
        assert_eq!(original_pb.sender_address, converted_pb.sender_address);
        assert_eq!(
            original_pb.sender_output_index,
            converted_pb.sender_output_index
        );
        assert_eq!(
            original_pb.recipient_address,
            converted_pb.recipient_address
        );
        assert_eq!(original_pb.sender_signature, converted_pb.sender_signature);
        assert_eq!(
            original_pb.source_transaction_hash,
            converted_pb.source_transaction_hash
        );
    }

    #[test]
    fn test_invalid_hex_prefix() {
        // Створимо TransactionDb з некоректним префіксом
        let db = TransactionDb {
            transaction_hash: "1xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), // Некоректний префікс
            transaction_type: 1,
            currency_id: 100,
            amount: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            decimal: "0x01".to_string(),
            timestamp: 1700000000,
            sender_address: "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
            sender_output_index: 99,
            recipient_address: "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
            sender_signature: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            source_transaction_hash: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        };

        let result = from_transaction_db(&db);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hex_length() {
        // Створимо TransactionDb з некоректною довжиною шістнадцяткового рядка
        let db = TransactionDb {
            transaction_hash: "0xaaa".to_string(), // Недостатньо символів
            transaction_type: 1,
            currency_id: 100,
            amount: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            decimal: "0x01".to_string(),
            timestamp: 1700000000,
            sender_address: "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
            sender_output_index: 99,
            recipient_address: "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
            sender_signature: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            source_transaction_hash: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        };

        let result = from_transaction_db(&db);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hex_content() {
        // Створимо TransactionDb з некоректним шістнадцятковим вмістом
        let db = TransactionDb {
            transaction_hash: "0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ".to_string(), // Некоректні символи
            transaction_type: 1,
            currency_id: 100,
            amount: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            decimal: "0x01".to_string(),
            timestamp: 1700000000,
            sender_address: "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
            sender_output_index: 99,
            recipient_address: "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
            sender_signature: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            source_transaction_hash: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        };

        let result = from_transaction_db(&db);
        assert!(result.is_err());
    }
}
