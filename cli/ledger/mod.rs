// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Aleo library.

// The Aleo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo library. If not, see <https://www.gnu.org/licenses/>.

pub mod load;
pub use load::*;

pub mod handlers;
pub use handlers::*;

use snarkvm::prelude::{
    with,
    Address,
    Block,
    BlockMemory,
    Identifier,
    Network,
    PrivateKey,
    Program,
    ProgramID,
    ProgramMemory,
    ProgramStore,
    RecordsFilter,
    Transaction,
    Value,
    ViewKey,
    Zero,
    U64,
    VM,
};

use anyhow::{bail, ensure, Result};
use core::str::FromStr;
use parking_lot::RwLock;
use snarkvm::circuit::Mode;
use std::{convert::TryFrom, sync::Arc};
use warp::{reply, Filter, Rejection};

pub(crate) type InternalStorage<N> = ProgramMemory<N>;
pub(crate) type InternalLedger<N> = snarkvm::prelude::Ledger<N, BlockMemory<N>, InternalStorage<N>>;
pub(crate) type InternalServer<N> = snarkvm::prelude::Server<N, BlockMemory<N>, InternalStorage<N>>;

/// A development ledger that provides an interface to request tokens from a faucet, deploy a program, and execute transactions
/// WARNING: This ledger is not secure and should not be used in production.
#[allow(dead_code)]
pub struct Ledger<N: Network> {
    /// The internal ledger.
    pub ledger: Arc<RwLock<InternalLedger<N>>>,
    /// The runtime.
    runtime: tokio::runtime::Runtime,
    /// The server.
    server: InternalServer<N>,
    /// The account private key.
    private_key: PrivateKey<N>,
    /// The account view key.
    view_key: ViewKey<N>,
    /// The account address.
    address: Address<N>,
}

impl<N: Network> Ledger<N> {
    /// Returns the account address.
    pub const fn address(&self) -> &Address<N> {
        &self.address
    }

    /// Returns the private key.
    pub const fn private_key(&self) -> &PrivateKey<N> {
        &self.private_key
    }

    /// Adds the given transaction to the memory pool.
    pub fn add_to_memory_pool(&self, transaction: Transaction<N>) -> Result<()> {
        self.ledger.write().add_to_memory_pool(transaction)
    }

    /// Advances the ledger to the next block.
    pub fn advance_to_next_block(&self) -> Result<Block<N>> {
        // Initialize an RNG.
        let rng = &mut ::rand::thread_rng();
        // Propose the next block.
        let next_block = self.ledger.read().propose_next_block(&self.private_key, rng)?;
        // Add the next block to the ledger.
        if let Err(error) = self.ledger.write().add_next_block(&next_block) {
            // Log the error.
            eprintln!("{error}");
        }
        // Return the next block.
        Ok(next_block)
    }

    /// Creates a transfer transaction.
    pub fn create_transfer(
        ledger: Arc<RwLock<InternalLedger<N>>>,
        private_key: &PrivateKey<N>,
        to: &Address<N>,
        amount: u64,
    ) -> Result<Transaction<N>> {
        // Derive the view key from the private key.
        let view_key = ViewKey::try_from(private_key)?;

        // Fetch the unspent record with the least gates, but enough for the transfer.
        let record = ledger
            .read()
            .find_records(&view_key, RecordsFilter::Unspent)?
            .filter(|(_, record)| (**record.gates()).ge(&U64::new(amount)))
            .min_by(|(_, a), (_, b)| (**a.gates()).cmp(&**b.gates()));

        // Prepare the record.
        let record = match record {
            Some((_, record)) => record,
            None => bail!("The Aleo account has no records to spend with sufficient balance."),
        };

        // Create a new transaction.
        Transaction::execute(
            ledger.read().vm(),
            private_key,
            &ProgramID::from_str("credits.aleo")?,
            Identifier::from_str("transfer")?,
            &[Value::Record(record), Value::from_str(&format!("{to}"))?, Value::from_str(&format!("{amount}u64"))?],
            None,
            &mut rand::thread_rng(),
        )
    }

    /// Creates a deploy transaction.
    fn create_deploy(
        ledger: Arc<RwLock<InternalLedger<N>>>,
        private_key: &PrivateKey<N>,
        program: &Program<N>,
        additional_fee: u64,
    ) -> Result<Transaction<N>> {
        // Construct the view key from the private key.
        let view_key = ViewKey::try_from(private_key)?;

        // Fetch the unspent record with the most gates.
        let record = ledger
            .read()
            .find_records(&view_key, RecordsFilter::Unspent)?
            .max_by(|(_, a), (_, b)| (**a.gates()).cmp(&**b.gates()));

        // Prepare the additional fee.
        let credits = match record {
            Some((_, record)) => record,
            None => bail!("The Aleo account has no records to spend."),
        };
        ensure!(***credits.gates() >= additional_fee, "The additional fee exceeds the record balance.");

        // Deploy.
        let transaction = Transaction::deploy(
            ledger.read().vm(),
            private_key,
            program,
            (credits, additional_fee),
            &mut rand::thread_rng(),
        )?;
        // Verify.
        assert!(ledger.read().vm().verify(&transaction));
        // Return the transaction.
        Ok(transaction)
    }
}
