use rusqlite::{Connection, Result};
use std::fs;
use sha3::{Digest, Keccak256};
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use hex::{encode, decode};

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

pub fn insert_property(conn: &Connection, key: &str, value: &str) -> Result<()> {
    let mut hasher = Keccak256::new();
    hasher.update(b"some_secure_data");
    let sym_key = hasher.finalize();
    let iv = &sym_key[0..16];

    let encrypted_value = encrypt(value.as_bytes(), &sym_key, iv);

    conn.execute(
        "INSERT INTO properties (property_key, property_value) VALUES (?1, ?2)",
        (key, encrypted_value),
    )?;
    Ok(())
}

pub fn get_properties(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut hasher = Keccak256::new();
    hasher.update(b"some_secure_data");
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

