use clap::{Arg, Command};
use ethers::{
    types::U256,
    utils::{format_units, hex},
};
use osanwelib::{db, generated::TransactionPb, keys, tx};
use prost::Message;
use rpassword::read_password;
use std::fs::File;
use std::io::{Read, Write};

pub fn get_matches() -> clap::ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Arsen Huzhva <arsenguzhva@gmail.com>")
        .about("Sending cryptocurrencies without commission and blockchain")
        .arg(
            Arg::new("wallet")
                .short('w')
                .long("wallet")
                .help("Displays the wallet address stored in the database")
                .action(clap::ArgAction::SetTrue), // Встановлюємо прапор
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .value_name("PASSWORD")
                .help("Password for accessing the wallet")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("set-password")
                .long("set-password")
                .value_name("PASSWORD")
                .help("Set the wallet password if it hasn't been set yet")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("list-assets")
                .short('l')
                .long("list-assets")
                .help("List all cryptocurrencies in the database")
                .action(clap::ArgAction::SetTrue),
        )
        // Новий прапорець --send, очікуємо три аргументи:
        .arg(
            Arg::new("send")
                .long("send")
                .num_args(3) // Кількість обов'язкових аргументів
                .value_names(["AMOUNT", "CURRENCY_ID", "RECIPIENT"])
                .help("Send tokens to the recipient. Example: --send 345.5 16842752 0x..."),
        )
        .arg(
            Arg::new("replenishing")
                .long("replenishing")
                .num_args(4)
                .value_names([
                    "RECIPIENT_ADDRESS",
                    "AMOUNT",
                    "CURRENCY_ID",
                    "SOURCE_TRANSACTION_HASH"
                ])
                .help("Replenish wallet from external blockchain. Example: --replenishing 0xeb718af7c8f7df1b50eb169be1a85630d3aefe68 2002.736 16842752 0x53004c1174523fb5b3ec8809c36dadf4c9300297a002d160f85c9b5eca73ca89"),
        )
        .arg(
            Arg::new("balance")
                .long("balance")
                .num_args(0..=1)  // 0 або 1 значення
                .value_name("WALLET_ADDRESS")
                .help("Show balance for the given wallet address. If no address is provided, shows your own wallet's balance.")
        )
        .arg(
            Arg::new("import")
                .long("import")
                .num_args(1)
                .value_name("FILE_PATH")
                .help("Import a transaction from an external file in .osnjs format"),
        )
        .get_matches()
}

