use crate::keys;
use crate::tx::TransactionDb;
use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex::{decode, encode};
use rand::Rng; // Для генерації випадкового IV
use rusqlite::{params, Connection, Result as SqlResult};
use sha3::{Digest, Keccak256};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

// AES-256 CBC
type Aes256Cbc = Cbc<Aes256, Pkcs7>;
pub const DB_PATH: &str = "osanwe.db";
pub const OSANWE_KEY: &str = "osanwe";
pub const TEST_PHRASE: &str = "interchange of thought";

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

/// Saves a `TransactionDb` record into the 'transactions' table.
/// Ensures the table exists before attempting to insert.
pub fn save_transaction(tx_db: &TransactionDb) -> Result<(), Box<dyn Error>> {
    // Ensure the 'transactions' table exists
    ensure_transactions_table_exists()?;

    // Open a connection to the database
    let conn = Connection::open(DB_PATH)?;

    // Prepare the SQL statement for insertion
    let mut stmt = conn.prepare(
        "INSERT INTO transactions (
            transaction_hash,
            transaction_type,
            currency_id,
            amount,
            decimal,
            timestamp,
            sender_address,
            sender_output_index,
            recipient_address,
            sender_signature,
            source_transaction_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;

    // Execute the insertion with the provided `TransactionDb` data
    stmt.execute(params![
        tx_db.transaction_hash,
        tx_db.transaction_type,
        tx_db.currency_id,
        tx_db.amount,
        tx_db.decimal,
        tx_db.timestamp as i64, // Ensure correct type for INTEGER
        tx_db.sender_address,
        tx_db.sender_output_index as i64, // Ensure correct type for INTEGER
        tx_db.recipient_address,
        tx_db.sender_signature,
        tx_db.source_transaction_hash,
    ])?;

    println!("Transaction saved successfully.");

    Ok(())
}



/// Отримує транзакцію з бази даних за її хешем.
/// 
/// # Аргументи
/// 
/// * `transaction_hash` - Хеш транзакції, яку потрібно знайти.
/// 
/// # Повертає
/// 
/// * `Ok(TransactionDb)` - Якщо транзакція знайдена та успішно десеріалізована.
/// * `Err(Box<dyn Error>)` - Якщо сталася помилка під час виконання запиту або десеріалізації.
/// 
pub fn get_transaction_by_hash(transaction_hash: &str) -> Result<TransactionDb, Box<dyn Error>> {
    // Відкриваємо з'єднання з базою даних
    let conn = Connection::open(DB_PATH)?;

    // Підготовлюємо SQL-запит для вибірки транзакції за хешем
    let mut stmt = conn.prepare(
        "SELECT 
            transaction_hash,
            transaction_type,
            currency_id,
            amount,
            decimal,
            timestamp,
            sender_address,
            sender_output_index,
            recipient_address,
            sender_signature,
            source_transaction_hash
         FROM transactions 
         WHERE transaction_hash = ?1",
    )?;

    // Виконуємо запит і отримуємо результат
    let tx_db = stmt.query_row(params![transaction_hash], |row| {
        Ok(TransactionDb {
            transaction_hash: row.get(0)?,
            transaction_type: row.get(1)?,
            currency_id: row.get(2)?,
            amount: row.get(3)?,
            decimal: row.get(4)?,
            timestamp: row.get(5)?,
            sender_address: row.get(6)?,
            sender_output_index: row.get(7)?,
            recipient_address: row.get(8)?,
            sender_signature: row.get(9)?,
            source_transaction_hash: row.get(10)?,
        })
    })?;

    Ok(tx_db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Допоміжна функція для чистого старту в кожному тесті
    fn remove_db() {
        let _ = fs::remove_file(DB_PATH);
    }

    #[test]
    fn test_database_initialization() {
        // Переконаємось, що файл БД видалений
        remove_db();
        // Викликаємо “перевірку і створення БД”
        check_and_create_database().unwrap();

        // Тепер файл має існувати
        assert!(fs::metadata(DB_PATH).is_ok(), "DB file was not created");

        // Перевіримо, чи створилася таблиця `properties`
        let conn = Connection::open(DB_PATH).unwrap();
        let properties_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='properties';",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(properties_exists, 1, "Table 'properties' not created");

        // Перевіримо, чи створилася таблиця `CryptoAssets` (якщо це потрібно в одному тесті)
        let crypto_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='CryptoAssets';",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(crypto_exists, 1, "Table 'CryptoAssets' not created");
    }

    #[test]
    fn test_set_and_check_password() {
        remove_db();
        check_and_create_database().unwrap();

        let external_key = b"testpassword";
        // Перед встановленням пароля перевіримо, що він не встановлений
        assert!(!is_password_set().unwrap());

        // Встановлюємо пароль
        set_password(external_key).unwrap();
        assert!(
            is_password_set().unwrap(),
            "`is_password_set()` should be true after setting password"
        );

        // Перевіряємо, що він коректний
        assert!(
            is_password_correct(external_key).unwrap(),
            "Password should be correct"
        );

        // З іншим ключем перевірка має провалитись
        let wrong_key = b"wrongpassword";
        assert!(
            !is_password_correct(wrong_key).unwrap(),
            "Wrong password should return false"
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"Hello, world!";
        let external_key = b"mysecurekey";

        let encrypted = encrypt(data, external_key).expect("Encryption failed");
        let decrypted = decrypt(&encrypted, external_key).expect("Decryption failed");
        assert_eq!(decrypted, data, "Decrypted data does not match original");
    }

    #[test]
    fn test_insert_and_get_property() {
        remove_db();
        check_and_create_database().unwrap();

        let external_key = b"mysecurekey";
        let key = "username";
        let value = "admin";

        insert_property(key, value, external_key).unwrap();
        let retrieved_value = get_property_by_key(key, external_key).unwrap();

        assert_eq!(
            retrieved_value, value,
            "Retrieved value doesn't match inserted value"
        );
    }

    #[test]
    fn test_save_transaction() {
        // Clean up any existing database
        remove_db();

        // Initialize the database (creates tables)
        check_and_create_database().unwrap();

        // Create a sample TransactionDb
        let tx_db = TransactionDb {
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

        // Save the transaction
        save_transaction(&tx_db).unwrap();

        // Verify that the transaction was inserted
        let conn = Connection::open(DB_PATH).unwrap();
        let retrieved_tx: TransactionDb = conn
            .query_row(
                "SELECT 
                transaction_hash,
                transaction_type,
                currency_id,
                amount,
                decimal,
                timestamp,
                sender_address,
                sender_output_index,
                recipient_address,
                sender_signature,
                source_transaction_hash
            FROM transactions WHERE transaction_hash = ?1",
                params![tx_db.transaction_hash],
                |row| {
                    Ok(TransactionDb {
                        transaction_hash: row.get(0)?,
                        transaction_type: row.get(1)?,
                        currency_id: row.get(2)?,
                        amount: row.get(3)?,
                        decimal: row.get(4)?,
                        timestamp: row.get(5)?,
                        sender_address: row.get(6)?,
                        sender_output_index: row.get(7)?,
                        recipient_address: row.get(8)?,
                        sender_signature: row.get(9)?,
                        source_transaction_hash: row.get(10)?,
                    })
                },
            )
            .unwrap();

        // Assert that the retrieved transaction matches the original
        assert_eq!(tx_db.transaction_hash, retrieved_tx.transaction_hash);
        assert_eq!(tx_db.transaction_type, retrieved_tx.transaction_type);
        assert_eq!(tx_db.currency_id, retrieved_tx.currency_id);
        assert_eq!(tx_db.amount, retrieved_tx.amount);
        assert_eq!(tx_db.decimal, retrieved_tx.decimal);
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
    fn test_get_transaction_by_hash() {
        // Очищуємо базу даних перед тестом
        remove_db();
        check_and_create_database().unwrap();

        // Створюємо зразок транзакції
        let tx_db = TransactionDb {
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            transaction_type: 2,
            currency_id: 200,
            amount: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef".to_string(),
            decimal: "0x02".to_string(),
            timestamp: 1700000001,
            sender_address: "0xsenderaddress1234567890abcdef".to_string(),
            sender_output_index: 100,
            recipient_address: "0xrecipientaddressabcdef123456".to_string(),
            sender_signature: "0xsendersignatureabcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            source_transaction_hash: "0xsourcehashabcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        };

        // Зберігаємо транзакцію
        save_transaction(&tx_db).unwrap();

        // Отримуємо транзакцію за хешем
        let retrieved_tx = get_transaction_by_hash(&tx_db.transaction_hash).unwrap();

        // Перевіряємо, чи збігаються дані
        assert_eq!(tx_db.transaction_hash, retrieved_tx.transaction_hash);
        assert_eq!(tx_db.transaction_type, retrieved_tx.transaction_type);
        assert_eq!(tx_db.currency_id, retrieved_tx.currency_id);
        assert_eq!(tx_db.amount, retrieved_tx.amount);
        assert_eq!(tx_db.decimal, retrieved_tx.decimal);
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

}
