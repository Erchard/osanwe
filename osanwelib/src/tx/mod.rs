use crate::db;
use crate::generated::TransactionPb;
use hex::decode;
use std::error::Error;

/// Структура, підготовлена до збереження в БД
#[derive(Debug, Clone)]
pub struct TransactionDb {
    pub transaction_hash: String,
    pub transaction_type: u32,
    pub currency_id: u32,
    pub amount: String,
    pub timestamp: u64,
    pub sender_address: Option<String>,
    pub sender_output_index: Option<u32>,
    pub recipient_address: String,
    pub sender_signature: Option<String>,
    pub source_transaction_hash: Option<String>,
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
        timestamp: tx.timestamp,
        sender_address: if tx.sender_address.is_empty() {
            None
        } else {
            Some(to_hex_string(&tx.sender_address))
        },
        sender_output_index: if tx.sender_address.is_empty() {
            None
        } else {
            Some(tx.sender_output_index)
        },
        recipient_address: to_hex_string(&tx.recipient_address),
        sender_signature: if tx.sender_signature.is_empty() {
            None
        } else {
            Some(to_hex_string(&tx.sender_signature))
        },
        source_transaction_hash: if tx.source_transaction_hash.is_empty() {
            None
        } else {
            Some(to_hex_string(&tx.source_transaction_hash))
        },
    }
}

pub fn from_transaction_db(tx_db: &TransactionDb) -> Result<TransactionPb, Box<dyn Error>> {
    validate_hex_length_with_prefix(&tx_db.transaction_hash, 32)?;
    validate_hex_length_with_prefix(&tx_db.amount, 32)?;
    validate_hex_length_with_prefix(&tx_db.recipient_address, 20)?;

    let sender_address = match &tx_db.sender_address {
        Some(addr) => {
            validate_hex_length_with_prefix(addr, 20)?;
            decode(&addr[2..])?
        }
        None => Vec::new(), // Порожній `Vec<u8>` якщо немає даних
    };

    let sender_signature = match &tx_db.sender_signature {
        Some(sig) => {
            validate_hex_length_with_prefix(sig, 65)?;
            decode(&sig[2..])?
        }
        None => Vec::new(),
    };

    let source_transaction_hash = match &tx_db.source_transaction_hash {
        Some(hash) => {
            validate_hex_length_with_prefix(hash, 32)?;
            decode(&hash[2..])?
        }
        None => Vec::new(),
    };

    Ok(TransactionPb {
        transaction_hash: decode(&tx_db.transaction_hash[2..])?,
        transaction_type: tx_db.transaction_type,
        currency_id: tx_db.currency_id,
        amount: decode(&tx_db.amount[2..])?,
        timestamp: tx_db.timestamp,
        sender_address,
        sender_output_index: tx_db.sender_output_index.unwrap_or(0), // 0, якщо поле відсутнє
        recipient_address: decode(&tx_db.recipient_address[2..])?,
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
        timestamp: timestamp.parse()?,
        sender_address: decode(&sender_address[2..])?,
        sender_output_index: sender_output_index.parse()?,
        recipient_address: decode(&recipient_address[2..])?,
        sender_signature: decode(&sender_signature[2..])?,
        source_transaction_hash: decode(&source_transaction_hash[2..])?,
    })
}

/// Зберігає транзакцію у базі даних.
///
/// # Аргументи
///
/// * `tx` - Транзакція у форматі `TransactionPb`.
///
/// # Повертає
///
/// * `Ok(())` - Якщо збереження успішне.
/// * `Err(Box<dyn Error>)` - Якщо виникла помилка.
///
pub fn store_transaction(tx: &TransactionPb) -> Result<(), Box<dyn Error>> {
    let tx_db = to_transaction_db(tx);
    db::save_transaction(&tx_db)?;
    Ok(())
}

