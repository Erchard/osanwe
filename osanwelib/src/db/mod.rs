use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex::{decode, encode};
use rusqlite::{Connection, Result};
use sha3::{Digest, Keccak256};
use std::fs;

// AES-256 CBC
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> String {
    let cipher = Aes256Cbc::new_from_slices(key, iv).unwrap();
    let encrypted_data = cipher.encrypt_vec(data);
    encode(encrypted_data)
}

fn decrypt(data: &str, key: &[u8], iv: &[u8]) -> Vec<u8> {
    let cipher = Aes256Cbc::new_from_slices(key, iv).unwrap();
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

pub fn create_database(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut hasher = Keccak256::new();

    // Оновлення хешера з зовнішнім ключем
    hasher.update(external_key);

    let sym_key = hasher.finalize();


    let iv = &sym_key[0..16];

    // Шифрування значення
    let encrypted_value = encrypt(value.as_bytes(), &sym_key, iv);

    // Вставка в базу даних
    conn.execute(
        "INSERT INTO properties (property_key, property_value) VALUES (?1, ?2)",
        (key, encrypted_value),
    )?;

    Ok(())
}

pub fn get_properties(conn: &Connection, external_key: &[u8]) -> Result<Vec<(String, String)>> {
    let mut hasher = Keccak256::new();
    hasher.update(external_key);
    let sym_key = hasher.finalize();
    let iv = &sym_key[0..16];

    let mut stmt = conn.prepare("SELECT property_key, property_value FROM properties")?;
    let property_iter = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let encrypted_value: String = row.get(1)?;
        let decrypted_value = String::from_utf8(decrypt(&encrypted_value, &sym_key, iv)).unwrap();
        Ok((key, decrypted_value))
    })?;

    let mut properties = Vec::new();
    for property in property_iter {
        properties.push(property?);
    }

    Ok(properties)
}

pub fn get_property_by_key(conn: &Connection, key: &str, external_key: &[u8]) -> Result<String> {
    let mut hasher = Keccak256::new();

    // Оновлення хешера з зовнішнім ключем
    hasher.update(external_key);

    let sym_key = hasher.finalize();
    let iv = &sym_key[0..16];

    // Підготовка запиту для отримання властивості
    let mut stmt = conn.prepare("SELECT property_value FROM properties WHERE property_key = ?1")?;
    let encrypted_value: String = stmt.query_row([key], |row| row.get(0))?;

    // Розшифрування значення
    let decrypted_value = String::from_utf8(decrypt(&encrypted_value, &sym_key, iv)).unwrap();

    Ok(decrypted_value)
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

            let mut hasher = Keccak256::new();
            hasher.update(external_key);
            let sym_key = hasher.finalize(); // 32 байти
            let iv = &sym_key[0..16];
            
            let expected_encrypted_value = encrypt(value.as_bytes(), &sym_key, iv);
            

        assert_eq!(
            encrypted_value, expected_encrypted_value,
            "Encrypted value does not match expected value"
        );

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_get_properties() {
        let db_path = "test_get_database.db";
        let _ = fs::remove_file(db_path);
        let conn = Connection::open(db_path).unwrap();
        create_database(&conn).unwrap();

        let external_key = b"0123456789abcdef"; // Зовнішній ключ для шифрування
        insert_property(&conn, "key1", "value1", external_key).unwrap();
        insert_property(&conn, "key2", "value2", external_key).unwrap();

        let properties = get_properties(&conn, external_key).unwrap(); // Додано external_key
        assert_eq!(properties.len(), 2); // Має бути 2 записи
        assert_eq!(properties[0].0, "key1");
        assert_eq!(properties[1].0, "key2");

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_encrypt() {
        let data = b"test data";
        let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes for AES-256
        let iv = b"0123456789abcdef"; // 16 bytes for AES block size

        let encrypted = encrypt(data, key, iv);

        // Переконайтеся, що зашифровані дані не є такими ж, як вихідні
        assert_ne!(encrypted, encode(data));
    }

    #[test]
    fn test_decrypt() {
        let data = b"test data";
        let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes for AES-256
        let iv = b"0123456789abcdef"; // 16 bytes for AES block size

        let encrypted = encrypt(data, key, iv);
        let decrypted = decrypt(&encrypted, key, iv);

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
}
