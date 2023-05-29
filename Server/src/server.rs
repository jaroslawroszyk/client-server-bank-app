use rusqlite::{Connection, Result};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const SAVE_PATH: &str = "data/customers.db";

struct Customer {
    name: String,
    surname: String,
    pesel: String,
    account_number: String,
    pin: String,
    balance: f64,
}

// impl Customer {
//     fn new(name: &str, surname: &str, pesel: &str, account_number: &str, pin: &str, balance: f64) -> Customer {
//         Customer {
//             name: name.to_string(),
//             surname: surname.to_string(),
//             pesel: pesel.to_string(),
//             account_number: account_number.to_string(),
//             pin: pin.to_string(),
//             balance,
//         }
//     }
// }

fn write_customers_to_database(
    customers: &[Customer],
    db_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open(db_path)?;

    conn.execute("DELETE FROM customers", [])?; // Clear existing data

    let tx = conn.transaction()?;
    let mut statement = tx.prepare(
        "INSERT INTO customers (name, surname, pesel, account_number, pin, balance)
         VALUES (?, ?, ?, ?, ?, ?)",
    )?;

    for customer in customers {
        statement.execute(&[
            &customer.name,
            &customer.surname,
            &customer.pesel,
            &customer.account_number,
            &customer.pin,
            &customer.balance.to_string(),
        ])?;
    }

    statement.finalize()?;

    tx.commit()?;
    Ok(())
}

fn read_customers_from_database(db_path: &str) -> Result<Vec<Customer>> {
    let conn = Connection::open(db_path)?;

    let mut statement =
        conn.prepare("SELECT name, surname, pesel, account_number, pin, balance FROM customers")?;
    let customer_rows = statement.query_map([], |row| {
        Ok(Customer {
            name: row.get(0)?,
            surname: row.get(1)?,
            pesel: row.get(2)?,
            account_number: row.get(3)?,
            pin: row.get(4)?,
            balance: row.get::<_, f64>(5)?,
        })
    })?;

    let customers: Result<Vec<Customer>> = customer_rows.collect();
    customers
}

fn find_account(account_number: &str, customers: &[Customer]) -> Option<usize> {
    customers
        .iter()
        .position(|customer| customer.account_number == account_number)
}

fn handle_balance(customers: &Vec<Customer>, index: usize) -> String {
    let customer = &customers[index];
    format!("Balance: {:.2}", customer.balance)
}

fn handle_client(mut stream: TcpStream, customers: Arc<Mutex<Vec<Customer>>>) {
    let mut buffer;

    loop {
        buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            return;
        }

        let input_temp = String::from_utf8_lossy(&buffer[..]);
        let input = input_temp.trim_end_matches('\0');

        let parts = input.split_whitespace().collect::<Vec<_>>();

        let command = parts.get(0).unwrap();
        let account_number = parts.get(1).unwrap();

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
                    "balance" => {
                        format!("Balance: {:.2}", customer.balance)
                    }
                    "withdraw" => {
                        let pin = parts.get(2).unwrap().to_string();
                        let amount = parts.get(3).unwrap().to_string().parse::<f64>().unwrap();

                        if customer.pin != pin {
                            "Invalid PIN.".to_string()
                        } else if customer.balance >= amount {
                            customer.balance -= amount;
                            format!("Success! New balance: {}", customer.balance)
                        } else {
                            "Insufficient funds.".to_string()
                        }
                    }
                    "deposit" => {
                        let amount = parts.get(2).unwrap().to_string().parse::<f64>().unwrap();
                        customer.balance += amount;
                        format!("Success")
                    }
                    "transfer" => {
                        let dest = parts.get(2).unwrap().to_string();
                        let amount = parts.get(3).unwrap().to_string().parse::<f64>().unwrap();
                        let pin = parts.get(4).unwrap().to_string();

                        if customer.pin != pin {
                            "Invalid PIN.".to_string()
                        } else if customer.balance >= amount {
                            if let Some(index2) = find_account(dest.as_str(), customers.as_ref()) {
                                customers[index].balance -= amount;
                                customers[index2].balance += amount;
                                format!("Success! New balance: {}", customers[index].balance)
                            } else {
                                "Unknown destination account number".to_string()
                            }
                        } else {
                            "Insufficient funds.".to_string()
                        }
                    }
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

fn main() -> io::Result<()> {
    let customers = Arc::new(Mutex::new(
        read_customers_from_database(SAVE_PATH).expect("Failed to read customers from save file."),
    ));

    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080...");

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
