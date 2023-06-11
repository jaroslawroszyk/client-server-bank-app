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
        self.stream.read(&mut buffer)?;

        let response = String::from_utf8_lossy(&buffer[..]);
        Ok(response.to_string())
    }

    fn handle_balance_operation(&mut self) -> io::Result<()> {
        let out = format!("balance {}", self.account_number);
        let response = self.send_request(&out)?;
        println!("{}", response);
        Ok(())
    }

    fn handle_withdraw_operation(&mut self) -> io::Result<()> {
        let mut amount: f64;
        loop {
            let amount_input = read_input("Enter withdraw amount: ")?;
            match amount_input.trim().parse() {
                Ok(value) if value > 0.0 => {
                    amount = value;
                    break;
                }
                _ => println!("Invalid amount. Please enter a valid number greater than 0."),
            };
        }
        let pin = read_input("Enter pin to confirm operation: ")?;
        let out = format!("withdraw {} {} {}", self.account_number, pin, amount);
        let response = self.send_request(&out)?;
        println!("{}", response);
        Ok(())
    }

    fn handle_deposit_operation(&mut self) -> io::Result<()> {
        let amount = read_input("Enter deposit amount: ")?;
        let out = format!("deposit {} {}", self.account_number, amount);
        let response = self.send_request(&out)?;
        println!("{}", response);
        Ok(())
    }

    fn handle_transfer_operation(&mut self) -> io::Result<()> {
        let number2 = loop {
            let account_number = read_input("Enter destination account number: ")?;
            // Sprawdzenie poprawności numeru konta (przykładowa logika sprawdzająca długość numeru konta)
            if account_number.len() == 10 {
                break account_number;
            }
            println!("Invalid account number. Please enter a valid account number.");
        };
    
        let mut amount = loop {
            let amount_input = read_input("Enter transfer amount: ")?;
            if let Ok(parsed_amount) = amount_input.parse::<f64>() {
                if parsed_amount >= 0.0 {
                    break parsed_amount;
                }
            }
            println!("Invalid amount. Please enter a non-negative number.");
        };
        
        let pin = loop {
            let entered_pin = read_input("Enter PIN to confirm operation: ")?;
            if entered_pin.len() == 4 {
                break entered_pin;
            }
            println!("Invalid PIN. Please enter a valid PIN.");
        };
    
        let out = format!(
            "transfer {} {} {} {}",
            self.account_number, number2, amount, pin
        );
        let response = self.send_request(&out)?;
    
        println!("{}", response);
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        println!("Connected to server!");

        Ok(loop {
            let operation =
                read_input("Enter your operation (balance, withdraw, deposit, transfer, exit): ")?;

            match operation.as_str() {
                "balance" => {
                    println!();
                    self.handle_balance_operation()?;
                }
                "withdraw" => {
                    println!();
                    self.handle_withdraw_operation()?;
                }
                "deposit" => {
                    println!();
                    self.handle_deposit_operation()?;
                }
                "transfer" => {
                    println!();
                    self.handle_transfer_operation()?;
                }
                "exit" => {
                    println!("SEA!");
                    break;
                }
                _ => {
                    println!("Unknown command '{}'", operation);
                }
            }
        })
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
    let mut account_number = String::new();
    let mut pin = String::new();
    let pin_for_client_one = "1234";
    let pin_for_client_two = "4321";

    while attempts > 0 {
        account_number = read_input("Enter your account number: ")?;
        pin = read_input("Enter your pin: ")?;

        let mut client = match Client::new(address, account_number.clone(), pin.clone()) {
            Ok(client) => client,
            Err(_) => {
                attempts -= 1;
                println!(
                    "Invalid account number or PIN! Please try again. Attempts left: {}",
                    attempts
                );
                if attempts == 0 {
                    println!("Exceeded maximum number of attempts. Exiting...");
                    return Ok(());
                }
                continue;
            }
        };

        if client.is_valid_account_number() && (client.account_number == "1234567890"  && pin == pin_for_client_one ||client.account_number == "0987654321" && pin == pin_for_client_two) {
            break;
        } else {
            attempts -= 1;
            println!(
                "Invalid account number or PIN! Please try again. Attempts left: {}",
                attempts
            );
            if attempts == 0 {
                println!("Exceeded maximum number of attempts. Exiting...");
                return Ok(());
            }
        }
    }

    let mut client = Client::new(address, account_number, pin)?;
    println!("Valid account number and PIN!");
    client.run()?;
    Ok(())
}


// Second version
// fn main() -> io::Result<()> {
//     let address = "127.0.0.1:8080";
//     // let account_number = read_input("Enter your account number: ")?;

//     // let mut client = Client::new(address, account_number.clone())?;
//     let mut client = Client::new(address, String::new())?;
//     client.run()?;
//     Ok(())
// }
