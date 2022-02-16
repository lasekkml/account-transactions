use super::client::{Client,ClientError};
use serde::{Deserialize, Serialize,Deserializer};

#[derive(Serialize, Debug, PartialEq, Clone)]
pub enum TransactionT {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

fn transaction_deserializer<'de,D>(deserializer: D) -> Result<TransactionT, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de>
        {
           match String::deserialize(deserializer).unwrap().to_lowercase().as_str() {
               "deposit" => Ok(TransactionT::Deposit),
               "withdrawal" => Ok(TransactionT::Withdrawal),
               "dispute" => Ok(TransactionT::Dispute),
               "resolve" => Ok(TransactionT::Resolve),
               "cargeback" => Ok(TransactionT::Chargeback),
               _=> Err(serde::de::Error::custom("transaction type incorrect")),
           }
        }

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Transaction {
    #[serde(deserialize_with="transaction_deserializer", rename = "type")]
    pub tt: TransactionT,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

pub struct TransactionsDispatcher{
    disputes: Vec<Transaction>,
    history: Vec<Transaction>,
    clients: Vec<Client>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Transaction processing error: {0}")]
    AccountError(#[from] ClientError),
    #[error("Transaction processing error: {0}")]
    Other(String),
}

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

    pub fn print_output(&self) ->(){
        let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(vec![]);
        self.clients.iter().for_each(|c| wrt.serialize(c).unwrap());
        print!("Output:\n{}",String::from_utf8(wrt.into_inner().unwrap()).unwrap() );
    }


    pub async fn process_transactions(&mut self, transaction: &Transaction) -> Result<(), Error> {
        println!("processing: {:?}",&transaction);
        let pos = self.get_client_index(transaction.client);
        self.history.push(transaction.clone());
        match transaction.tt{
            TransactionT::Deposit => self.clients[pos].deposit(transaction.amount.unwrap_or(0.0)).await?,
            TransactionT::Withdrawal => self.clients[pos].withdrawal(transaction.amount.unwrap_or(0.0)).await?,
            TransactionT::Dispute => {
                let h_pos = get_transaction_index(&self.history,transaction.tx)?;
                let mut t = self.history[h_pos].clone();
                t.tt = TransactionT::Dispute;
                self.disputes.push(t.clone());
                self.clients[pos].dispute(t).await?;
            },
            TransactionT::Resolve => {
                let d_pos = get_transaction_index(&self.disputes,transaction.tx)?;
                let mut t = self.disputes.swap_remove(d_pos);
                t.tt = TransactionT::Resolve;
                self.clients[pos].resolve(t).await?;
            },
            TransactionT::Chargeback => {
                let d_pos = get_transaction_index(&self.disputes,transaction.tx)?;
                let mut t = self.disputes.swap_remove(d_pos);
                t.tt = TransactionT::Chargeback;
                self.clients[pos].chargeback(transaction.clone()).await?;
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

    #[tokio::test]
    async fn test_process_transactions(){
        let ts = vec![Transaction::new(TransactionT::Deposit,1,0, Some(20.0)), Transaction::new(TransactionT::Withdrawal,1,0, Some(20.0))];
        let mut td = TransactionsDispatcher::new();
        for i in 0..ts.len() {
            td.process_transactions(&ts[i]).await.unwrap();
        }
        assert_eq!(td.clients.len(),1);
        let c = td.clients.pop().unwrap();
        assert_eq!(c.total,0.0);
    }



}