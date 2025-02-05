use crate::keys;
use crate::tx::TransactionDb;
use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex::{decode, encode};
use rand::Rng; // Для генерації випадкового IV
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use sha3::{Digest, Keccak256};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use ethers::types::U256;

// AES-256 CBC
type Aes256Cbc = Cbc<Aes256, Pkcs7>;
pub const DB_PATH: &str = "osanwe.db";
pub const OSANWE_KEY: &str = "osanwe";
pub const TEST_PHRASE: &str = "interchange of thought";

#[derive(Debug)]
pub struct CryptoAsset {
    pub id: i32,
    pub net_type: i32,
    pub chain_code: i32,
    pub token_id: i32,
    pub symbol: String,
    pub description: Option<String>,
}

fn create_cipher(external_key: &[u8], iv: Option<&[u8]>) -> (Aes256Cbc, Vec<u8>) {
    let mut hasher = Keccak256::new();
    hasher.update(external_key);
    let sym_key = hasher.finalize();
    let sym_key = sym_key[..32].to_vec(); // Явно беремо 32 байти для AES-256

    let iv = match iv {
        Some(iv) => iv.to_vec(), // Використовуємо переданий IV
        None => {
            let mut new_iv = [0u8; 16];
            rand::thread_rng().fill(&mut new_iv);
            new_iv.to_vec()
        }
    };

    let cipher = Aes256Cbc::new_from_slices(&sym_key, &iv).unwrap();
    (cipher, iv)
}

fn encrypt(data: &[u8], external_key: &[u8]) -> Result<String, Box<dyn Error>> {
    let (cipher, iv) = create_cipher(external_key, None);
    let mut encrypted_data = cipher.encrypt_vec(data);

    let mut full_data = iv;
    full_data.append(&mut encrypted_data);

    Ok(encode(full_data))
}

fn decrypt(data: &str, external_key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let encrypted_data = decode(data).map_err(|e| format!("Hex decode error: {}", e))?;
    if encrypted_data.len() < 16 {
        return Err("Невірний формат даних".into());
    }

    let iv = &encrypted_data[0..16]; // Витягуємо IV
    let ciphertext = &encrypted_data[16..]; // Витягуємо шифротекст

    let (cipher, _) = create_cipher(external_key, Some(iv)); // Використовуємо той самий IV

    let decrypted_data = cipher
        .decrypt_vec(ciphertext)
        .map_err(|e| format!("Decryption error: {}", e))?;

    Ok(decrypted_data)
}

pub fn check_and_create_database() -> Result<(), Box<dyn std::error::Error>> {
    if fs::metadata(DB_PATH).is_ok() {
        println!("Database found, all is well.");
    } else {
        let conn = Connection::open(DB_PATH)?;
        create_database(&conn)?;
        println!("Database created successfully.");
    }

    create_cryptoassets_table_if_needed()?; // виконає CREATE TABLE / INSERT з файлу, якщо треба

    Ok(())
}

pub fn create_database(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS properties (
                  property_key   TEXT PRIMARY KEY,
                  property_value TEXT NOT NULL
                  )",
        [],
    )?;
    Ok(())
}

pub fn get_all_cryptoassets() -> Result<Vec<CryptoAsset>, Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;

    let mut stmt = conn.prepare(
        r#"
        SELECT 
            id,
            net_type,
            chain_code,
            token_id,
            symbol,
            description
        FROM CryptoAssets
        ORDER BY symbol ASC, id ASC
        "#,
    )?;

    let assets_iter = stmt.query_map([], |row| {
        Ok(CryptoAsset {
            id: row.get(0)?,
            net_type: row.get(1)?,
            chain_code: row.get(2)?,
            token_id: row.get(3)?,
            symbol: row.get(4)?,
            description: row.get(5)?,
        })
    })?;

    let mut assets = Vec::new();
    for asset_res in assets_iter {
        assets.push(asset_res?);
    }

    Ok(assets)
}

pub fn insert_property(key: &str, value: &str, external_key: &[u8]) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;

    let encrypted_value = encrypt(value.as_bytes(), external_key)?; // Розпакування `Result`

    conn.execute(
        "INSERT OR REPLACE INTO properties (property_key, property_value) VALUES (?1, ?2)",
        (key, encrypted_value),
    )?;

    Ok(())
}

