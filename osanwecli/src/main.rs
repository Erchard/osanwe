use clap::{Arg, Command};
use osanwelib::*;
use rpassword::read_password;

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
