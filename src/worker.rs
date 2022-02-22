
use super::transactions::{Transaction, TransactionsDispatcher};
use std::sync::mpsc::Receiver;

pub async fn worker(receiver: Receiver<Transaction>) {
    let mut td = TransactionsDispatcher::new();
    loop {
        let transaction = match receiver.recv() {
            Err(_) => {
                td.print_output();
                break;
            },
            Ok(t) => t,
        };
        match td.process_transactions(&transaction).await{
            Err(err) => println!("During processing transaction {:?} error occured:\n {} ",transaction,err),
            _=> (),
        };
    }
}

