use std::io::{self, Read, Write};
use std::net::TcpStream;

struct Client {
    stream: TcpStream,
    account_number: String,
    pin: String,
}

impl Client {
    fn new(address: &str, account_number: String, pin: String) -> io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(Self {
            stream,
            account_number,
            pin,
        })
    }

    fn send_request(&mut self, request: &str) -> io::Result<String> {
        self.stream.write(request.as_bytes())?;

        let mut buffer = [0; 512];
        let bytes_read = self.stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        Ok(response)
    }

    fn handle_balance_operation(&mut self) -> io::Result<()> {
        let request = format!("balance {} {}\n", self.account_number, self.pin);
        let response = self.send_request(&request)?;
        println!("{}", response);
        Ok(())
    }

    // ... pozostałe metody obsługujące operacje

    fn run(&mut self) -> io::Result<()> {
        println!("Connected to server!");

        loop {
            let operation =
                read_input("Enter your operation (balance, withdraw, deposit, transfer, exit): ")?;

            match operation.as_str() {
                "balance" => {
                    println!();
                    self.handle_balance_operation()?;
                }
                // ... obsługa pozostałych operacji
                "exit" => {
                    println!("Goodbye!");
                    break;
                }
                _ => {
                    println!("Unknown command '{}'", operation);
                }
            }
        }

        Ok(())
    }

    fn is_valid_account_number(&self) -> bool {
        self.account_number.len() == 10
    }
}

fn read_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn main() -> io::Result<()> {
    let address = "127.0.0.1:8080";

    let mut attempts = 3;
    let mut account_number = read_input("Enter your account number: ")?;
    let mut pin = String::new();

    while attempts > 0 {
        pin = read_input("Enter your pin: ")?;
        let client = Client::new(address, account_number.clone(), pin.clone());

        match client {
            Ok(mut client) => {
                if client.is_valid_account_number() {
                    println!("Valid account number and PIN!");
                    return client.run();
                } else {
                    println!("Invalid account number.");
                }
            }
            Err(_) => {
                attempts -= 1;
                println!(
                    "Invalid account number or PIN. Attempts remaining: {}",
                    attempts
                );
            }
        }
    }

    println!("Too many incorrect attempts. Exiting...");
    Ok(())
}
