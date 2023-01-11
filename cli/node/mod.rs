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

use snarkos::{
    account::Account,
    node::{ledger::RecordMap, messages::NodeType, NodeInterface},
};

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
use snarkvm::synthesizer::{ConsensusMemory, ConsensusStorage};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use time::OffsetDateTime;
use tokio::{task::JoinHandle, time::timeout};

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
    /// The spawned handles.
    handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
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
        let account = Account::try_from(private_key)?;
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
            handles: Default::default(),
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

// Note: We cannot use `NodeInterface` directly, since it requires satisfying the trait bound Routing<N>.
// TODO: Refactor.
impl<N: Network> DevelopmentBeacon<N> {
    /// Returns the node type.
    fn node_type(&self) -> NodeType {
        NodeType::Beacon
    }

    /// Returns the account private key of the node.
    pub fn private_key(&self) -> &PrivateKey<N> {
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

    /// Returns `true` if the node is in development mode.
    /// Note that the development beacon is always in development mode.
    fn is_dev(&self) -> bool {
        true
    }

    /// Handles OS signals for the node to intercept and perform a clean shutdown.
    /// Note: Only Ctrl-C is supported; it should work on both Unix-family systems and Windows.
    pub fn handle_signals(&self) {
        let node = self.clone();
        tokio::task::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    node.shut_down().await;
                    std::process::exit(0);
                }
                Err(error) => error!("tokio::signal::ctrl_c encountered an error: {}", error),
            }
        });
    }

    /// Shuts down the node.
    async fn shut_down(&self) {
        info!("Shutting down...");

        // Shut down block production.
        trace!("Shutting down block production...");
        self.shutdown.store(true, Ordering::SeqCst);

        // Abort the tasks.
        trace!("Shutting down the beacon...");
        self.handles.read().iter().for_each(|handle| handle.abort());

        // Shut down the ledger.
        trace!("Shutting down the ledger...");
        // self.ledger.shut_down().await;

        info!("Node has shut down.");
    }
}

impl<N: Network> DevelopmentBeacon<N> {
    /// Initialize a new instance of block production.
    async fn initialize_block_production(&self) {
        let beacon = self.clone();
        self.handles.write().push(tokio::spawn(async move {
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
        }));
    }

    /// Produces the next block and propagates it to all peers.
    // TODO: This implementation only produces a block if there is are pending transactions.
    //   Eventially, we should parameterize this so that users can spin up devnets to their liking.
    async fn produce_next_block(&self) -> Result<()> {
        // Produce a transaction if the mempool is empty.
        if self.consensus.memory_pool().num_unconfirmed_transactions() == 0 {
            // If there are no unconfirmed transactions, then there is no need to do anything.
            return Ok(());
        }

        // Propose the next block.
        let beacon = self.clone();
        match tokio::task::spawn_blocking(move || {
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
