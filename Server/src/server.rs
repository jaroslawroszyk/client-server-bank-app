use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
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
        print!("Nie masz pieniedzy byku");
        false
    }
    fn get_info(&self) -> String {
        format!(
            "Imie: {}\nNazwisko: {}\nPESEL: {}\nNumer konta: {}\nSaldo: {}",
            self.first_name, self.last_name, self.pesel, self.account_number, self.balance
        )
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }
}

fn handle_client(mut stream: TcpStream, client: &Arc<Mutex<Client>>) -> io::Result<()> {
    Ok(loop {
        let mut input = [0; 512];
        let bytes_read = stream.read(&mut input)?;
        let input_str = std::str::from_utf8(&input[..bytes_read]).unwrap().trim();

        if bytes_read == 0 {
            return Ok(());
        }

        let mut client_ref = client.lock().unwrap();

        match input_str {
            "clear" => {
                let response = format!("\x1B[2J\x1B[1;1H");
                stream.write_all(response.as_bytes())?;
            }
            "data" => {
                let response = format!("Dane klienta: {}", client_ref.get_info());
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
                if client_ref.withdraw(ammount) {
                    stream.write_all(b"Wyplata zrealizowana\n")?;
                } else {
                    stream.write_all(b"Brak wystarczajacych srodkow\n")?;
                }
            }
            "exit" => {
                break;
            }
            _ => {
                stream.write_all(b"Nieznane polecenie\n")?;
            }
        }
    })
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let client = Arc::new(Mutex::new(Client::new(
        "John".to_string(),
        "JOhny".to_string(),
        "123421421".to_string(),
        "1234".to_string(),
        1000.0,
    )));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let client = Arc::clone(&client);
        thread::spawn(move || {
            if let Err(e) = handle_client(stream, &client) {
                eprintln!("Error occurred while handling client: {}", e);
            }
        });
    }
}
