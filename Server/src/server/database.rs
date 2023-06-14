pub mod database {
    use std::sync::{Arc, Mutex};

    use crate::server::customer::Customer;
    use rusqlite::{Connection, Result};

    pub fn write_customers_to_database(
        customers_mutex: Arc<Mutex<Vec<Customer>>>,
        db_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = Connection::open(db_path)?;
        conn.execute("DELETE FROM customers", [])?; // Clear existing data

        let tx = conn.transaction()?;
        let mut statement = tx.prepare(
            "INSERT INTO customers (name, surname, pesel, account_number, pin, balance)
             VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        let customers = customers_mutex.lock().unwrap();

        for customer in customers.iter() {
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

    pub fn read_customers_from_database(
        db_path: &str,
    ) -> Result<Vec<Customer>, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        let mut statement = conn
            .prepare("SELECT name, surname, pesel, account_number, pin, balance FROM customers")?;
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
        customers.map_err(|err| err.into())
    }
}
