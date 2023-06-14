//dodac przy logowaniu haslo czy cos
pub mod server {
    use core::time;
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

    fn handle_balance(customers: Arc<Mutex<Vec<Customer>>>, idx: usize) -> String {
        let customer = customers.lock().unwrap();
        format!("Balance: {:.2}", customer[idx].balance)
    }

    fn handle_withdraw(
        parts: &[String],
        customers: Arc<Mutex<Vec<Customer>>>,
        idx: usize,
    ) -> String {
        let mut binding = customers.lock().unwrap();
        let customer = binding.get_mut(idx).unwrap();

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

    fn handle_deposit(
        parts: &[String],
        customers: Arc<Mutex<Vec<Customer>>>,
        idx: usize,
    ) -> String {
        let mut binding = customers.lock().unwrap();
        let customer = binding.get_mut(idx).unwrap();
        let amount = parts.get(2).unwrap().to_string().parse::<f64>().unwrap();
        customer.balance += amount;
        "Success".to_string()
    }

    fn handle_transfer(
        parts: &[String],
        customers_mut: Arc<Mutex<Vec<Customer>>>,
        idx: usize,
    ) -> String {
        let mut customers = customers_mut.lock().unwrap();

        let dest = parts.get(2).unwrap().to_string();
        let amount = parts.get(3).unwrap().to_string().parse::<f64>().unwrap();
        let pin = parts.get(4).unwrap().to_string();
        if amount <= 0.0 {
            format!("Incorrect ammount cannot be negative")
        } else {
            if customers[idx].pin != pin {
                "Invalid PIN.".to_string()
            } else if customers[idx].balance >= amount {
                if let Some(index2) = find_account(dest.as_str(), customers.as_ref()) {
                    customers[idx].balance -= amount;
                    customers[index2].balance += amount;
                    format!("Success! New balance: {}", customers[idx].balance)
                } else {
                    "Unknown destination account number".to_string()
                }
            } else {
                "Insufficient funds.".to_string()
            }
        }
    }

    fn validate_customer(
        account_number: &str,
        pin: &str,
        customers: Arc<Mutex<Vec<Customer>>>,
    ) -> Option<usize> {
        let customers = customers.lock().unwrap();
        let customer = customers
            .iter()
            .position(|customer| customer.account_number == account_number);
        if let Some(idx) = customer {
            if customers[idx].pin == pin {
                return Some(idx);
            }
        }
        return None;
    }

    fn convert_stream_to_parts(
        stream: &mut TcpStream,
        buffer: &mut [u8],
    ) -> Result<Vec<String>, ()> {
        let bytes_read = stream.read(buffer).unwrap();
        if bytes_read == 0 {
            return Err(());
        }

        let input = String::from_utf8_lossy(&buffer[..bytes_read]).to_owned();

        Ok(input.split_whitespace().map(|s| s.to_owned()).collect())
    }

    pub fn handle_client(mut stream: TcpStream, customers: Arc<Mutex<Vec<Customer>>>) {
        let mut buffer: [u8; 512] = [0; 512];
        dbg!(&stream);

        let Ok(parts) = convert_stream_to_parts(&mut stream, &mut buffer) else {  let _ = stream.write("inbalid request".as_bytes()); return };
        dbg!(&parts);
        let account_number = parts[1].clone();
        let pin = parts[2].to_string();
        let Some(customer_index) = validate_customer(&account_number, &pin, customers.clone())
        else {
            let _ = stream.write("not valid".as_bytes());
            return};
        let _ = stream.write("logged in!".trim().as_bytes());

        loop {
            let Ok(parts) = convert_stream_to_parts(&mut stream, &mut buffer) else { continue };

            dbg!(&parts);

            let command = parts[0].clone();

            println!(
                "Command: '{}', account number: {}",
                command,
                account_number.clone()
            );
            let response: String;

            response = match command.trim() {
                "balance" => handle_balance(customers.clone(), customer_index),
                "withdraw" => handle_withdraw(&parts, customers.clone(), customer_index),
                "deposit" => handle_deposit(&parts, customers.clone(), customer_index),
                "transfer" => handle_transfer(&parts, customers.clone(), customer_index),

                _ => "Invalid operation.".to_string(),
            };
            write_customers_to_database(customers.clone(), SAVE_PATH)
                .expect("Failed to save customers to file.");

            let _ = stream.write(response.as_bytes());
            thread::sleep(time::Duration::from_millis(50));
        }
    }
    // pub fn handle_client(mut stream: TcpStream, customers: Arc<Mutex<Vec<Customer>>>) {
    //     let mut buffer = [0; 512];

    //     let bytes_read = stream.read(&mut buffer).unwrap();
    //     if bytes_read == 0 {
    //         return;
    //     }

    //     let input = String::from_utf8_lossy(&buffer[..bytes_read]);
    //     let parts: Vec<&str> = input.split_whitespace().collect();

    //     dbg!(&parts);
    //     let command = parts[0];
    //     let account_number = parts[1];
    //     let pin = parts[2];

    //     loop {
    //         let bytes_read = stream.read(&mut buffer).unwrap();
    //         if bytes_read == 0 {
    //             return;
    //         }

    //         let input = String::from_utf8_lossy(&buffer[..bytes_read]);
    //         let parts: Vec<&str> = input.split_whitespace().collect();

    //         dbg!(&parts);
    //         let command = parts[0];
    //         let account_number = parts[1];

    //         let response: String;

    //         println!(
    //             "Command: '{}', account number: {}",
    //             command,
    //             account_number.clone()
    //         );
    //         let mut customers = customers.lock().unwrap();

    //         match find_account(account_number, customers.as_ref()) {
    //             Some(index) => {
    //                 let customer = &mut customers[index];
    //                 response = match command.trim() {
    //                     "balance" => handle_balance(customer),
    //                     "withdraw" => handle_withdraw(&parts, customer),
    //                     "deposit" => handle_deposit(&parts, customer),
    //                     "transfer" => {
    //                         // transferowanie
    //                         // todo handle_transfer
    //                         let dest = parts.get(2).unwrap().to_string();
    //                         let amount = parts.get(3).unwrap().to_string().parse::<f64>().unwrap();
    //                         let pin = parts.get(4).unwrap().to_string();
    //                         if amount <= 0.0 {
    //                             format!("Incorrect ammount cannot be negative")
    //                         } else {
    //                             if customer.pin != pin {
    //                                 "Invalid PIN.".to_string()
    //                             } else if customer.balance >= amount {
    //                                 if let Some(index2) =
    //                                     find_account(dest.as_str(), customers.as_ref())
    //                                 {
    //                                     customers[index].balance -= amount;
    //                                     customers[index2].balance += amount;
    //                                     format!(
    //                                         "Success! New balance: {}",
    //                                         customers[index].balance
    //                                     )
    //                                 } else {
    //                                     "Unknown destination account number".to_string()
    //                                 }
    //                             } else {
    //                                 "Insufficient funds.".to_string()
    //                             }
    //                         }
    //                     }
    //                     _ => "Invalid operation.".to_string(),
    //                 };
    //                 write_customers_to_database(customers.as_ref(), SAVE_PATH)
    //                     .expect("Failed to save customers to file.");
    //             }

    //             None => {
    //                 response = "Unknown account number".to_string();
    //             }
    //         }
    //         let _ = stream.write(response.as_bytes());
    //     }
    // }

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
