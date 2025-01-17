use osanwelib::db;
use clap::{Arg, Command};



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
                .value_parser(clap::value_parser!(String))
        )
        .get_matches()
}

fn main() {
    let matches = get_matches();

    let db_path = matches.get_one::<String>("db_path")
        .map(|s| s.as_str())
        .unwrap_or("osanwe.db");

    if let Err(e) = db::check_and_create_database(db_path) {
        eprintln!("Error checking or creating database: {:?}", e);
    }

    greet();
}

pub fn greet() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    println!("Welcome to {} v{}!", name, version);
}