fn main() {
    let matches = get_matches();

    if let Err(e) = db::check_and_create_database() {
        eprintln!("Error checking or creating database: {:?}", e);
    }

    if let Some(new_password) = matches.get_one::<String>("set-password") {
        if !db::is_password_set() {
            if let Err(e) = db::set_password(new_password.as_bytes()) {
                eprintln!("Error saving password: {:?}", e);
            } else {
                println!("Password has been successfully set.");
            }
        } else {
            println!("Password is already set. Cannot change it.");
        }
    }

    // Використовуємо нову функцію is_password_set, яка повертає bool
    if !db::is_password_set() {
        println!("Password is not set. Please set a new password:");
        if let Some(password) = prompt_for_password() {
            if let Err(e) = db::set_password(password.as_bytes()) {
                eprintln!("Error saving password: {:?}", e);
            } else {
                println!("Password has been successfully set.");
            }
        }
    }

    if matches.get_flag("wallet") {
        let password = matches
            .get_one::<String>("password")
            .cloned() // Використовуємо cloned(), щоб отримати власну копію
            .or_else(|| {
                println!("Enter password:");
                match read_password() {
                    Ok(password) => Some(password), // Повертаємо String
                    Err(e) => {
                        eprintln!("Error reading password: {:?}", e);
                        None
                    }
                }
            });

        if let Some(password) = password {
            match db::is_password_correct(password.as_bytes()) {
                Ok(true) => {
                    match keys::get_wallet_address(password.as_bytes()) {
                        // Виклик правильного методу
                        Ok(address) => println!("Wallet Address: {}", address),
                        Err(e) => eprintln!("Error retrieving wallet address: {:?}", e),
                    }
                }
                Ok(false) => eprintln!("Incorrect password."),
                Err(e) => eprintln!("Error checking password: {:?}", e),
            }
        }
    }

    // Якщо користувач вказав --list-assets, виводимо список
    if matches.get_flag("list-assets") {
        match db::get_all_cryptoassets() {
            Ok(assets) => {
                for asset in assets {
                    // Якщо description може бути NULL, краще підставити пустий рядок
                    let desc = asset.description.unwrap_or_default();
                    println!("{}\t{}\t{}", asset.id, asset.symbol, desc);
                }
            }
            Err(e) => eprintln!("Error retrieving crypto assets: {:?}", e),
        }
    }

    // ------------------
    // 3) Логіка для --send amount currency_id recipient
    // ------------------
    if let Some(values) = matches.get_many::<String>("send") {
        // Перетворюємо ValuesRef у вектор
        let values: Vec<&String> = values.collect();

        // Переконуємося, що у нас є рівно три аргументи
        if values.len() != 3 {
            eprintln!("--send requires exactly 3 arguments: AMOUNT CURRENCY_ID RECIPIENT");
            return;
        }

        let amount_str = &values[0];
        let currency_id_str = &values[1];
        let recipient = &values[2];

        // Отримання пароля
        let password = matches.get_one::<String>("password").cloned().or_else(|| {
            println!("Enter password:");
            match read_password() {
                Ok(password) => Some(password),
                Err(e) => {
                    eprintln!("Error reading password: {:?}", e);
                    None
                }
            }
        });

        if let Some(password) = password {
            // Перевірка пароля
            match db::is_password_correct(password.as_bytes()) {
                Ok(true) => {
                    // Спробуємо конвертувати currency_id_str у u32
                    let currency_id: u32 = match currency_id_str.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            eprintln!(
                                "Invalid currency_id: '{}'. Must be a valid u32.",
                                currency_id_str
                            );
                            return;
                        }
                    };

                    // Зберігаємо сума як текст, а currency_id як u32
                    println!("Send request received:");
                    println!("  Amount (string): {}", amount_str);
                    println!("  Currency ID (u32): {}", currency_id);
                    println!("  Recipient: {}", recipient);

                    match tx::send_money(&password, &amount_str, currency_id, &recipient) {
                        Ok(transaction) => match tx::store_transaction(&transaction) {
                            Ok(_) => {
                                match save_transaction_as_json(&transaction) {
                                    Ok(_) => println!("Ok"),
                                    Err(e) => println!("Err {}", e),
                                }
                                println!("Ok");
                            }
                            Err(e) => println!("Err {}", e),
                        },
                        Err(e) => println!("Err {}", e),
                    };
                }
                Ok(false) => eprintln!("Incorrect password."),
                Err(e) => eprintln!("Error checking password: {:?}", e),
            }
        }
    }

    // Нова логіка для --replenishing
    if let Some(values) = matches.get_many::<String>("replenishing") {
        let values: Vec<&String> = values.collect();
        if values.len() != 4 {
            eprintln!("--replenishing requires exactly 4 arguments: RECIPIENT_ADDRESS AMOUNT CURRENCY_ID SOURCE_TRANSACTION_HASH");
            return;
        }

        let recipient_address = &values[0];
        let amount_str = &values[1];
        let currency_id_str = &values[2];
        let source_transaction_hash = &values[3];

        // Спробуємо конвертувати currency_id_str у u32
        let currency_id: u32 = match currency_id_str.parse() {
            Ok(val) => val,
            Err(_) => {
                eprintln!(
                    "Invalid currency_id: '{}'. Must be a valid u32.",
                    currency_id_str
                );
                return;
            }
        };

        println!("Replenishing request received:");
        println!("  Recipient Address: {}", recipient_address);
        println!("  Amount: {}", amount_str);
        println!("  Currency ID: {}", currency_id);
        println!("  Source Transaction Hash: {}", source_transaction_hash);

        match tx::replenishing(
            &recipient_address,
            &amount_str,
            currency_id,
            &source_transaction_hash,
        ) {
            Ok(transaction) => match tx::store_transaction(&transaction) {
                Ok(_) => {
                    match save_transaction_as_json(&transaction) {
                        Ok(_) => println!("Ok"),
                        Err(e) => println!("Err {}", e),
                    }
                    println!("Ok");
                }
                Err(e) => println!("Err {}", e),
            },
            Err(e) => println!("Err {}", e),
        }
    }

    // 5) Логіка для --balance [address?]
    if matches.contains_id("balance") {
        let maybe_address = matches.get_one::<String>("balance");

        // Якщо немає адреси, беремо власну
        let address = if let Some(addr) = maybe_address {
            addr.clone()
        } else {
            println!("No wallet address provided. Showing your own wallet balance requires the password:");
            let password = match get_or_prompt_password(&matches) {
                Some(p) => p,
                None => {
                    eprintln!("Cannot read password. Aborting balance check.");
                    return;
                }
            };
            // Перевіряємо пароль і дістаємо адресу з БД
            match db::is_password_correct(password.as_bytes()) {
                Ok(true) => match keys::get_wallet_address(password.as_bytes()) {
                    Ok(addr) => addr,
                    Err(e) => {
                        eprintln!("Error retrieving your wallet address: {:?}", e);
                        return;
                    }
                },
                Ok(false) => {
                    eprintln!("Incorrect password.");
                    return;
                }
                Err(e) => {
                    eprintln!("Error checking password: {:?}", e);
                    return;
                }
            }
        };

        // Отримуємо 32 байти балансу з локальної БД
        match db::get_wallet_balance(&address) {
            Ok(balance_bytes) => {
                let big_balance = U256::from_big_endian(&balance_bytes);

                // Форматуємо з урахуванням 18 дец. знаків (наприклад, якщо це wei)
                match format_units(big_balance, 18) {
                    Ok(value) => println!("Balance of {}:\n{}", address, value),
                    Err(e) => eprintln!("Error formatting balance: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error retrieving balance: {:?}", e),
        }
    }

    // Логіка для --import <file_path>
    if let Some(file_path) = matches.get_one::<String>("import") {
        println!("Import transaction from file: {}", file_path);
        match import_transaction(file_path) {
            Ok(content) => {
                println!("File content:\n{}", content);
                // Use `?` safely now that main() returns `Result`
                match tx::json_to_txpb(&content) {
                    Ok(tx_db) => match tx::store_transaction(&tx_db) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error checking password: {:?}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error checking password: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => eprintln!("Error importing transaction: {:?}", e),
        }
    }

    greet();
}

// Допоміжна функція - отримати пароль або прочитати з консолі, якщо не переданий
fn get_or_prompt_password(matches: &clap::ArgMatches) -> Option<String> {
    if let Some(pass) = matches.get_one::<String>("password") {
        return Some(pass.clone());
    }
    println!("Enter password:");
    match read_password() {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("Error reading password: {:?}", e);
            None
        }
    }
}

// Функція імпорту транзакції з файлу (поки що лише зчитує весь файл як текст і повертає)
fn import_transaction(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn save_transaction_as_json(transaction: &TransactionPb) -> Result<(), Box<dyn std::error::Error>> {
    let tx_db = tx::to_transaction_db(&transaction);
    let tx_json = tx::tx_to_json(&tx_db)?;

    let file_path = hex::encode(transaction.transaction_hash.clone()) + ".osnjs";

    // Записуємо байти у файл
    let mut file = File::create(file_path)?;
    file.write_all(tx_json.as_bytes())?;
    Ok(())
}

pub fn prompt_for_password() -> Option<String> {
    loop {
        println!("Enter new password:");
        let password = match read_password() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading password: {:?}", e);
                return None;
            }
        };

        println!("Confirm password:");
        let confirm_password = match read_password() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading confirmation password: {:?}", e);
                return None;
            }
        };

        if password == confirm_password {
            return Some(password);
        } else {
            println!("Passwords do not match. Please try again.");
        }
    }
}

pub fn greet() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    println!("Welcome to {} v{}!", name, version);
}

// Функція для збереження транзакції у файл
pub fn save_transaction_to_file(
    transaction: &TransactionPb,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = hex::encode(transaction.transaction_hash.clone()) + ".osnpb";
    // Серіалізуємо об'єкт у вектор байтів
    let mut buf = Vec::new();
    transaction.encode(&mut buf)?;

    // Записуємо байти у файл
    let mut file = File::create(file_path)?;
    file.write_all(&buf)?;
    Ok(())
}