/// Отримує транзакцію з бази даних за її хешем.
///
/// # Аргументи
///
/// * `transaction_hash` - Хеш транзакції у форматі 0x...
///
/// # Повертає
///
/// * `Ok(TransactionPb)` - Якщо транзакція знайдена.
/// * `Err(Box<dyn Error>)` - Якщо транзакцію не знайдено або виникла помилка.
///
pub fn fetch_transaction(transaction_hash: &str) -> Result<TransactionPb, Box<dyn Error>> {
    let tx_db = db::get_transaction_by_hash(transaction_hash)?;
    let tx_pb = from_transaction_db(&tx_db)?;
    Ok(tx_pb)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transaction_pb() -> TransactionPb {
        TransactionPb {
            transaction_hash: vec![0xAA; 32],
            transaction_type: 1,
            currency_id: 100,
            amount: vec![0xBB; 32],
            timestamp: 1700000000,
            sender_address: vec![0xCC; 20],
            sender_output_index: 99,
            recipient_address: vec![0xDD; 20],
            sender_signature: vec![0xEE; 65],
            source_transaction_hash: vec![0xFF; 32],
        }
    }

    fn sample_transaction_db() -> TransactionDb {
        TransactionDb {
            transaction_hash: "0x".to_owned() + &"AA".repeat(32),
            transaction_type: 1,
            currency_id: 100,
            amount: "0x".to_owned() + &"BB".repeat(32),
            timestamp: 1700000000,
            sender_address: Some("0x".to_owned() + &"CC".repeat(20)),
            sender_output_index: Some(99),
            recipient_address: "0x".to_owned() + &"DD".repeat(20),
            sender_signature: Some("0x".to_owned() + &"EE".repeat(65)),
            source_transaction_hash: Some("0x".to_owned() + &"FF".repeat(32)),
        }
    }

    fn sample_transaction_pb_with_missing_fields() -> TransactionPb {
        TransactionPb {
            transaction_hash: vec![0xAA; 32],
            transaction_type: 1,
            currency_id: 100,
            amount: vec![0xBB; 32],
            timestamp: 1700000000,
            sender_address: Vec::new(),
            sender_output_index: 0,
            recipient_address: vec![0xDD; 20],
            sender_signature: Vec::new(),
            source_transaction_hash: Vec::new(),
        }
    }

    fn sample_transaction_db_with_missing_fields() -> TransactionDb {
        TransactionDb {
            transaction_hash: "0x".to_owned() + &"AA".repeat(32),
            transaction_type: 1,
            currency_id: 100,
            amount: "0x".to_owned() + &"BB".repeat(32),
            timestamp: 1700000000,
            sender_address: None,
            sender_output_index: None,
            recipient_address: "0x".to_owned() + &"DD".repeat(20),
            sender_signature: None,
            source_transaction_hash: None,
        }
    }

    #[test]
    fn test_conversion_to_transaction_db() {
        let pb = sample_transaction_pb();
        let db = to_transaction_db(&pb);
        assert_eq!(db.transaction_hash.len(), 66);
        assert_eq!(db.amount.len(), 66);
        assert!(db.sender_address.is_some());
        assert!(db.sender_signature.is_some());
        assert!(db.source_transaction_hash.is_some());
    }

    #[test]
    fn test_conversion_to_transaction_db_with_missing_fields() {
        let pb = sample_transaction_pb_with_missing_fields();
        let db = to_transaction_db(&pb);
        assert_eq!(db.transaction_hash.len(), 66);
        assert_eq!(db.amount.len(), 66);
        assert!(db.sender_address.is_none());
        assert!(db.sender_signature.is_none());
        assert!(db.source_transaction_hash.is_none());
    }

    #[test]
    fn test_conversion_from_transaction_db() {
        let db = sample_transaction_db();
        let pb = from_transaction_db(&db).unwrap();
        assert_eq!(pb.transaction_hash.len(), 32);
        assert_eq!(pb.amount.len(), 32);
        assert!(!pb.sender_address.is_empty());
        assert!(!pb.sender_signature.is_empty());
        assert!(!pb.source_transaction_hash.is_empty());
    }

    #[test]
    fn test_conversion_from_transaction_db_with_missing_fields() {
        let db = sample_transaction_db_with_missing_fields();
        let pb = from_transaction_db(&db).unwrap();
        assert_eq!(pb.transaction_hash.len(), 32);
        assert_eq!(pb.amount.len(), 32);
        assert!(pb.sender_address.is_empty());
        assert!(pb.sender_signature.is_empty());
        assert!(pb.source_transaction_hash.is_empty());
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_pb = sample_transaction_pb();
        let db = to_transaction_db(&original_pb);
        let converted_pb = from_transaction_db(&db).unwrap();
        assert_eq!(original_pb, converted_pb);
    }

    #[test]
    fn test_round_trip_conversion_with_missing_fields() {
        let original_pb = sample_transaction_pb_with_missing_fields();
        let db = to_transaction_db(&original_pb);
        let converted_pb = from_transaction_db(&db).unwrap();
        assert_eq!(original_pb, converted_pb);
    }

    #[test]
    fn test_invalid_hex_prefix() {
        let mut db = sample_transaction_db();
        db.transaction_hash = "1x".to_owned() + &"AA".repeat(32);
        assert!(from_transaction_db(&db).is_err());
    }

    #[test]
    fn test_invalid_hex_length() {
        let mut db = sample_transaction_db();
        db.transaction_hash = "0xAAA".to_string();
        assert!(from_transaction_db(&db).is_err());
    }

    #[test]
    fn test_invalid_hex_content() {
        let mut db = sample_transaction_db();
        db.transaction_hash =
            "0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ".to_string();
        assert!(from_transaction_db(&db).is_err());
    }
}
