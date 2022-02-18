mod client;
mod transactions;
use clap::{App, Arg};
use transactions::{Transaction, TransactionsDispatcher};
use csv;
// LCOV_EXCL_START
fn get_transactions(transactions: &str) -> Vec<Transaction> {
    let mut data = csv::Reader::from_path(transactions).unwrap();
    let data = data.deserialize::<Transaction>();
    let mut vec = vec![];
    data.for_each(|t| {
        vec.push(t.unwrap()); 
    });
    vec
}


#[tokio::main]
async fn main() {
    let matches = App::new("Program realizes simple account transactions: deposit, withdrawal, dispute, resolve and charge-back. It has been prepared solely for training purposes.")
        .author("Kamil Łasek")
        .version("1.0.0")
        .usage(
            "The program requires a file with transactions as input in csv format such as:

            type,       client, tx,	amount
            deposit,        1,  1,	   1.0
            withdrawal,     2,  2,       0.234
            "
        )
        .arg(
            Arg::with_name("transactions")
                .required(true)
                .takes_value(true)
                .help("scv file contains transactions")
        )
        .get_matches();

    let transactions = get_transactions(&matches.value_of("transactions").unwrap());
    let mut td = TransactionsDispatcher::new();
    for i in 0..transactions.len() {
        match td.process_transactions(&transactions[i.clone()]).await{
            Err(err) => println!("During processing transaction {:?} error occured:\n {} ",transactions[i],err),
            _=> ()
        }
    }
    td.print_output();
}

// LCOV_EXCL_STOP
