# Osanwe

## Overview

Osanwe is a decentralized project that allows users to transfer cryptocurrencies without the need for a blockchain and without incurring any fees. The code is fully open-source and free to use, licensed under the MIT License. The project is developed by Arsen Huzhva.

## Purpose and Uniqueness

The main goal of Osanwe is to provide a seamless and cost-effective way to send cryptocurrencies. Unlike traditional blockchain systems, Osanwe eliminates transaction fees and delays, making it an ideal solution for users who want to transfer funds quickly and efficiently. The project features two client versions: a command-line interface (CLI) for integration with other applications and a desktop GUI version for a broader audience. Additionally, there is a server version that can be beneficial for online stores looking to accept cryptocurrency payments without intermediaries.

## Project Structure

The project consists of the following components:

- **osanwelib**: The core library that contains the main functionalities, including database management and encryption.
- **osanwecli**: The command-line interface for users who prefer to interact with the system via terminal commands.
- **osanwedt**: The desktop GUI version for users who prefer a graphical interface.
- **osanwesrv**: The server version that facilitates payment processing and transaction history analysis.

## Building the Project

To build the Osanwe project from source, follow these steps:

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/yourusername/osanwe.git
   cd osanwe
   ```

2. **Install Rust**:
   Ensure you have Rust installed on your machine. You can install it using [rustup](https://rustup.rs/).

3. **Build the Project**:
   Navigate to the project root directory and run the following command:
   ```bash
   cargo build --release
   ```

## Running the Applications

After building the project, you can run each application as follows:

- **Command-Line Interface (CLI)**:
  To run the CLI application, use:
  ```bash
  cargo run -p osanwecli
  ```

- **Desktop GUI**:
  To run the desktop GUI application, use:
  ```bash
  cargo run -p osanwedt
  ```

- **Server**:
  To run the server application, use:
  ```bash
  cargo run -p osanwesrv
  ```

## Contributing

Contributions are welcome! If you would like to contribute to the project, please fork the repository and submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