pub fn get_property_by_key(key: &str, external_key: &[u8]) -> Result<String, Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare("SELECT property_value FROM properties WHERE property_key = ?1")?;
    let encrypted_value: String = stmt.query_row([key], |row| row.get(0))?;

    let decrypted_bytes = decrypt(&encrypted_value, external_key)?; // Використовуємо `?`
    let decrypted_value = String::from_utf8(decrypted_bytes)?; // Розпакування перед `String::from_utf8()`

    Ok(decrypted_value)
}

pub fn is_password_set() -> SqlResult<bool> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM properties WHERE property_key = ?1")?;
    let count: i64 = stmt.query_row([OSANWE_KEY], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn is_password_correct(external_key: &[u8]) -> SqlResult<bool> {
    match get_property_by_key(OSANWE_KEY, external_key) {
        Ok(test_phrase) => Ok(test_phrase == TEST_PHRASE),
        Err(e) => {
            eprintln!("Failed to decrypt password: {}", e);
            Ok(false) // Якщо не вдалося розшифрувати, пароль неправильний
        }
    }
}

pub fn set_password(external_key: &[u8]) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;
    create_database(&conn)?;
    match insert_property(OSANWE_KEY, TEST_PHRASE, external_key) {
        Ok(_) => println!("Password has been successfully set."),
        Err(e) => {
            eprintln!("Error setting password in database: {}", e);
            return Err(e);
        }
    }
    let address = keys::generate_save_keypair(external_key);
    println!("Generated Ethereum Address: {:?}", address);
    Ok(())
}

/// Перевіряє, чи існує таблиця CryptoAssets. Якщо ні - зчитує файл SQL та виконує його.
/// Вважаємо, що у файлі CryptoAssets.sql є CREATE TABLE та INSERT-и.
pub fn create_cryptoassets_table_if_needed() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='CryptoAssets';",
        [],
        |row| row.get(0),
    )?;

    // Якщо count == 0 — таблиці ще немає
    if count == 0 {
        // Build the absolute path to CryptoAssets.sql using the crate’s root
        let sql_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("db")
            .join("CryptoAssets.sql");

        // Optionally check if the file actually exists:
        if !sql_path.exists() {
            return Err(format!("SQL file not found at {}", sql_path.display()).into());
        }

        let sql = fs::read_to_string(sql_path)?;
        conn.execute_batch(&sql)?;
        println!("Table 'CryptoAssets' created and data inserted from CryptoAssets.sql");
    } else {
        println!("Table 'CryptoAssets' already exists. No action needed.");
    }

    Ok(())
}

/// Ensures that the 'transactions' table exists in the database.
/// If it does not exist, it creates the table by executing the SQL in 'transactions.sql'.
fn ensure_transactions_table_exists() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;

    // Check if the 'transactions' table exists
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transactions';",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        // Path to the 'transactions.sql' file
        let sql_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("db")
            .join("transactions.sql");

        // Verify that the SQL file exists
        if !sql_path.exists() {
            return Err(format!(
                "SQL file for creating 'transactions' table not found at {}",
                sql_path.display()
            )
            .into());
        }

        // Read and execute the SQL statements from 'transactions.sql'
        let sql = fs::read_to_string(sql_path)?;
        conn.execute_batch(&sql)?;
        println!("Table 'transactions' created successfully.");
    } else {
        println!("Table 'transactions' already exists. No action needed.");
    }

    Ok(())
}

