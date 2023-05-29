pub struct Customer {
    pub name: String,
    pub surname: String,
    pub pesel: String,
    pub account_number: String,
    pub pin: String,
    pub balance: f64,
}

pub fn find_account(account_number: &str, customers: &[Customer]) -> Option<usize> {
    customers
        .iter()
        .position(|customer| customer.account_number == account_number)
}
