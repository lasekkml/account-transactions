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

fn get_params() -> String {
    let matches = App::new("Program writted for interview purpose as an exercise")
        .author("Kamil Łasek")
        .version("1.0.0")
        .usage(
            "Transactions application 
    Reqires file with transaction in csv format such as:

    client, available, held, total, locked
         1,       1.5,  0.0,   1.5, false
         2,       2.0,  0.0,   2.0, false"
        )
        .arg(
            Arg::with_name("transactions")
                .required(true)
                .takes_value(true)
                .help("scv file contains transactions")
        )
        .get_matches();
    matches.value_of("transactions").unwrap().to_string()
}

#[tokio::main]
async fn main() {
    let mut td = TransactionsDispatcher::new();
    let transactions = get_transactions(&get_params());
    for i in 0..transactions.len() {
        println!("i={}",&i);
        match td.process_transactions(&transactions[i.clone()]).await{
            Err(err) => println!("During processing transaction {:?} error occured:\n {} ",transactions[i],err),
            _=> ()
        }
    }
    td.print_output();
}

// LCOV_EXCL_STOP