pub fn save_transaction(tx_db: &TransactionDb) -> Result<(), Box<dyn Error>> {
    ensure_transactions_table_exists()?;

    let conn = Connection::open(DB_PATH)?;

    let mut stmt = conn.prepare(
        "INSERT INTO transactions (
            transaction_hash,
            transaction_type,
            currency_id,
            amount,
            timestamp,
            sender_address,
            sender_output_index,
            recipient_address,
            sender_signature,
            source_transaction_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;

    stmt.execute(params![
        &tx_db.transaction_hash,
        tx_db.transaction_type,
        tx_db.currency_id,
        &tx_db.amount,
        tx_db.timestamp as i64,
        &tx_db.sender_address,
        tx_db.sender_output_index.map(|v| v as i64), // NULL якщо `None`
        &tx_db.recipient_address,
        &tx_db.sender_signature,
        &tx_db.source_transaction_hash,
    ])?;

    println!("Transaction saved successfully.");
    Ok(())
}

pub fn get_transaction_by_hash(transaction_hash: &str) -> Result<TransactionDb, Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;

    let mut stmt = conn.prepare(
        "SELECT 
            transaction_hash,
            transaction_type,
            currency_id,
            amount,
            timestamp,
            sender_address,
            sender_output_index,
            recipient_address,
            sender_signature,
            source_transaction_hash
         FROM transactions 
         WHERE transaction_hash = ?1",
    )?;

    let tx_db = stmt.query_row(params![transaction_hash], |row| {
        Ok(TransactionDb {
            transaction_hash: row.get(0)?,
            transaction_type: row.get(1)?,
            currency_id: row.get(2)?,
            amount: row.get(3)?,
            timestamp: row.get(4)?,
            sender_address: row.get::<_, Option<String>>(5)?, // Очікуємо NULL
            sender_output_index: row.get::<_, Option<i64>>(6)?.map(|v| v as u32),
            recipient_address: row.get(7)?,
            sender_signature: row.get::<_, Option<String>>(8)?, // Очікуємо NULL
            source_transaction_hash: row.get::<_, Option<String>>(9)?, // Очікуємо NULL
        })
    })?;

    Ok(tx_db)
}

/// Повертає наступний sender_output_index для заданого sender_address.
/// Якщо записів немає, повертається 0.
pub fn get_next_sender_output_index(sender_address: &str) -> Result<u32, Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;

    // Виконуємо запит для отримання максимального значення sender_output_index для даної адреси.
    let max_index: Option<i64> = conn
        .query_row(
            "SELECT MAX(sender_output_index) FROM transactions WHERE sender_address = ?1",
            params![sender_address],
            |row| row.get(0),
        )
        .optional()?; // Використовуємо optional, оскільки результат може бути NULL

    // Якщо записів немає, повертаємо 0, інакше збільшуємо знайдене значення на 1.
    let next_index = max_index.map(|v| v + 1).unwrap_or(1);

    Ok(next_index as u32)
}


