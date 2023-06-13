pub mod server {
    use std::io::{self, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crate::server::database::database::{
        read_customers_from_database, write_customers_to_database,
    };

    use crate::server::config::SAVE_PATH;
    use crate::server::config::SERVER_ADDRESS;
    use crate::server::customer::{find_account, Customer};

    fn handle_balance(customer: &Customer) -> String {
        format!("Balance: {:.2}", customer.balance)
    }
    fn handle_withdraw(parts: &[&str], customer: &mut Customer) -> String {
        let pin = parts.get(2).unwrap().to_string();
        let amount = parts.get(3).unwrap().to_string().parse::<f64>().unwrap();

        if customer.pin != pin {
            "Invalid PIN.".to_string()
        } else if customer.balance >= amount {
            customer.balance -= amount;
            format!("Success! New balance: {:.2}", customer.balance)
        } else {
            "Insufficient funds.".to_string()
        }
    }
    // ... pozostałe funkcje obsługujące operacje

    pub fn handle_client(mut stream: TcpStream, customers: Arc<Mutex<Vec<Customer>>>) {
        let mut buffer = [0; 512];

        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                return;
            }

            let input = String::from_utf8_lossy(&buffer[..bytes_read]);
            let parts: Vec<&str> = input.split_whitespace().collect();

            let command = parts[0];
            let account_number = parts[1];

            let response: String;

            println!(
                "Command: '{}', account number: {}",
                command,
                account_number.clone()
            );
            let mut customers = customers.lock().unwrap();

            match find_account(account_number, customers.as_ref()) {
                Some(index) => {
                    let customer = &mut customers[index];
                    response = match command.trim() {
                        "balance" => handle_balance(customer),
                        "withdraw" => handle_withdraw(customer),
                        "deposit" => handle_deposit(customer),
                        // ... obsługa pozostałych operacji
                        _ => "Invalid operation.".to_string(),
                    };
                    write_customers_to_database(customers.as_ref(), SAVE_PATH)
                        .expect("Failed to save customers to file.");
                }

                None => {
                    response = "Unknown account number".to_string();
                }
            }
            let _ = stream.write(response.as_bytes());
        }
    }

    pub fn start_server() -> io::Result<()> {
        let customers = Arc::new(Mutex::new(
            read_customers_from_database(SAVE_PATH)
                .expect("Failed to read customers from save file."),
        ));

        let listener = TcpListener::bind(SERVER_ADDRESS)?;
        println!("Server listening on address {}...", SERVER_ADDRESS);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let customers = Arc::clone(&customers);
                    println!("New connection: {}", stream.peer_addr()?);
                    thread::spawn(move || handle_client(stream, customers));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }
}
