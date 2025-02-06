use crate::db;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::utils::keccak256;
use hex::{decode, encode};
use rand::thread_rng;
use std::error::Error;

pub const PRIV_KEY: &str = "priv_key";
pub const WALLET: &str = "wallet";

/// Генерує нову пару ключів Ethereum (приватний і публічний).
pub fn generate_ethereum_keypair() -> (SigningKey, Address) {
    let signing_key = SigningKey::random(&mut thread_rng()); // Генеруємо випадковий закритий ключ
    let wallet = LocalWallet::from(signing_key.clone()); // Створюємо гаманець з цього ключа
    let address = wallet.address(); // Отримуємо Ethereum-адресу

    println!("Private Key: {}", hex::encode(signing_key.to_bytes()));
    println!("Ethereum Address: {:?}", address);

    (signing_key, address)
}

pub fn generate_save_keypair(external_key: &[u8]) -> Result<Address, Box<dyn std::error::Error>> {
    // Generate the Ethereum keypair
    let (signing_key, address) = generate_ethereum_keypair();

    // Convert SigningKey to a hex string
    let signing_key_hex = encode(signing_key.to_bytes());

    // Convert Address to a string
    let address_str = format!("{:?}", address); // Alternatively, use address.to_string() if available

    // Save the keypair to the database
    save_keypair(&signing_key_hex, &address_str, external_key)?;

    Ok(address)
}

pub fn save_keypair(
    signing_key: &str,
    address: &str,
    external_key: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    db::insert_property(PRIV_KEY, signing_key, external_key)?;
    db::insert_property(WALLET, address, external_key)?;
    Ok(())
}

pub fn get_wallet_address(external_key: &[u8]) -> Result<String, Box<dyn Error>> {
    db::get_property_by_key(WALLET, external_key)
}

pub fn sign_byte_array_sync(data: Vec<u8>, external_key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // Отримуємо збережений приватний ключ з бази даних
    let priv_key_hex = db::get_property_by_key(PRIV_KEY, external_key)?;
    let priv_key_bytes = decode(priv_key_hex)?;

    // Перетворюємо Vec<u8> у [u8; 32]
    let priv_key_array: [u8; 32] = priv_key_bytes
        .try_into()
        .map_err(|_| "Invalid private key length")?;

    // Створюємо гаманець із приватного ключа
    let wallet = LocalWallet::from(SigningKey::from_bytes((&priv_key_array).into())?);

    // Хешуємо дані за допомогою Keccak-256 (Ethereum-стандарт)
    let digest = keccak256(&data);
    let hash = H256::from_slice(&digest);

    // Синхронно підписуємо хешовані дані
    let signature = wallet.sign_hash(hash)?;

    // Повертаємо підпис у вигляді байтового масиву
    Ok(signature.to_vec())
}

/// Відновлює (recover) адресу підписанта з байтів повідомлення (`data`) і байтів підпису (`signature`).
pub fn recover_signer_sync(data: &[u8], signature: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // 1. Хешуємо вхідні дані (EVM-стиль, Keccak-256)
    let digest = keccak256(data);
    let hash = H256::from_slice(&digest);

    // 2. Конвертуємо 65-байтовий підпис (r, s, v) у тип `ethers::types::Signature`
    let signature = Signature::try_from(signature)
        .map_err(|_| "Invalid signature length or format. Expected 65 bytes (r,s,v)".to_string())?;

    // 3. Відновлюємо адресу, яка підписала хеш
    let recovered_address = signature.recover(hash)?;

    // 4. Повертаємо 20 байтів адреси у `Vec<u8>` (для порівняння з sender_address)
    Ok(recovered_address.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::keccak256;

    #[test]
    fn test_sign_byte_array() {
        let external_key = b"test_key";
        let data = b"Hello, Osanwe!".to_vec();

        // Generate keys and store in the DB for testing
        let (signing_key, _address) = generate_ethereum_keypair();
        let priv_key_hex = hex::encode(signing_key.to_bytes());
        db::insert_property(PRIV_KEY, &priv_key_hex, external_key).unwrap();

        // Await the signature future
        let signature = sign_byte_array_sync(data.clone(), external_key).unwrap();
        assert!(!signature.is_empty(), "Підпис не повинен бути порожнім");
    }

    #[test]
    fn test_generate_ethereum_keypair() {
        // Генеруємо пару ключів
        let (private_key, address) = generate_ethereum_keypair();

        // Переконуємося, що приватний ключ не порожній
        let private_key_bytes = private_key.to_bytes();
        assert_eq!(
            private_key_bytes.len(),
            32,
            "Приватний ключ має бути 32 байти"
        );

        // Отримуємо відкритий ключ у вигляді стисленого SEC1
        let public_key = private_key.verifying_key();
        let uncompressed_public_key = public_key.to_encoded_point(false);
        let pub_key_bytes = &uncompressed_public_key.as_bytes()[1..]; // Видаляємо префікс 0x04

        // Перевіряємо, що відкритий ключ має правильний розмір
        assert_eq!(pub_key_bytes.len(), 64, "Публічний ключ має бути 64 байти");

        // Генеруємо Ethereum-адресу вручну через Keccak-256
        let derived_address = Address::from_slice(&keccak256(pub_key_bytes)[12..]);

        // Перевіряємо, що отримана адреса відповідає очікуваній
        assert_eq!(address, derived_address, "Згенерована адреса некоректна");
    }
}
