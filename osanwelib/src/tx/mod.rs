use crate::generated::TransactionPb;
use crate::{db, grpc_client, keys};
use ethers::{
    types::U256,
    utils::{format_units, hex as ethers_hex, keccak256, parse_units},
};
use hex::decode;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Converts a byte slice to a hex string with "0x" prefix
fn to_hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Структура, підготовлена до збереження в БД

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] // Added PartialEq and Eq for testing
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

/// Функція, яка конвертує TransactionDb у JSON-рядок.
pub fn tx_to_json(tx: &TransactionDb) -> Result<String, Box<dyn Error>> {
    let json = serde_json::to_string(tx)?;
    Ok(json)
}

/// Функція, яка конвертує JSON-рядок у `TransactionDb`.
pub fn json_to_tx(json_str: &str) -> Result<TransactionDb, Box<dyn std::error::Error>> {
    let tx_db: TransactionDb = serde_json::from_str(json_str)?;
    Ok(tx_db)
}

/// Функція, яка конвертує JSON-рядок у `TransactionDb`.
pub fn json_to_txpb(json_str: &str) -> Result<TransactionPb, Box<dyn std::error::Error>> {
    let tx_db: TransactionDb = serde_json::from_str(json_str)?;
    let tx_pb = from_transaction_db(&tx_db)?;
    Ok(tx_pb)
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
    verify_transaction(tx)?;

    let tx_db = to_transaction_db(tx);

    store_transaction_db(&tx_db)?;
    Ok(())
}

pub fn store_transaction_db(tx_db: &TransactionDb) -> Result<(), Box<dyn Error>> {
    // Перевіряємо, чи транзакція вже існує
    if db::get_transaction_by_hash(&tx_db.transaction_hash).is_ok() {
        println!("Transaction already exists. Skipping insertion.");
        return Ok(()); // Якщо запис вже є, ігноруємо подальші дії
    }

    // Зберігаємо транзакцію в базі даних
    db::save_transaction(&tx_db)?;

    // Конвертуємо TransactionDb у TransactionPb (для відправки)
    let transaction_pb = from_transaction_db(&tx_db)?;

    // Виводимо повідомлення перед відправкою
    println!("Sending transaction to server, please wait...");

    // Створюємо власний runtime для синхронного виконання асинхронного коду
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        match grpc_client::send_transaction_to_server(transaction_pb).await {
            Ok(_) => println!("Transaction sent successfully."),
            Err(e) => eprintln!("Error sending transaction to server: {:?}", e),
        }
    });

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
    match db::get_transaction_by_hash(transaction_hash) {
        Ok(tx_db) => Ok(from_transaction_db(&tx_db)?),
        Err(err) if err.to_string().contains("QueryReturnedNoRows") => {
            Err(format!("Transaction with hash {} not found", transaction_hash).into())
        }
        Err(err) => Err(err.into()),
    }
}

/// Конвертує суму з рядка у 32-байтовий шістнадцятковий рядок.
///
/// # Аргументи
///
/// * `amount_str` - Сума у вигляді рядка, наприклад, "345.5".
///
/// # Повертає
///
/// * `Ok(String)` - 32-байтовий шістнадцятковий рядок з префіксом "0x".
/// * `Err(Box<dyn Error>)` - Якщо виникла помилка під час конвертації.
///
/// # Приклад
///
/// ```
/// use osanwelib::tx::convert_amount_to_hex;
///
/// let hex = convert_amount_to_hex("0.000000000000000023")?;
/// assert_eq!(hex, "0x0000000000000000000000000000000000000000000000000000000000000017");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn convert_amount_to_hex(amount_str: &str) -> Result<String, Box<dyn Error>> {
    let bytes = convert_amount_to_bytes(amount_str)?; // Використовуємо `?` замість `unwrap()`

    // Перетворення байтів у шістнадцятковий рядок з префіксом "0x"
    let hex_str = format!("0x{}", ethers_hex::encode(bytes));

    Ok(hex_str)
}

pub fn convert_amount_to_bytes(amount_str: &str) -> Result<[u8; 32], Box<dyn Error>> {
    let decimals = 18; // Кількість десяткових знаків для ETH. Змінюйте за потреби.

    // Перетворення рядка у U256 (wei) використовуючи ethers::utils::parse_units
    let amount_wei: U256 = parse_units(amount_str, decimals)
        .map_err(|e| Box::<dyn Error>::from(e))?
        .into(); // Explicitly convert the error

    // Перетворення U256 у 32-байтовий масив (big-endian)
    let mut bytes = [0u8; 32];
    amount_wei.to_big_endian(&mut bytes);
    Ok(bytes)
}

