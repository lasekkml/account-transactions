mod client;
mod transactions;
mod worker;
use std::sync::mpsc::channel;

use clap::{App, Arg};
use transactions::Transaction;
use worker::worker;
use csv;

// LCOV_EXCL_START
fn get_transactions(transactions: &str) -> Vec<Transaction> {
    let mut data = match csv::Reader::from_path(transactions){
        Ok(d) => d,
        Err(e) => {
            println!("{}",e);
            std::process::exit(1);
        },
    };
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
    let (sender,receiver) = channel();
    let worker = tokio::spawn(async move {
        worker(receiver).await;
    });
    transactions.iter().for_each(|t| sender.send(t.clone()).unwrap());
    drop(sender);
    worker.await.unwrap();



}

// LCOV_EXCL_STOP
