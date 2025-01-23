use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex::{decode, encode};
use rusqlite::{Connection, Result};
use sha3::{Digest, Keccak256};
use std::fs;
use crate::keys;

// AES-256 CBC
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub const OSANWE_KEY: &str = "osanwe";
pub const TEST_PHRASE: &str = "interchange of thought";
pub const PRIV_KEY: &str = "priv_key";
pub const WALLET: &str = "wallet";

/// Допоміжна функція, що створює cipher (AES-256 CBC) із `external_key`.
fn create_cipher(external_key: &[u8]) -> Aes256Cbc {
    let mut hasher = Keccak256::new();
    hasher.update(external_key);

    // 32-байтовий ключ
    let sym_key = hasher.finalize();
    // Перші 16 байтів від симетричного ключа — IV
    let iv = &sym_key[0..16];

    Aes256Cbc::new_from_slices(&sym_key, iv).unwrap()
}

/// Зашифрувати байти `data`, використовуючи хеш від `external_key`.
fn encrypt(data: &[u8], external_key: &[u8]) -> String {
    let cipher = create_cipher(external_key);
    let encrypted_data = cipher.encrypt_vec(data);
    encode(encrypted_data)
}

/// Розшифрувати рядок (hex) `data`, використовуючи хеш від `external_key`.
fn decrypt(data: &str, external_key: &[u8]) -> Vec<u8> {
    let cipher = create_cipher(external_key);
    let encrypted_data = decode(data).unwrap();
    cipher.decrypt_vec(&encrypted_data).unwrap()
}