pub fn replenishing(
    recipient_address: &str,
    amount_str: &str,
    currency_id: u32,
    source_transaction: &str,
) -> Result<TransactionPb, Box<dyn Error>> {
    let recipient_bytes = decode(&recipient_address[2..])?;
    let amount_bytes = convert_amount_to_bytes(amount_str)?;
    let source_transaction_hash = decode(&source_transaction[2..])?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

    let mut transaction = TransactionPb {
        transaction_hash: Vec::new(), // Порожнє
        transaction_type: 1,          // За замовчуванням
        currency_id,
        amount: amount_bytes.to_vec(),
        timestamp,
        sender_address: Vec::new(),
        sender_output_index: 0,
        recipient_address: recipient_bytes,
        sender_signature: Vec::new(), // Порожнє
        source_transaction_hash,
    };

    let data = tx_to_bytes(&transaction);
    println!("data={:?}", data);

    let transaction_hash = keccak256(&data);

    transaction.transaction_hash = transaction_hash.to_vec();
    println!("{:?}", transaction);
    Ok(transaction)
}

pub fn send_money(
    external_key: &str,
    amount_str: &str,
    currency_id: u32,
    recipient: &str,
) -> Result<TransactionPb, Box<dyn Error>> {
    // 1. Отримуємо адресу відправника зі сховища ключів
    let sender_address_str = keys::get_wallet_address(external_key.as_bytes())?;
    let sender_address = decode(&sender_address_str[2..])?;

    // 2. Зчитуємо поточний баланс гаманця (32 байти в Big-Endian)
    let balance_bytes = db::get_wallet_balance(&sender_address_str)?;
    let big_balance = U256::from_big_endian(&balance_bytes);

    // 3. Парсимо кількість, яку збираємось відправити, у форматі U256 (wei, якщо 18 знаків після коми)
    let amount_wei: U256 = parse_units(amount_str, 18)?.into();

    // 4. Перевіряємо, чи вистачає балансу
    if big_balance < amount_wei {
        // Можна додатково відформатувати баланс у звичних одиницях (наприклад, ETH).
        let balance_formatted = format_units(big_balance, 18)?;
        return Err(format!(
            "Insufficient funds. Your wallet has {} (wei), which is less than the requested amount {}",
            balance_formatted,
            amount_str
        )
        .into());
    }

    // 5. Якщо коштів достатньо, конвертуємо amount_str у 32-байтове представлення:
    let amount_bytes = convert_amount_to_bytes(amount_str)?;

    let recipient_bytes = decode(&recipient[2..])?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

    // Приклад: отримати наступний відправний індекс (UTXO-подібний лічильник) з БД
    let sender_output_index = db::get_next_sender_output_index(&sender_address_str)?;

    // 6. Формуємо транзакцію
    let mut transaction = TransactionPb {
        transaction_hash: Vec::new(),
        transaction_type: 2,
        currency_id,
        amount: amount_bytes.to_vec(),
        timestamp,
        sender_address,
        sender_output_index,
        recipient_address: recipient_bytes,
        sender_signature: Vec::new(),
        source_transaction_hash: Vec::new(),
    };

    // 7. Рахуємо хеш транзакції (без підпису, тому що підпис йде поверх)
    let data = tx_to_bytes(&transaction);
    let transaction_hash = keccak256(&data);
    transaction.transaction_hash = transaction_hash.to_vec();

    // 8. Підписуємо байти транзакції
    let sender_signature = keys::sign_byte_array_sync(data, external_key.as_bytes())?;
    transaction.sender_signature = sender_signature;

    // Повертаємо готову транзакцію
    Ok(transaction)
}

