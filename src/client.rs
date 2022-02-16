// use std::sync::Arc;
use super::Transaction;
use serde::Serialize;
use anyhow::Result;
use std::sync::atomic::{AtomicUsize, Ordering};

static CLIENT_ID: AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { CLIENT_ID.fetch_add(1, Ordering::Release) }


#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct Client {
    #[serde(rename = "client")]
    pub id: u16,
    /// The total funds that are available for trading, staking, withdrawal, etc. This should be equal to the (total - held) amounts
    pub available: f32, 
    /// The total funds that are held for dispute. This should be equal to (total - available) amounts
    pub held: f32, 
    /// The total funds that are available or held. This should be equal to (available +   held)
    pub total: f32, 
     /// Whether the account is locked. An account is locked if a (charge back occurs)??
    pub locked: bool,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ClientError {
    #[error("Client operation failure {0}")]
    Other(String),
}
impl Client {
    pub fn new(id: Option<u16>, available: Option<f32>, held: Option<f32>, total: Option<f32>, locked: Option<bool> ) -> Client {
        let id = id.unwrap_or((get_id()) as u16);
        let locked = locked.unwrap_or(false);
        let held = held.unwrap_or(0.0);
        let available = available.unwrap_or(0.0);
        let total = total.unwrap_or(held + available);
        Client{id,available,held,total,locked}
    }

    fn update_total(&mut self) -> Result<(), ClientError> {
        self.total = self.available + self.held;
        Ok(())
    }

    fn update_available(&mut self) -> Result<(), ClientError> {
        if self.total - self.held < 0.0{
            return Err(ClientError::Other("Available funds insufficient".to_string()));
        }
        self.available = self.total - self.held;
        Ok(())
    }

    pub async  fn withdrawal(&mut self,amount: f32) -> Result<(), ClientError> {
        if amount > self.available {
            return Err(ClientError::Other("Withdrawal failed due to insufficient funds".to_string()));
        }
        self.available -= amount;
        self.update_total()
    }

    pub async fn deposit(&mut self,amount: f32) -> Result<(), ClientError> {
        self.available += amount;
        self.update_total()
    }

    pub async fn dispute(&mut self, transaction: Transaction) ->Result<(), ClientError> {
        if self.available < transaction.amount.unwrap_or(0.0) {
            return Err(ClientError::Other("Dispute failed due to insufficient funds on account".to_string()));
        }
        self.available -= transaction.amount.unwrap_or(0.0);
        self.held += transaction.amount.unwrap_or(0.0);
        self.update_total()
    }

    pub async fn resolve(&mut self, transaction: Transaction) ->Result<(), ClientError> {
        if self.held < transaction.amount.unwrap_or(0.0) {
            return Err(ClientError::Other("Resolve dispute failed due to insufficient funds on held account".to_string()));
        }
        self.held -= transaction.amount.unwrap_or(0.0);
        self.update_available()
    }

    pub async fn chargeback(&mut self, transaction: Transaction) ->Result<(), ClientError> {
        if self.held < transaction.amount.unwrap_or(0.0) {
            return Err(ClientError::Other("Chargeback failed due to insufficient funds.".to_string()));
        }
        self.locked = true;
        self.held -=transaction.amount.unwrap_or(0.0);
        self.update_total()
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::transactions::TransactionT;
    
    #[test]
    fn test_update_total() {
        let mut c1 = Client{
            id: 1,
            held: 1.0,
            total: 1.0,
            available: 1.0,
            locked: false
        };
        assert_eq!(c1.update_total().unwrap(), ());
    }

    #[test]
    fn test_update_available() {
        let mut c1 = Client::new(None, None, None, None, None);
        assert_eq!(c1.update_available().unwrap(), ());
    }

    #[test]
    fn test_new() {
        let c1 = Client::new(None, None, None, None, None);
        let c2 = Client::new(Some(100),Some(1.0),Some(1.0),Some(1.0),Some(true));
        let c3 = Client{
            id: 100,
            held: 1.0,
            total: 1.0,
            available: 1.0,
            locked: true
        };

        assert_ne!(c1,c2);
        assert_eq!(c2,c3);
    }

    #[tokio::test]
    async fn test_withdrawal() {
        let mut c1 = Client::new(None, Some(100.0), Some(20.0), None, None);
        assert_eq!(c1.withdrawal(80.0).await,Ok(()));
        assert_ne!(c1.withdrawal(80.0).await,Ok(()));
        }

    #[tokio::test]
    async fn test_deposit() {
        let mut c1 = Client::new(None, Some(100.0), Some(20.0), None, None);
        assert_eq!(c1.deposit(80.0).await,Ok(()));
        assert_eq!(c1.total,200.0);
        }

    #[tokio::test]
    async fn test_resolve() {
        let mut c1 = Client::new(None, Some(100.0), Some(20.0), None, None);
        let t = Transaction::new(TransactionT::Resolve, CLIENT_ID.load(Ordering::Relaxed)as u16, 0, Some(20.0));
        assert_eq!(c1.resolve(t).await,Ok(()));
        assert_eq!(c1.held,0.0);
        assert_eq!(c1.available, 120.0)
    }

    #[tokio::test]
    async fn test_dispute() {
        let mut c1 = Client::new(None, Some(100.0), Some(20.0), None, None);
        let t = Transaction::new(TransactionT::Dispute,CLIENT_ID.load(Ordering::Relaxed) as u16,0, Some(20.0));
        assert_eq!(c1.dispute(t).await,Ok(()));
        assert_eq!(c1.held,40.0);
        assert_eq!(c1.available, 80.0);
        assert_eq!(c1.total, 120.0);
    }
    #[tokio::test]
    async fn test_chargeback() {
        let mut c1 = Client::new(None, Some(100.0), Some(20.0), None, None);
        let t = Transaction::new(TransactionT::Chargeback,CLIENT_ID.load(Ordering::Relaxed) as u16,0, Some(20.0));
        assert_eq!(c1.chargeback(t).await,Ok(()));
        assert_eq!(c1.held,0.0);
        assert_eq!(c1.total, 100.0);
        assert_eq!(c1.locked, true);
    }


}
