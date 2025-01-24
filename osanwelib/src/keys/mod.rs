use crate::db;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use hex::encode;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::keccak256;

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
