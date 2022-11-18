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

#![forbid(unsafe_code)]

use crate::node::TransactionPool;

use snarkos_node_consensus::{coinbase_target, proof_target};
use snarkos_node_ledger::Ledger;

use snarkvm::prelude::*;

use anyhow::{anyhow, Result};
use rayon::iter::ParallelIterator;
use time::OffsetDateTime;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Clone)]
pub struct SingleNodeConsensus<N: Network, C: ConsensusStorage<N>> {
    /// The ledger.
    ledger: Ledger<N, C>,
    /// The memory pool.
    memory_pool: TransactionPool<N>,
}

impl<N: Network, C: ConsensusStorage<N>> SingleNodeConsensus<N, C> {
    /// Initializes a new instance of consensus.
    pub fn new(ledger: Ledger<N, C>) -> Result<Self> {
        // Initialize consensus.
        Ok(Self { ledger, memory_pool: Default::default() })
    }

    /// Returns the memory pool.
    pub const fn memory_pool(&self) -> &TransactionPool<N> {
        &self.memory_pool
    }

    /// Adds the given unconfirmed transaction to the memory pool.
    pub fn add_unconfirmed_transaction(&self, transaction: Transaction<N>) -> Result<()> {
        // Ensure the transaction is not already in the memory pool.
        if self.memory_pool.contains_unconfirmed_transaction(transaction.id()) {
            bail!("Transaction is already in the memory pool.");
        }
        // Check that the transaction is well-formed and unique.
        self.check_transaction_basic(&transaction)?;
        // Insert the transaction to the memory pool.
        self.memory_pool.add_unconfirmed_transaction(&transaction);

        Ok(())
    }

    /// Returns a candidate for the next block in the ledger.
    pub fn propose_next_block<R: Rng + CryptoRng>(&self, private_key: &PrivateKey<N>, rng: &mut R) -> Result<Block<N>> {
        // Retrieve the latest state root.
        let latest_state_root = self.ledger.latest_state_root();
        // Retrieve the latest block.
        let latest_block = self.ledger.latest_block();
        // Retrieve the latest height.
        let latest_height = latest_block.height();

        // Select the transactions from the memory pool.
        let transactions = self.memory_pool.candidate_transactions(self).into_iter().collect::<Transactions<N>>();

        // Construct the coinbase solution.
        let coinbase = None;
        let coinbase_accumulator_point = Field::<N>::zero();

        // Fetch the next round state.
        let next_timestamp = OffsetDateTime::now_utc().unix_timestamp();
        let next_height = latest_height.saturating_add(1);
        let next_round = latest_block.round().saturating_add(1);

        // Construct the next coinbase target.
        let next_coinbase_target = coinbase_target(
            latest_block.last_coinbase_target(),
            latest_block.last_coinbase_timestamp(),
            next_timestamp,
            N::ANCHOR_TIME,
            N::NUM_BLOCKS_PER_EPOCH,
        )?;

        // Construct the next proof target.
        let next_proof_target = proof_target(next_coinbase_target);

        // Construct the next last coinbase target and next last coinbase timestamp.
        let next_last_coinbase_target = latest_block.last_coinbase_target();
        let next_last_coinbase_timestamp = latest_block.last_coinbase_timestamp();

        // Construct the metadata.
        let metadata = Metadata::new(
            N::ID,
            next_round,
            next_height,
            next_coinbase_target,
            next_proof_target,
            next_last_coinbase_target,
            next_last_coinbase_timestamp,
            next_timestamp,
        )?;

        // Construct the header.
        let header = Header::from(latest_state_root, transactions.to_root()?, coinbase_accumulator_point, metadata)?;

        // Construct the new block.
        Block::new(private_key, latest_block.hash(), header, transactions, coinbase, rng)
    }

    /// Advances the ledger to the next block.
    pub fn advance_to_next_block(&self, block: &Block<N>) -> Result<()> {
        // Adds the next block to the ledger.
        self.ledger.add_next_block(block)?;

        // Clear the memory pool of unconfirmed transactions that are now invalid.
        self.memory_pool.clear_invalid_transactions(self);

        Ok(())
    }

    /// Clears the memory pool of invalid solutions and transactions.
    pub fn refresh_memory_pool(&self) -> Result<()> {
        // Clear the memory pool of unconfirmed transactions that are now invalid.
        self.memory_pool.clear_invalid_transactions(self);
        Ok(())
    }

    /// Clears the memory pool of all solutions and transactions.
    pub fn clear_memory_pool(&self) -> Result<()> {
        // Clear the memory pool of unconfirmed transactions that are now invalid.
        self.memory_pool.clear_unconfirmed_transactions();
        Ok(())
    }