/// Перетворює TransactionPb у вектор байтів, який містить наступні поля:
/// - transaction_type (u32, 4 байти, big-endian)
/// - currency_id (u32, 4 байти, big-endian)
/// - amount (масив байтів, наприклад, 32 байти)
/// - timestamp (u64, 8 байтів, big-endian)
/// - sender_address (масив байтів, наприклад, 20 байтів)
/// - sender_output_index (u32, 4 байти, big-endian)
/// - recipient_address (масив байтів, наприклад, 20 байтів)
pub fn tx_to_bytes(tx: &TransactionPb) -> Vec<u8> {
    let mut buffer = Vec::new();

    if tx.transaction_type == 1 {
        // Серіалізація для транзакції типу 1
        buffer.extend_from_slice(&tx.transaction_type.to_be_bytes());
        buffer.extend_from_slice(&tx.currency_id.to_be_bytes());
        buffer.extend_from_slice(&tx.amount);
        buffer.extend_from_slice(&tx.timestamp.to_be_bytes());
        buffer.extend_from_slice(&tx.recipient_address);
        buffer.extend_from_slice(&tx.source_transaction_hash);
    } else if tx.transaction_type == 2 {
        // Серіалізація для транзакції типу 2 (на даному етапі така ж, як для типу 1)
        buffer.extend_from_slice(&tx.transaction_type.to_be_bytes());
        buffer.extend_from_slice(&tx.currency_id.to_be_bytes());
        buffer.extend_from_slice(&tx.amount);
        buffer.extend_from_slice(&tx.timestamp.to_be_bytes());
        buffer.extend_from_slice(&tx.sender_address);
        buffer.extend_from_slice(&tx.sender_output_index.to_be_bytes());
        buffer.extend_from_slice(&tx.recipient_address);
    } else {
        // Обробка випадку, коли тип транзакції не розпізнано
        // Можна повертати помилку або виконувати якусь дефолтну логіку
        // Наприклад, для простоти повернемо пустий вектор:
        return Vec::new();
    }
    buffer
}

/// Перевіряє цілісність транзакції:
/// 1. Хеш `transaction_hash` має збігатись із `keccak256(tx_to_bytes(tx))`.
/// 2. Якщо тип транзакції = 2 (наприклад, надсилання коштів),
///    підпис (`sender_signature`) має бути валідною і належати `sender_address`.
pub fn verify_transaction(tx: &TransactionPb) -> Result<(), Box<dyn Error>> {
    // 1. Формуємо байтове подання транзакції (без підпису).
    let data = tx_to_bytes(tx);

    // 2. Перевіряємо, що хеш збігається з `transaction_hash`.
    let computed_hash = keccak256(&data);
    if computed_hash[..] != tx.transaction_hash[..] {
        return Err("Invalid transaction hash: does not match keccak256(tx_to_bytes)".into());
    }

    // 3. Для транзакцій, які вимагають підпису (наприклад, type=2), перевіряємо підпис:
    if tx.transaction_type == 2 {
        // a) Переконуємось, що поле підпису не порожнє
        if tx.sender_signature.is_empty() {
            return Err("Missing sender signature for transaction type 2".into());
        }

        // b) Відновлюємо адресу підписанта (recover) за допомогою наявного коду в keys (псевдо-приклад)
        // Припустімо, що у вас є `keys::recover_signer_sync(data, signature) -> Result<Vec<u8>, Box<dyn Error>>`
        // яка повертає адресу (20 байтів).
        let recovered_address = keys::recover_signer_sync(&data, &tx.sender_signature)?;

        // c) Звіряємо з адресою відправника.
        if recovered_address != tx.sender_address {
            return Err(
                "Signature mismatch: recovered address does not match sender_address".into(),
            );
        }
    }

    // Якщо всі перевірки пройшли успішно – повертаємо Ok
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_send_money() {
        let result = send_money(
            "password",
            "100.5",
            1,
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdef",
        );
        println!("{:?}", result);
    }

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
    fn test_convert_amount_to_hex_large_amount() {
        let amount_str = "345.5";
        // 345.5 * 10^18 = 345500000000000000000 wei
        // Convert to hex: 345500000000000000000 = 0x12bac6937669760000
        // Pad with leading zeros to make it 32 bytes (64 hex chars)
        let expected_hex = "0x000000000000000000000000000000000000000000000012bac6937669760000";
        let result = convert_amount_to_hex(amount_str).unwrap();

        // Assert lengths first
        assert_eq!(
            result.len(),
            expected_hex.len(),
            "Hex strings have different lengths"
        );

        // Compare byte-by-byte
        assert_eq!(
            result.as_bytes(),
            expected_hex.as_bytes(),
            "Hex strings differ at byte level"
        );
    }

    #[test]
    fn test_convert_amount_to_hex_large_amount_numeric() {
        let amount_str = "345.5";
        let expected_decimal = U256::from_dec_str("345500000000000000000").unwrap();

        let result_hex = convert_amount_to_hex(amount_str).unwrap();
        let result_bytes = hex::decode(&result_hex[2..]).unwrap();
        let result_decimal = U256::from_big_endian(&result_bytes);

        assert_eq!(
            result_decimal, expected_decimal,
            "Numeric values do not match"
        );
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
