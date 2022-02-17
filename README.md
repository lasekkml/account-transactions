# account-transactions
Program realizes simple account transactions: deposit, withdrawal, dispute, resolve and charge-back. It has been prepared solely for training purposes.
Current solution assumes that all transactions are valid, in example "discussions" transactions are not checked if the client id matches to correspond to transaction id from which the dispute relates.
There is no mechanism to unfreeze charged-back client.

## Usage:
The program requires a file with transactions as input in csv format such as:

            type,  client,  tx,  amount
         deposit,       1,   1,     1.0
      withdrawal,       2,   2,   0.234    



Run program:

    account-transactions <transactions-file>

Output will be printed on the screen in format:

    client, available, held, total, locked
         1,       1.5,  0.0,   1.5, false
         2,       2.0,  0.0,   2.0, false
    
