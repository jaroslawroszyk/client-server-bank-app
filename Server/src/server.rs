// use std::io::{Read, Write};
// use std::net::TcpListener;
// use std::thread;

// #[derive(Debug)]
// struct Client {
//     name: String,
//     surname: String,
//     pesel: i32,
//     acc_no: i32,
//     balance: f64,
// }

// impl Client {
//     fn new(name: String, surname: String, pesel: i32, acc_no: i32, balance: f64) -> Self {
//         Self {
//             name,
//             surname,
//             pesel,
//             acc_no,
//             balance,
//         }
//     }

//     fn deposit(&mut self, ammount: f64) {
//         self.balance += ammount;
//     }

//     fn withdraw(&mut self, ammount: f64) {
//         self.balance -= ammount;
//     }

//     fn get_balance(&self) {
//         self.balance;
//     }
// }

// fn handle_client(mut stream: std::net::TcpStream) {
//     let mut buf = [0; 512];
//     loop {
//         let bytes_read = stream.read(&mut buf).unwrap();
//         if bytes_read == 0 {
//             return;
//         }
//         let received = String::from_utf8_lossy(&buf[0..bytes_read]);
//         // println!("Recived msg: {}", received);
//         // let response = format!("response: {}",msg);
//         let response = match received.trim() {
//             "siema" => String::from("siema mordo"),
//             "co tam" => String::from("dobrze a u ciebie, może piwka?"),
//             _ => String::from("Nie rozumiem :("),
//         };
//         let _ = stream.write(response.as_bytes());
//     }
// }

// fn main() {
//     let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
//     println!("Listening on 127.0.0.1:8080");
//     for stream in listener.incoming() {
//         match stream {
//             Ok(stream) => {
//                 println!("Accepted connection from {:?}", stream.peer_addr().unwrap());
//                 thread::spawn(move || handle_client(stream));
//             }
//             Err(e) => {
//                 println!("Error: {}", e);
//             }
//         }
//     }
// }

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// #[derive(Debug)]
#[allow(dead_code)]
struct Client {
    first_name: String,
    last_name: String,
    pesel: String,
    account_number: String,
    balance: f64,
}

impl Client {
    fn new(
        first_name: String,
        last_name: String,
        pesel: String,
        account_number: String,
        balance: f64,
    ) -> Self {
        Self {
            first_name,
            last_name,
            pesel,
            account_number,
            balance,
        }
    }

    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: f64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            return true;
        }
        false
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }
}

// fn handle_client(mut stream: TcpStream, client: Arc<Mutex<Client>>) {
//     loop {
//         let mut buf = [0; 512];
//         let bytes_read = stream.read(&mut buf).unwrap();
//         if bytes_read == 0 {
//             return;
//         }
//         let client_ref = client.lock().unwrap();
//         let msg = String::from_utf8_lossy(&buf[0..bytes_read]);
//         match msg.trim() {
//             "saldo" => {
//                 let response = format!("Saldo klienta: {}", client_ref.get_balance());
//                 let _ = stream.write(response.as_bytes());
//             }
//             "deposit" => {
//                 let response = format!("Deposit ammount: {}", client_ref.get_balance());
//                 let _ = stream.write(response.as_bytes());
//             }
//             _ => {
//                 let _ = stream.write(b"Nieznane polecenie");
//             }
//         }
//     }
// }

fn handle_client(mut stream: TcpStream, client: &Arc<Mutex<Client>>) -> io::Result<()> {
    Ok(loop {
        let mut input = [0; 512];
        let bytes_read = stream.read(&mut input)?;
        let input_str = std::str::from_utf8(&input[..bytes_read]).unwrap().trim();

        if bytes_read == 0 {
            return Ok(());
        }

        let mut client_ref = client.lock().unwrap();
        /*

                 match num{
          1=>println!("One"),
          2=>println!("Two"),
          3=>println!("Three"),
          _=>println!("Rest of the number"),

        }
                 */
        match input_str {
            "clear" => {
                let response = format!("\x1B[2J\x1B[1;1H");
                stream.write_all(response.as_bytes())?;
            }
            "saldo" => {
                let response = format!("Saldo klienta: {}", client_ref.get_balance());
                stream.write_all(response.as_bytes())?;
            }
            cmd if cmd.starts_with("deposit ") => {
                let amount_str = cmd.trim_start_matches("deposit ").trim();
                let amount = match amount_str.parse::<f64>() {
                    Ok(amount) => amount,
                    Err(_) => {
                        stream.write_all(b"Nieprawidlowa kwota\n")?;
                        continue;
                    }
                };
                client_ref.deposit(amount);
                stream.write_all(b"Wplata zrealizowana\n")?;
            }
            cmd if cmd.starts_with("withdraw ") => {
                let amount_str = cmd.trim_start_matches("withdraw ").trim();
                let ammount = match amount_str.parse::<f64>() {
                    Ok(ammount) => ammount,
                    Err(_) => {
                        stream.write_all(b"Nieprawidlowa kwota\n")?;
                        continue;
                    }
                };
                client_ref.withdraw(ammount);
                stream.write_all(b"Wyplata zrealizowana\n")?;
            }
            "exit" => {
                break;
            }
            _ => {
                stream.write_all(b"Nieznane polecenie\n")?;
            }
        }

        // if input_str == "saldo" {
        //     let response = format!("Saldo klienta: {}", client_ref.get_balance());
        //     stream.write_all(response.as_bytes())?;
        // } else if input_str.starts_with("deposit ") {
        //     let amount_str = input_str.trim_start_matches("deposit ").trim();
        //     let amount = match amount_str.parse::<f64>() {
        //         Ok(amount) => amount,
        //         Err(_) => {
        //             stream.write_all(b"Nieprawidlowa kwota\n")?;
        //             continue;
        //         }
        //     };
        //     client_ref.deposit(amount);
        //     stream.write_all(b"Wplata zrealizowana\n")?;
        // } else {
        //     stream.write_all(b"Nieznane polecenie\n")?;
        // }
    })
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let client = Arc::new(Mutex::new(Client::new(
        "John".to_string(),
        "Doe".to_string(),
        "1234567890".to_string(),
        "01234567890123456789012345".to_string(),
        1000.0,
    )));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let client = Arc::clone(&client);
        thread::spawn(move || {
            handle_client(stream, &client);
        });
    }
}
