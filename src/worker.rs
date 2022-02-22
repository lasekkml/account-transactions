
use super::transactions::{Transaction, TransactionsDispatcher};
use std::sync::mpsc::Receiver;

pub async fn worker(receiver: Receiver<Transaction>) {
    let mut td = TransactionsDispatcher::new();
    loop {
        let transaction = match receiver.recv() {
            Err(_) => {
                println!("finnish processing transactions");
                td.print_output();
                break;
            },
            Ok(t) => t,
        };
        match td.process_transactions(&transaction){
            Err(err) => println!("During processing transaction {:?} error occured:\n {} ",transaction,err),
            _=> (),
        };
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::transactions::TransactionT;
    use std::sync::mpsc::channel;
    #[tokio::test]
    async fn test_worker(){
        let ts = vec![Transaction::new(TransactionT::Deposit,1,0, Some(20.0)),
        Transaction::new(TransactionT::Dispute,1,0,None), 
        Transaction::new(TransactionT::Resolve,1,0,None), 
        Transaction::new(TransactionT::Withdrawal,1,1, Some(20.0)),
        Transaction::new(TransactionT::Deposit,1,2,Some(0.1)),
        Transaction::new(TransactionT::Dispute,1,2,None),
        Transaction::new(TransactionT::Chargeback,1,2,None)];
        
        let (sender,receiver) = channel();
        let worker = tokio::spawn(async move {
            worker(receiver).await;
        });
        ts.iter().for_each(|t| {
            sender.send(t.clone()).unwrap();
        });
        drop(sender);
        assert_eq!(worker.await.unwrap(),());
    }
    



}
