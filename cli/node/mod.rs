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

// TODO: Cleanup and generalize.

pub mod consensus;
pub use consensus::*;

pub mod ledger;
pub use ledger::*;

pub mod pool;
pub use pool::*;

pub mod rest;
pub use rest::*;

pub mod routes;
pub use routes::*;

use snarkos_account::Account;
use snarkos_node_executor::{spawn_task_loop, Executor, NodeType, Status};
use snarkos_node_ledger::RecordMap;
use snarkos_node_store::ConsensusDB;
use snarkvm::prelude::{
    Address,
    Block,
    Identifier,
    Network,
    PrivateKey,
    ProgramID,
    ProverSolution,
    Transaction,
    Value,
    ViewKey,
    Zero,
};

use anyhow::{bail, Result};
use core::{str::FromStr, time::Duration};
use parking_lot::RwLock;
use snarkvm::synthesizer::ConsensusMemory;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use time::OffsetDateTime;
use tokio::time::timeout;

// TODO: Better name
/// A development beacon is an isolated full node, capable of producing blocks.
#[derive(Clone)]
pub struct DevelopmentBeacon<N: Network> {
    /// The account of the node.
    account: Account<N>,
    /// The consensus module of the node.
    consensus: SingleNodeConsensus<N, ConsensusMemory<N>>,
    /// The ledger of the node.
    ledger: Ledger<N, ConsensusMemory<N>>,
    /// The REST server of the node.
    rest: Option<Arc<Rest<N, ConsensusMemory<N>>>>,
    /// The time it to generate a block.
    block_generation_time: Arc<AtomicU64>,
    /// The unspent records.
    unspent_records: Arc<RwLock<RecordMap<N>>>,
    /// The shutdown signal.
    shutdown: Arc<AtomicBool>,
}

impl<N: Network> DevelopmentBeacon<N> {
    /// Initializes a new beacon node.
    pub async fn new(
        rest_ip: Option<SocketAddr>,
        private_key: PrivateKey<N>,
        genesis: Option<Block<N>>,
        dev: Option<u16>,
    ) -> Result<Self> {
        // Initialize the node account.
        let account = Account::from(private_key)?;
        // Initialize the ledger.
        let ledger = Ledger::load(genesis, dev)?;
        // Initialize the consensus.
        let consensus = SingleNodeConsensus::new(ledger.clone())?;
        // Initialize the REST server.
        let rest = match rest_ip {
            Some(rest_ip) => {
                Some(Arc::new(Rest::start(rest_ip, account.clone(), Some(consensus.clone()), ledger.clone())?))
            }
            None => None,
        };
        // Initialize the block generation time.
        let block_generation_time = Arc::new(AtomicU64::new(2));
        // Retrieve the unspent records.
        let unspent_records = ledger.find_unspent_records(account.view_key())?;
        // Initialize the node.
        let node = Self {
            account,
            consensus,
            ledger,
            rest,
            block_generation_time,
            unspent_records: Arc::new(RwLock::new(unspent_records)),
            shutdown: Default::default(),
        };
        // Initialize the block production.
        node.initialize_block_production().await;
        // Initialize the signal handler.
        node.handle_signals();
        // Return the node.
        Ok(node)
    }

    /// Returns the ledger.
    pub fn ledger(&self) -> &Ledger<N, ConsensusMemory<N>> {
        &self.ledger
    }

    /// Returns the REST server.
    pub fn rest(&self) -> &Option<Arc<Rest<N, ConsensusMemory<N>>>> {
        &self.rest
    }
}

#[async_trait]
impl<N: Network> Executor for DevelopmentBeacon<N> {
    /// The node type.
    const NODE_TYPE: NodeType = NodeType::Beacon;

    /// Disconnects from peers and shuts down the node.
    async fn shut_down(&self) {
        info!("Shutting down...");
        // Update the node status.
        Self::status().update(Status::ShuttingDown);

        // Shut down the ledger.
        trace!("Proceeding to shut down the ledger...");
        self.shutdown.store(true, Ordering::SeqCst);

        // Flush the tasks.
        Self::resources().shut_down();
        trace!("Node has shut down.");
    }
}

// Note: This is a modification on `NodeInterface`, which requires a router.
// TODO: Refactor.
impl<N: Network> DevelopmentBeacon<N> {
    /// Returns the node type.
    fn node_type(&self) -> NodeType {
        Self::NODE_TYPE
    }

    /// Returns the account private key of the node.
    fn private_key(&self) -> &PrivateKey<N> {
        self.account.private_key()
    }

    /// Returns the account view key of the node.
    fn view_key(&self) -> &ViewKey<N> {
        self.account.view_key()
    }

    /// Returns the account address of the node.
    fn address(&self) -> Address<N> {
        self.account.address()
    }
}

impl<N: Network> DevelopmentBeacon<N> {
    /// Initialize a new instance of block production.
    async fn initialize_block_production(&self) {
        let beacon = self.clone();
        spawn_task_loop!(Self, {
            // Expected time per block.
            const ROUND_TIME: u64 = 15; // 15 seconds per block

            // Produce blocks.
            loop {
                // Fetch the current timestamp.
                let current_timestamp = OffsetDateTime::now_utc().unix_timestamp();
                // Compute the elapsed time.
                let elapsed_time = current_timestamp.saturating_sub(beacon.ledger.latest_timestamp()) as u64;

                // Do not produce a block if the elapsed time has not exceeded `ROUND_TIME - block_generation_time`.
                // This will ensure a block is produced at intervals of approximately `ROUND_TIME`.
                let time_to_wait = ROUND_TIME.saturating_sub(beacon.block_generation_time.load(Ordering::SeqCst));
                trace!("Waiting for {time_to_wait} seconds before producing a block...");
                // TODO: More sophisticated block production.
                tokio::time::sleep(Duration::from_secs(time_to_wait)).await;

                // Start a timer.
                let timer = std::time::Instant::now();
                // Produce the next block and propagate it to all peers.
                match beacon.produce_next_block().await {
                    // Update the block generation time.
                    Ok(()) => beacon.block_generation_time.store(timer.elapsed().as_secs(), Ordering::SeqCst),
                    Err(error) => error!("{error}"),
                }

                // If the Ctrl-C handler registered the signal, stop the node once the current block is complete.
                if beacon.shutdown.load(Ordering::Relaxed) {
                    info!("Shutting down block production");
                    break;
                }
            }
        });
    }