pub fn get_wallet_balance(wallet_address: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_PATH)?;

    // 1) Всі вхідні amount
    let mut stmt = conn.prepare("SELECT amount FROM transactions WHERE recipient_address = ?1")?;
    let incoming_amounts: Vec<String> = stmt
        .query_map(params![wallet_address], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // 2) Всі вихідні amount
    let mut stmt = conn.prepare("SELECT amount FROM transactions WHERE sender_address = ?1")?;
    let outgoing_amounts: Vec<String> = stmt
        .query_map(params![wallet_address], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // Використовуємо U256 із crates `ethers`.
    let mut total = U256::zero();

    // Додаємо все, що прийшло
    for amount_hex in incoming_amounts {
        // Відрізаємо `0x`, якщо воно є
        let s = amount_hex.trim_start_matches("0x");
        let val = U256::from_str_radix(s, 16)?; // Парсимо шістнадцятковий рядок
        total = total.checked_add(val).ok_or("Overflow in addition")?;
    }

    // Віднімаємо все, що пішло
    for amount_hex in outgoing_amounts {
        let s = amount_hex.trim_start_matches("0x");
        let val = U256::from_str_radix(s, 16)?;
        total = total.checked_sub(val).ok_or("Underflow in subtraction")?;
    }

    // Повертаємо 32 байти в Big-Endian (зазвичай), або хочемо зберегти little?
    // U256 має методи для виводу байтів:
    let mut buf = [0u8; 32];
    total.to_big_endian(&mut buf);
    Ok(buf.to_vec())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::TransactionPb;
    use crate::tx::from_transaction_db;
    use crate::tx::to_transaction_db;
    use std::fs;

    /// Допоміжна функція для чистого старту в кожному тесті
    fn remove_db() {
        let _ = fs::remove_file(DB_PATH);
    }

    #[test]
    fn test_conversion_to_transaction_db() {
        let pb = TransactionPb {
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
        };
        let db = to_transaction_db(&pb);

        assert_eq!(db.transaction_hash.len(), 66);
        assert_eq!(db.amount.len(), 66);
        assert_eq!(db.sender_address.as_ref().map(|s| s.len()), Some(42));
        assert_eq!(db.sender_signature.as_ref().map(|s| s.len()), Some(132));
        assert_eq!(
            db.source_transaction_hash.as_ref().map(|s| s.len()),
            Some(66)
        );
    }

    #[test]
    fn test_conversion_to_transaction_db_with_missing_fields() {
        let pb = TransactionPb {
            transaction_hash: vec![0xAA; 32],
            transaction_type: 1,
            currency_id: 100,
            amount: vec![0xBB; 32],
            timestamp: 1700000000,
            sender_address: Vec::new(), // Відсутній відправник
            sender_output_index: 0,
            recipient_address: vec![0xDD; 20],
            sender_signature: Vec::new(),        // Відсутній підпис
            source_transaction_hash: Vec::new(), // Відсутній хеш
        };
        let db = to_transaction_db(&pb);

        assert_eq!(db.transaction_hash.len(), 66);
        assert_eq!(db.amount.len(), 66);
        assert!(db.sender_address.is_none());
        assert!(db.sender_signature.is_none());
        assert!(db.source_transaction_hash.is_none());
    }

    #[test]
    fn test_conversion_from_transaction_db() {
        let db = TransactionDb {
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
        };
        let pb = from_transaction_db(&db).unwrap();

        assert_eq!(pb.transaction_hash.len(), 32);
        assert_eq!(pb.amount.len(), 32);
        assert!(!pb.sender_address.is_empty());
        assert!(!pb.sender_signature.is_empty());
        assert!(!pb.source_transaction_hash.is_empty());
    }

    #[test]
    fn test_conversion_from_transaction_db_with_missing_fields() {
        let db = TransactionDb {
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
        };
        let pb = from_transaction_db(&db).unwrap();

        assert_eq!(pb.transaction_hash.len(), 32);
        assert_eq!(pb.amount.len(), 32);
        assert!(pb.sender_address.is_empty());
        assert!(pb.sender_signature.is_empty());
        assert!(pb.source_transaction_hash.is_empty());
    }

    #[test]
    fn test_save_and_fetch_transaction_with_missing_fields() {
        remove_db();
        check_and_create_database().unwrap();

        let tx_db = TransactionDb {
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            transaction_type: 2,
            currency_id: 200,
            amount: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef"
                .to_string(),
            timestamp: 1700000001,
            sender_address: None,
            sender_output_index: None,
            recipient_address: "0xrecipientaddressabcdef123456".to_string(),
            sender_signature: None,
            source_transaction_hash: None,
        };

        save_transaction(&tx_db).unwrap();

        let retrieved_tx = get_transaction_by_hash(&tx_db.transaction_hash).unwrap();

        assert_eq!(tx_db.transaction_hash, retrieved_tx.transaction_hash);
        assert_eq!(tx_db.transaction_type, retrieved_tx.transaction_type);
        assert_eq!(tx_db.currency_id, retrieved_tx.currency_id);
        assert_eq!(tx_db.amount, retrieved_tx.amount);
        assert_eq!(tx_db.timestamp, retrieved_tx.timestamp);
        assert_eq!(tx_db.sender_address, retrieved_tx.sender_address);
        assert_eq!(tx_db.sender_output_index, retrieved_tx.sender_output_index);
        assert_eq!(tx_db.recipient_address, retrieved_tx.recipient_address);
        assert_eq!(tx_db.sender_signature, retrieved_tx.sender_signature);
        assert_eq!(
            tx_db.source_transaction_hash,
            retrieved_tx.source_transaction_hash
        );
    }

    #[test]
    fn test_get_all_cryptoassets() {
        // Видаляємо, щоб почати з чистої бази
        remove_db();

        // Створюємо базу і таблицю
        check_and_create_database().unwrap();

        // Викликаємо функцію, яка повертає всі активи
        let assets = get_all_cryptoassets().expect("Failed to fetch assets");

        // Переконуємося, що список не порожній
        assert!(!assets.is_empty(), "Assets list should not be empty");

        // Наприклад, перевіримо, що там є щонайменше один запис із символом ETH
        let contains_eth = assets.iter().any(|a| a.symbol == "ETH");
        assert!(contains_eth, "There must be an ETH entry in the list");
    }
}
