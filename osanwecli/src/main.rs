use clap::{Arg, Command};
use rpassword::read_password;
use osanwelib::{db,tx,keys};

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
        .get_matches()
}

fn main() {
    let matches = get_matches();

    if let Err(e) = db::check_and_create_database() {
        eprintln!("Error checking or creating database: {:?}", e);
    }

    match db::is_password_set() {
        Ok(is_set) => {
            if !is_set {
                println!("Password is not set. Please set a new password:");
                if let Some(password) = prompt_for_password() {
                    if let Err(e) = db::set_password(password.as_bytes()) {
                        eprintln!("Error saving password: {:?}", e);
                    } else {
                        println!("Password has been successfully set.");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error checking the password: {:?}", e);
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
                            Ok(_) => println!("Ok"),
                            Err(e) => println!("Err {}", e),
                        },
                        Err(e) => println!("Err {}", e),
                    };

                    // Тут можна додати логіку збереження транзакції в базу даних
                }
                Ok(false) => eprintln!("Incorrect password."),
                Err(e) => eprintln!("Error checking password: {:?}", e),
            }
        }
    }

    greet();
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