pub fn check_and_create_database(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if fs::metadata(db_path).is_ok() {
        println!("Database found, all is well.");
    } else {
        let conn = Connection::open(db_path)?;
        create_database(&conn)?;
        println!("Database created successfully.");
    }
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

pub fn insert_property(
    conn: &Connection,
    key: &str,
    value: &str,
    external_key: &[u8],
) -> Result<()> {
    // Шифрування значення
    let encrypted_value = encrypt(value.as_bytes(), external_key);

    // Вставка в базу даних
    conn.execute(
        "INSERT OR REPLACE INTO properties (property_key, property_value)
        VALUES (?1, ?2)",
        (key, encrypted_value),
    )?;

    Ok(())
}

pub fn get_property_by_key(conn: &Connection, key: &str, external_key: &[u8]) -> Result<String> {
    // Підготовка запиту для отримання властивості
    let mut stmt = conn.prepare("SELECT property_value FROM properties WHERE property_key = ?1")?;
    let encrypted_value: String = stmt.query_row([key], |row| row.get(0))?;

    // Розшифрування значення
    let decrypted_value = String::from_utf8(decrypt(&encrypted_value, external_key)).unwrap();

    Ok(decrypted_value)
}

pub fn is_password_set(db_path: &str) -> Result<bool> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM properties WHERE property_key = ?1")?;
    let count: i64 = stmt.query_row([OSANWE_KEY], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn is_password_correct(conn: &Connection, external_key: &[u8]) -> Result<bool> {
    let test_phrase = get_property_by_key(conn, OSANWE_KEY, external_key).unwrap();
    Ok(test_phrase == TEST_PHRASE)
}

pub fn set_password(db_path: &str, external_key:&[u8]) -> Result<()> {
    let conn = Connection::open(db_path)?;
    create_database(&conn)?; // Переконуємося, що таблиця існує
    insert_property(&conn, OSANWE_KEY, TEST_PHRASE, external_key)?;
    println!("Password has been successfully set.");
    let address = keys::generate_save_keypair(db_path, external_key);
    println!("Generated Ethereum Address: {:?}", address);
    Ok(())
}

pub fn save_keypair(
    db_path: &str,
    signing_key: &str,
    address: &str,
    external_key: &[u8],
) -> Result<()> {
    let conn = Connection::open(db_path)?;
    insert_property(&conn, PRIV_KEY, signing_key, external_key)?;
    insert_property(&conn, WALLET, address, external_key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_database() {
        let db_path = "test_database.db";

        // Видалити базу даних, якщо вона існує
        let _ = fs::remove_file(db_path);

        // Виклик функції для створення бази даних
        let conn = Connection::open(db_path).unwrap();
        create_database(&conn).unwrap();

        // Перевірка, чи таблиця була створена
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM properties", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0); // Таблиця повинна бути порожньою

        // Очистити тестову базу даних
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_check_and_create_database() {
        let db_path = "test_check_database.db";

        // Видалити базу даних, якщо вона існує
        let _ = fs::remove_file(db_path);

        // Перевірка, чи база даних створюється
        check_and_create_database(db_path).unwrap();
        assert!(fs::metadata(db_path).is_ok()); // База даних повинна існувати

        // Виклик функції ще раз, щоб перевірити, що вона не створює нову базу даних
        check_and_create_database(db_path).unwrap();

        // Очистити тестову базу даних
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_insert_property() {
        let db_path = "test_insert_database.db";
        let _ = fs::remove_file(db_path);
        let conn = Connection::open(db_path).unwrap();
        create_database(&conn).unwrap();

        let external_key = b"0123456789abcdef"; // Зовнішній ключ для шифрування
        let key = "key1";
        let value = "value1";

        // Вставка властивості в базу даних
        let result = insert_property(&conn, key, value, external_key);
        assert!(
            result.is_ok(),
            "Failed to insert property: {:?}",
            result.err()
        );

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM properties", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1); // Має бути 1 запис

        let encrypted_value: String = conn
            .query_row(
                "SELECT property_value FROM properties WHERE property_key = 'key1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let expected_encrypted_value = encrypt(value.as_bytes(), external_key);

        assert_eq!(
            encrypted_value, expected_encrypted_value,
            "Encrypted value does not match expected value"
        );

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_encrypt() {
        let data = b"test data";
        let external_key = b"0123456789abcdef";

        let encrypted = encrypt(data, external_key);

        // Переконайтеся, що зашифровані дані не є такими ж, як вихідні
        assert_ne!(encrypted, encode(data));
    }

    #[test]
    fn test_decrypt() {
        let data = b"test data";
        let external_key = b"0123456789abcdef";

        let encrypted = encrypt(data, external_key);
        let decrypted = decrypt(&encrypted, external_key);

        // Переконайтеся, що розшифровані дані відповідають вихідним
        assert_eq!(decrypted, data.to_vec());
    }

    #[test]
    fn test_get_property_by_key() {
        let db_path = "test_get_property_database.db";
        let _ = fs::remove_file(db_path);
        let conn = Connection::open(db_path).unwrap();
        create_database(&conn).unwrap();

        let external_key = b"0123456789abcdef"; // Зовнішній ключ для шифрування
        let key = "property_key";
        let value = "property_value";

        // Вставка властивості в базу даних
        insert_property(&conn, key, value, external_key).unwrap();

        // Отримання властивості за ключем
        let retrieved_value = get_property_by_key(&conn, key, external_key).unwrap();

        // Перевірка, чи отримане значення відповідає оригінальному
        assert_eq!(retrieved_value, value);

        // Очистка тестової бази даних
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_symmetric_key_length() {
        let external_key = b"0123456789abcdef"; // Зовнішній ключ для шифрування
        let mut hasher = Keccak256::new();

        // Оновлення хешера з зовнішнім ключем
        hasher.update(external_key);

        let sym_key = hasher.finalize();

        // Перевірка довжини симетричного ключа
        assert!(
            sym_key.len() >= 32,
            "Symmetric key is too short: {}",
            sym_key.len()
        );
    }

    #[test]
    fn test_is_password_correct() {
        let db_path = "test_is_password_correct.db";

        // Видаляємо файл БД перед тестом, щоб не було старих даних
        let _ = fs::remove_file(db_path);

        let conn = Connection::open(db_path).unwrap();
        create_database(&conn).unwrap();

        let external_key = b"0123456789abcdef";

        // 1. Вставляємо правильну фразу (TEST_PHRASE) для ключа OSANWE_KEY
        insert_property(&conn, OSANWE_KEY, TEST_PHRASE, external_key).unwrap();

        // 2. Тепер перевіряємо, чи пароль правильний
        let result_correct = is_password_correct(&conn, external_key).unwrap();
        assert!(
            result_correct,
            "Expected password to be correct (true), but got false"
        );

        // 3. Оновлюємо існуючий запис, щоб він був неправильним

        insert_property(&conn, OSANWE_KEY, "wrong phrase", external_key).unwrap();

        // 4. Тепер має повертати false
        let result_incorrect = is_password_correct(&conn, external_key).unwrap();
        assert!(
            !result_incorrect,
            "Expected password to be incorrect (false), but got true"
        );

        // Прибираємо за собою
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_set_password() {
        let db_path = "test_set_password.db";
        let _ = fs::remove_file(db_path);
        let external_key = "mypassword".as_bytes();

        // Установка паролю
        set_password(db_path, external_key).unwrap();

        // Перевірка, що пароль коректно встановлено
        let conn = Connection::open(db_path).unwrap();
        let is_correct = is_password_correct(&conn, external_key).unwrap();
        assert!(is_correct, "Password should be correct");
    }

    #[test]
    fn test_set_password_persistence() {
        let db_path = "test_set_password_persistence.db";
        let _ = fs::remove_file(db_path);
        let external_key = "persistpassword".as_bytes();

        // Встановлення пароля
        set_password(db_path, external_key).unwrap();

        // Закриваємо з'єднання та відкриваємо нове
        let conn = Connection::open(db_path).unwrap();
        assert!(
            is_password_correct(&conn, external_key).unwrap(),
            "Password should persist after reopening DB"
        );
    }
}
