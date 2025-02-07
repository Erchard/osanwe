use std::process::{Child, Command};
use std::{env, path::PathBuf, thread, time};

fn project_root() -> PathBuf {
    let mut dir = env::current_dir().expect("Failed to get current dir");

    // Піднімаємось до кореня проєкту, де знаходиться Cargo.toml
    while !dir.join("Cargo.toml").exists() {
        dir.pop();
    }

    dir
}

fn start_server() -> Child {
    let root = project_root();

    // Перевіряємо, чи сервер вже працює
    if std::net::TcpStream::connect("127.0.0.1:50051").is_ok() {
        println!("Сервер вже запущений, завершуємо процес...");
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "osanwesrv.exe"])
            .output();

        // Додатковий цикл перевірки, щоб дочекатися повного закриття
        for _ in 0..5 {
            thread::sleep(time::Duration::from_secs(1));
            if std::net::TcpStream::connect("127.0.0.1:50051").is_err() {
                break;
            }
        }
    }

    println!("Запускаємо сервер...");
    let server_process = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("osanwesrv")
        .current_dir(&root)
        .spawn()
        .expect("Failed to start server");

    thread::sleep(time::Duration::from_secs(2)); // Додатковий час для запуску
    server_process
}

fn run_client() -> String {
    let root = project_root();

    // Спочатку встановлюємо пароль за допомогою --set-password 1234.
    // Якщо пароль вже встановлено, CLI виведе повідомлення "Password is already set. Cannot change it.",
    // але це не заважає продовженню роботи.
    let password_setup = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("osanwecli")
        .arg("--")
        .arg("--set-password")
        .arg("1234")
        .current_dir(&root)
        .output()
        .expect("Failed to set password");

    let password_stdout =
        String::from_utf8(password_setup.stdout).expect("Invalid UTF-8 in stdout");
    let password_stderr =
        String::from_utf8(password_setup.stderr).expect("Invalid UTF-8 in stderr");
    println!("Password setup stdout: {}", password_stdout);
    println!("Password setup stderr: {}", password_stderr);

    // Потім виконуємо команду поповнення гаманця --replenishing
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("osanwecli")
        .arg("--")
        .arg("--replenishing")
        .arg("0xe828fbd93539c4ef42b58d3e7261dcb53f2c6a95")
        .arg("992002.736")
        .arg("16842752")
        .arg("0x53004c1174523fb5b3ec8809c36dadf4c9300297a002d160f85c9b5eca73ca89")
        .current_dir(&root)
        .output()
        .expect("Failed to run client");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
    println!("Client stdout: {}", stdout);
    println!("Client stderr: {}", stderr);

    stdout
}

#[test]
fn test_transaction_flow() {
    println!("Запуск інтеграційного тесту...");
    let mut server = start_server();

    // Дати серверу час запуститися
    thread::sleep(time::Duration::from_secs(2));

    let output = run_client();

    assert!(
        output.contains("Transaction sent successfully"),
        "Unexpected output: {}",
        output
    );

    // Завершити сервер
    server.kill().expect("Failed to stop server");
}
