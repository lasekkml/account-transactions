use super::client::{Client,ClientError};
use serde::{Deserialize, Serialize,Deserializer};


#[derive(Serialize, Debug, PartialEq, Clone)]
pub enum TransactionT {
    /// A deposit is a credit to the client's asset account, meaning it should increase the available and
    /// total funds of the client account
    Deposit,
    /// A withdraw is a debit to the client's asset account, meaning it should decrease the available and
    /// total funds of the client account
    Withdrawal,
    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    /// The transaction shouldn't be reversed yet but the associated funds should be held. This means
    /// that the clients available funds should decrease by the amount disputed, their held funds should
    /// increase by the amount disputed, while their total funds should remain the same.
    Dispute,
    /// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
    /// were previously disputed are no longer disputed. This means that the clients held funds should
    /// decrease by the amount no longer disputed, their available funds should increase by the
    /// amount no longer disputed, and their total funds should remain the same.
    Resolve,
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn. This means that the clients held funds and
    /// total funds should decrease by the amount previously disputed. If a chargeback occurs the
    /// client's account should be immediately frozen.
    Chargeback,
}
// LCOV_EXCL_START
fn transaction_deserializer<'de,D>(deserializer: D) -> Result<TransactionT, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de>
        {
           match String::deserialize(deserializer).unwrap().to_lowercase().as_str() {
               "deposit" => Ok(TransactionT::Deposit),
               "withdrawal" => Ok(TransactionT::Withdrawal),
               "dispute" => Ok(TransactionT::Dispute),
               "resolve" => Ok(TransactionT::Resolve),
               "chargeback" => Ok(TransactionT::Chargeback),
               _=> Err(serde::de::Error::custom("transaction type incorrect")),
           }
        }

// LCOV_EXCL_STOP
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Transaction {
    #[serde(deserialize_with="transaction_deserializer", rename = "type")]
    /// Type of the trasaction
    pub tt: TransactionT,
    /// ClientId to which the transaction is assigned
    pub client: u16,
    /// Transaction id (should be unique)
    pub tx: u32,
    /// Optional amount
    pub amount: Option<f32>,
}

pub struct TransactionsDispatcher{
    disputes: Vec<Transaction>,
    history: Vec<Transaction>,
    clients: Vec<Client>,
}
// LCOV_EXCL_START
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Transaction processing error: {0}")]
    AccountError(#[from] ClientError),
    #[error("Transaction processing error: {0}")]
    Other(String),
}
// LCOV_EXCL_STOP
fn get_transaction_index(vec: &Vec<Transaction>,tx: u32) -> Result<usize, Error> {
    match vec.iter().position(|t| t.tx == tx) {
        Some(tx) => Ok(tx),
        None => Err(Error::Other(format!("Transaction with id {} not found",tx)))
    }
}

impl TransactionsDispatcher {
    pub fn new() -> TransactionsDispatcher{
        TransactionsDispatcher{disputes: vec![],history: vec![], clients: vec![]}
    }

    fn get_client_index(&mut self,id: u16) -> usize {
        match self.clients.iter().position(|c| c.id == id) {
            Some(c) => c,
            None => {
                self.clients.push(Client::new(Some(id),None,None,None,None));
                self.clients.len()-1
            },
        }
    }
    
// LCOV_EXCL_START
/// Prints clients accounts in csv format
    pub fn print_output(&self) ->(){
        let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(vec![]);
        self.clients.iter().for_each(|c| wrt.serialize(c).unwrap());
        print!("Output:\n{}",String::from_utf8(wrt.into_inner().unwrap()).unwrap() );
    }
// LCOV_EXCL_STOP

/// Processing transaction with client account
    pub fn process_transactions(&mut self, transaction: &Transaction) -> Result<(), Error> {
        let pos = self.get_client_index(transaction.client);
        if self.clients[pos].locked {
            return Err(Error::Other("Client is already locked".to_string()));
        }
        self.history.push(transaction.clone());
        match transaction.tt{
            TransactionT::Deposit => self.clients[pos].deposit(transaction.amount.unwrap_or(0.0))?,
            TransactionT::Withdrawal => self.clients[pos].withdrawal(transaction.amount.unwrap_or(0.0))?,
            TransactionT::Dispute => {
                let h_pos = get_transaction_index(&self.history,transaction.tx)?;
                let mut t = self.history[h_pos].clone();
                t.tt = TransactionT::Dispute;
                self.clients[pos].dispute(&t)?;
                self.disputes.push(t);
            },
            TransactionT::Resolve => {
                let d_pos = get_transaction_index(&self.disputes,transaction.tx)?;
                let mut t = self.disputes.swap_remove(d_pos);
                t.tt = TransactionT::Resolve;
                self.clients[pos].resolve(&t)?;
            },
            TransactionT::Chargeback => {
                let d_pos = get_transaction_index(&self.disputes,transaction.tx)?;
                let mut transaction = self.disputes.swap_remove(d_pos);
                transaction.tt = TransactionT::Chargeback;
                self.clients[pos].chargeback(&transaction)?;
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    impl Transaction {
        pub fn new(tt: TransactionT, client: u16, tx: u32, amount: Option<f32> ) -> Transaction {
            Transaction{tt,client,tx,amount}
        }
    }

    #[test]
    fn test_process_transactions(){
        let ts = vec![Transaction::new(TransactionT::Deposit,1,0, Some(20.0)),
        Transaction::new(TransactionT::Dispute,1,0,None), 
        Transaction::new(TransactionT::Resolve,1,0,None), 
        Transaction::new(TransactionT::Withdrawal,1,1, Some(20.0)),
        Transaction::new(TransactionT::Deposit,1,2,Some(0.1)),
        Transaction::new(TransactionT::Dispute,1,2,None),
        Transaction::new(TransactionT::Chargeback,1,2,None)];
        let mut td = TransactionsDispatcher::new();
        for i in 0..ts.len() {
            td.process_transactions(&ts[i]).unwrap();
        }
        assert_eq!(td.clients.len(),1);
        let c = td.clients.pop().unwrap();
        assert_eq!(c.total,0.0);
    }



}