    /// Produces the next block and propagates it to all peers.
    async fn produce_next_block(&self) -> Result<()> {
        let mut beacon_transaction: Option<Transaction<N>> = None;

        // Produce a transaction if the mempool is empty.
        if self.consensus.memory_pool().num_unconfirmed_transactions() == 0 {
            // Create a transfer transaction.
            let beacon = self.clone();
            let transaction = match tokio::task::spawn_blocking(move || {
                // Fetch an unspent record.
                let (commitment, record) = match beacon.unspent_records.write().shift_remove_index(0) {
                    Some(record) => record,
                    None => bail!("The beacon has no unspent records available"),
                };

                // Initialize an RNG.
                let rng = &mut rand::thread_rng();

                // Prepare the inputs.
                let to = beacon.account.address();
                let amount = 1;
                let inputs = [
                    Value::Record(record.clone()),
                    Value::from_str(&format!("{to}"))?,
                    Value::from_str(&format!("{amount}u64"))?,
                ];

                // Create a new transaction.
                let transaction = Transaction::execute(
                    beacon.ledger.vm(),
                    beacon.account.private_key(),
                    ProgramID::from_str("credits.aleo")?,
                    Identifier::from_str("transfer")?,
                    inputs.iter(),
                    None,
                    None,
                    rng,
                );

                match transaction {
                    Ok(transaction) => Ok(transaction),
                    Err(error) => {
                        // Push the record back into the unspent records.
                        beacon.unspent_records.write().insert(commitment, record);
                        bail!("Failed to create a transaction: {error}")
                    }
                }
            })
            .await
            {
                Ok(Ok(transaction)) => transaction,
                Ok(Err(error)) => bail!("Failed to create a transfer transaction for the next block: {error}"),
                Err(error) => bail!("Failed to create a transfer transaction for the next block: {error}"),
            };
            // Save the beacon transaction.
            beacon_transaction = Some(transaction.clone());

            // Add the transaction to the memory pool.
            let beacon = self.clone();
            match tokio::task::spawn_blocking(move || beacon.consensus.add_unconfirmed_transaction(transaction)).await {
                Ok(Ok(())) => (),
                Ok(Err(error)) => bail!("Failed to add the transaction to the memory pool: {error}"),
                Err(error) => bail!("Failed to add the transaction to the memory pool: {error}"),
            }
        }

        // Propose the next block.
        let beacon = self.clone();
        let next_block = match tokio::task::spawn_blocking(move || {
            let next_block = beacon.consensus.propose_next_block(beacon.private_key(), &mut rand::thread_rng())?;

            // Ensure the block is a valid next block.
            if let Err(error) = beacon.consensus.check_next_block(&next_block) {
                // Clear the memory pool of all solutions and transactions.
                trace!("Clearing the memory pool...");
                beacon.consensus.clear_memory_pool()?;
                trace!("Cleared the memory pool");
                bail!("Proposed an invalid block: {error}")
            }

            // Advance to the next block.
            match beacon.consensus.advance_to_next_block(&next_block) {
                Ok(()) => {
                    // If the beacon produced a transaction, save its output records.
                    if let Some(transaction) = beacon_transaction {
                        // Save the unspent records.
                        if let Err(error) = transaction.into_transitions().try_for_each(|transition| {
                            for (commitment, record) in transition.into_records() {
                                let record = record.decrypt(beacon.account.view_key())?;
                                if !record.gates().is_zero() {
                                    beacon.unspent_records.write().insert(commitment, record);
                                }
                            }
                            Ok::<_, anyhow::Error>(())
                        }) {
                            warn!("Unable to save the beacon unspent records, recomputing unspent records: {error}");
                            // Recompute the unspent records.
                            *beacon.unspent_records.write() =
                                beacon.ledger.find_unspent_records(beacon.account.view_key())?;
                        };
                    }
                    // Log the next block.
                    match serde_json::to_string_pretty(&next_block.header()) {
                        Ok(header) => info!("Block {}: {header}", next_block.height()),
                        Err(error) => info!("Block {}: (serde failed: {error})", next_block.height()),
                    }
                }
                Err(error) => {
                    // Clear the memory pool of all solutions and transactions.
                    trace!("Clearing the memory pool...");
                    beacon.consensus.clear_memory_pool()?;
                    trace!("Cleared the memory pool");
                    bail!("Failed to advance to the next block: {error}")
                }
            }

            Ok(next_block)
        })
        .await
        {
            Ok(Ok(next_block)) => next_block,
            Ok(Err(error)) => {
                // Sleep for one second.
                tokio::time::sleep(Duration::from_secs(1)).await;
                bail!("Failed to propose the next block: {error}")
            }
            Err(error) => {
                // Sleep for one second.
                tokio::time::sleep(Duration::from_secs(1)).await;
                bail!("Failed to propose the next block (JoinError): {error}")
            }
        };

        Ok(())
    }
}