    /// Checks the given block is valid next block.
    pub fn check_next_block(&self, block: &Block<N>) -> Result<()> {
        // Ensure the previous block hash is correct.
        if self.ledger.latest_hash() != block.previous_hash() {
            bail!("The next block has an incorrect previous block hash")
        }

        // Ensure the block hash does not already exist.
        if self.ledger.contains_block_hash(&block.hash())? {
            bail!("Block hash '{}' already exists in the ledger", block.hash())
        }

        // Ensure the next block height is correct.
        if self.ledger.latest_height() > 0 && self.ledger.latest_height() + 1 != block.height() {
            bail!("The next block has an incorrect block height")
        }

        // Ensure the block height does not already exist.
        if self.ledger.contains_block_height(block.height())? {
            bail!("Block height '{}' already exists in the ledger", block.height())
        }

        // TODO (raychu86): Ensure the next round number includes timeouts.
        // Ensure the next round is correct.
        if self.ledger.latest_round() > 0
            && self.ledger.latest_round() + 1 /*+ block.number_of_timeouts()*/ != block.round()
        {
            bail!("The next block has an incorrect round number")
        }

        // TODO (raychu86): Ensure the next block timestamp is the median of proposed blocks.
        // Ensure the next block timestamp is after the current block timestamp.
        if block.height() > 0 {
            let next_timestamp = block.header().timestamp();
            let latest_timestamp = self.ledger.latest_block().header().timestamp();
            if next_timestamp <= latest_timestamp {
                bail!("The next block timestamp {next_timestamp} is before the current timestamp {latest_timestamp}")
            }
        }

        for transaction_id in block.transaction_ids() {
            // Ensure the transaction in the block do not already exist.
            if self.ledger.contains_transaction_id(transaction_id)? {
                bail!("Transaction '{transaction_id}' already exists in the ledger")
            }
        }

        /* Input */

        // Ensure the ledger does not already contain a given serial numbers.
        for serial_number in block.serial_numbers() {
            if self.ledger.contains_serial_number(serial_number)? {
                bail!("Serial number '{serial_number}' already exists in the ledger")
            }
        }

        /* Output */

        // Ensure the ledger does not already contain a given commitments.
        for commitment in block.commitments() {
            if self.ledger.contains_commitment(commitment)? {
                bail!("Commitment '{commitment}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given nonces.
        for nonce in block.nonces() {
            if self.ledger.contains_nonce(nonce)? {
                bail!("Nonce '{nonce}' already exists in the ledger")
            }
        }

        /* Metadata */

        // Ensure the ledger does not already contain a given transition public keys.
        for tpk in block.transition_public_keys() {
            if self.ledger.contains_tpk(tpk)? {
                bail!("Transition public key '{tpk}' already exists in the ledger")
            }
        }

        /* Block Header */

        // If the block is the genesis block, check that it is valid.
        if block.height() == 0 && !block.is_genesis() {
            bail!("Invalid genesis block");
        }

        // Ensure the block header is valid.
        if !block.header().is_valid() {
            bail!("Invalid block header: {:?}", block.header());
        }

        // Check the last coinbase members in the block.
        if block.height() > 0 {
            match block.coinbase() {
                Some(_) => {
                    // Ensure the last coinbase target matches the coinbase target.
                    if block.last_coinbase_target() != block.coinbase_target() {
                        bail!("The last coinbase target does not match the coinbase target")
                    }
                    // Ensure the last coinbase timestamp matches the block timestamp.
                    if block.last_coinbase_timestamp() != block.timestamp() {
                        bail!("The last coinbase timestamp does not match the block timestamp")
                    }
                }
                None => {
                    // Ensure the last coinbase target matches the previous block coinbase target.
                    if block.last_coinbase_target() != self.ledger.last_coinbase_target() {
                        bail!("The last coinbase target does not match the previous block coinbase target")
                    }
                    // Ensure the last coinbase timestamp matches the previous block's last coinbase timestamp.
                    if block.last_coinbase_timestamp() != self.ledger.last_coinbase_timestamp() {
                        bail!("The last coinbase timestamp does not match the previous block's last coinbase timestamp")
                    }
                }
            }
        }

        // Ensure the coinbase target is correct.
        let expected_coinbase_target = coinbase_target(
            self.ledger.last_coinbase_target(),
            self.ledger.last_coinbase_timestamp(),
            block.timestamp(),
            N::ANCHOR_TIME,
            N::NUM_BLOCKS_PER_EPOCH,
        )?;
        if block.coinbase_target() != expected_coinbase_target {
            bail!("Invalid coinbase target: expected {}, got {}", expected_coinbase_target, block.coinbase_target())
        }

        // Ensure the proof target is correct.
        let expected_proof_target = proof_target(expected_coinbase_target);
        if block.proof_target() != expected_proof_target {
            bail!("Invalid proof target: expected {}, got {}", expected_proof_target, block.proof_target())
        }

        /* Block Hash */

        // Compute the Merkle root of the block header.
        let header_root = match block.header().to_root() {
            Ok(root) => root,
            Err(error) => bail!("Failed to compute the Merkle root of the block header: {error}"),
        };

        // Check the block hash.
        match N::hash_bhp1024(&[block.previous_hash().to_bits_le(), header_root.to_bits_le()].concat()) {
            Ok(candidate_hash) => {
                // Ensure the block hash matches the one in the block.
                if candidate_hash != *block.hash() {
                    bail!("Block {} ({}) has an incorrect block hash.", block.height(), block.hash());
                }
            }
            Err(error) => {
                bail!("Unable to compute block hash for block {} ({}): {error}", block.height(), block.hash())
            }
        };

        /* Signature */
        // Check the signature.
        let signer = block.signature().to_address();
        if !block.signature().verify(&signer, &[*block.hash()]) {
            bail!("Invalid signature for block {} ({})", block.height(), block.hash());
        }

        /* Transactions */

        // Compute the transactions root.
        match block.transactions().to_root() {
            // Ensure the transactions root matches the one in the block header.
            Ok(root) => {
                if root != block.header().transactions_root() {
                    bail!(
                        "Block {} ({}) has an incorrect transactions root: expected {}",
                        block.height(),
                        block.hash(),
                        block.header().transactions_root()
                    );
                }
            }
            Err(error) => bail!("Failed to compute the Merkle root of the block transactions: {error}"),
        };

        // Ensure the transactions list is not empty.
        if block.transactions().is_empty() {
            bail!("Cannot validate an empty transactions list");
        }

        // Ensure the number of transactions is within the allowed range.
        if block.transactions().len() > Transactions::<N>::MAX_TRANSACTIONS {
            bail!("Cannot validate a block with more than {} transactions", Transactions::<N>::MAX_TRANSACTIONS);
        }

        // Ensure each transaction is well-formed and unique.
        cfg_iter!(block.transactions()).try_for_each(|(_, transaction)| {
            self.check_transaction_basic(transaction)
                .map_err(|e| anyhow!("Invalid transaction found in the transactions list: {e}"))
        })?;

        /* Coinbase Proof */

        // Ensure the coinbase solution is valid, if it exists.
        if block.coinbase().is_some() {
            bail!("`SingleNodeConsensus` does not accept blocks with coinbase solutions");
        } else {
            // Ensure that the block header does not contain a coinbase accumulator point.
            if block.header().coinbase_accumulator_point() != Field::<N>::zero() {
                bail!("Coinbase accumulator point should be zero as there is no coinbase solution in the block.");
            }
        }

        Ok(())
    }

    /// Checks the given transaction is well-formed and unique.
    pub fn check_transaction_basic(&self, transaction: &Transaction<N>) -> Result<()> {
        let transaction_id = transaction.id();

        // Ensure the ledger does not already contain the given transaction ID.
        if self.ledger.contains_transaction_id(&transaction_id)? {
            bail!("Transaction '{transaction_id}' already exists in the ledger")
        }

        /* Fee */

        // Ensure transactions with a positive balance must pay for its storage in bytes.
        let fee = transaction.fee()?;
        if fee >= 0 && transaction.to_bytes_le()?.len() < usize::try_from(fee)? {
            bail!("Transaction '{transaction_id}' has insufficient fee to cover its storage in bytes")
        }

        /* Proof(s) */

        // Ensure the transaction is valid.
        if !self.ledger.vm().verify(transaction) {
            bail!("Transaction '{transaction_id}' is invalid")
        }

        /* Input */

        // Ensure the ledger does not already contain the given input ID.
        for input_id in transaction.input_ids() {
            if self.ledger.contains_input_id(input_id)? {
                bail!("Input ID '{input_id}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given serial numbers.
        for serial_number in transaction.serial_numbers() {
            if self.ledger.contains_serial_number(serial_number)? {
                bail!("Serial number '{serial_number}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given tag.
        for tag in transaction.tags() {
            if self.ledger.contains_tag(tag)? {
                bail!("Tag '{tag}' already exists in the ledger")
            }
        }

        /* Output */

        // Ensure the ledger does not already contain the given output ID.
        for output_id in transaction.output_ids() {
            if self.ledger.contains_output_id(output_id)? {
                bail!("Output ID '{output_id}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given commitments.
        for commitment in transaction.commitments() {
            if self.ledger.contains_commitment(commitment)? {
                bail!("Commitment '{commitment}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given nonces.
        for nonce in transaction.nonces() {
            if self.ledger.contains_nonce(nonce)? {
                bail!("Nonce '{nonce}' already exists in the ledger")
            }
        }

        /* Program */

        // Ensure that the ledger does not already contain the given program ID.
        if let Transaction::Deploy(_, deployment, _) = &transaction {
            let program_id = deployment.program_id();
            if self.ledger.contains_program_id(program_id)? {
                bail!("Program ID '{program_id}' already exists in the ledger")
            }
        }

        /* Metadata */

        // Ensure the ledger does not already contain a given transition public keys.
        for tpk in transaction.transition_public_keys() {
            if self.ledger.contains_tpk(tpk)? {
                bail!("Transition public key '{tpk}' already exists in the ledger")
            }
        }

        // Ensure the ledger does not already contain a given transition commitment.
        for tcm in transaction.transition_commitments() {
            if self.ledger.contains_tcm(tcm)? {
                bail!("Transition commitment '{tcm}' already exists in the ledger")
            }
        }

        Ok(())
    }
}
