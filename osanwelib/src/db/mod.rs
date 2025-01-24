use crate::keys;
use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex::{decode, encode};
use rand::Rng; // Для генерації випадкового IV
use rusqlite::{Connection, Result};
use sha3::{Digest, Keccak256};
use std::error::Error;
use std::fs;

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

pub fn is_password_set() -> Result<bool> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM properties WHERE property_key = ?1")?;
    let count: i64 = stmt.query_row([OSANWE_KEY], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn is_password_correct(external_key: &[u8]) -> Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_database() {
        let _ = fs::remove_file(DB_PATH);
        let conn = Connection::open(DB_PATH).unwrap();
        create_database(&conn).unwrap();

        // Перевіряємо, що таблиця створена, а не конкретне число записів
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='properties'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(exists, 1); // 1 означає, що таблиця створена
    }

    #[test]
    fn test_check_and_create_database() {
        let _ = fs::remove_file(DB_PATH);
        check_and_create_database().unwrap();
        assert!(fs::metadata(DB_PATH).is_ok());
    }

    #[test]
    fn test_insert_and_get_property() {
        let _ = fs::remove_file(DB_PATH);
        check_and_create_database().unwrap();

        let external_key = b"mysecurekey";
        let key = "username";
        let value = "admin";

        insert_property(key, value, external_key).unwrap();
        let retrieved_value = get_property_by_key(key, external_key).unwrap();

        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_encrypt_decrypt() -> Result<(), Box<dyn Error>> {
        let data = b"Hello, world!";
        let external_key = b"mysecurekey";

        let encrypted = encrypt(data, external_key)?; // Розпаковуємо `Result<String, Box<dyn Error>>`
        let decrypted = decrypt(&encrypted, external_key)?; // Розпаковуємо `Result<Vec<u8>, Box<dyn Error>>`

        assert_eq!(decrypted, data); // Тепер порівнюємо `Vec<u8>` з `&[u8]`

        Ok(())
    }

    #[test]
    fn test_is_password_set() {
        let _ = fs::remove_file(DB_PATH);
        check_and_create_database().unwrap();
        let external_key = b"testpassword";

        // Переконуємося, що пароль НЕ встановлений
        assert!(!is_password_set().unwrap());

        set_password(external_key).unwrap();

        // Переконуємося, що тепер пароль встановлено
        assert!(is_password_set().unwrap_or(false));
    }

    #[test]
    fn test_is_password_correct() {
        let _ = fs::remove_file(DB_PATH);
        check_and_create_database().unwrap();
        let external_key = b"correctpassword";

        set_password(external_key).unwrap();

        // Перевіряємо, чи пароль коректний
        assert!(is_password_correct(external_key).unwrap());

        let wrong_key = b"wrongpassword";

        // Переконуємося, що некоректний пароль повертає `false`, а не панікує
        assert!(!is_password_correct(wrong_key).unwrap());
    }

    #[test]
    fn test_set_password() {
        let _ = fs::remove_file(DB_PATH);
        check_and_create_database().unwrap();
        let external_key = b"newpassword";

        set_password(external_key).unwrap();
        assert!(is_password_correct(external_key).unwrap());
    }
}
