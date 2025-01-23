use clap::{Arg, Command};
use osanwelib::db;
use rpassword::read_password;

pub fn get_matches() -> clap::ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Arsen Huzhva <arsenguzhva@gmail.com>")
        .about("Sending cryptocurrencies without commission and blockchain")
        .arg(
            Arg::new("db_path")
                .short('d')
                .long("db")
                .value_name("FILE")
                .help("Sets a custom database file")
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches()
}

fn main() {
    let matches = get_matches();

    let db_path = matches
        .get_one::<String>("db_path")
        .map(|s| s.as_str())
        .unwrap_or("osanwe.db");

    if let Err(e) = db::check_and_create_database(db_path) {
        eprintln!("Error checking or creating database: {:?}", e);
    }

    match db::is_password_set(db_path) {
        Ok(is_set) => {
            if !is_set {
                println!("Password is not set. Please set a new password:");
                if let Some(password) = prompt_for_password() {
                    if let Err(e) = db::set_password(db_path, password.as_bytes()) {
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
