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

use crate::Network;

use snarkvm::file::Manifest;

use crate::node::DevelopmentBeacon;
use anyhow::{bail, ensure, Result};
use clap::Parser;
use colored::*;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use snarkvm::prelude::{Block, ConsensusMemory, ConsensusStore, PrivateKey, VM};
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};
use tokio::{runtime, runtime::Runtime};

// TODO: Quiet option
// TODO: Rethink CLI interface

/// Commands to operate a local development node.
#[derive(Clone, Debug, Parser)]
pub enum Node {
    /// Starts a local development node.
    Start {
        /// A private key.
        #[clap(short, long, conflicts_with = "manifest_file")]
        key: Option<String>,
        /// A path to a directory containing a manifest file.
        #[clap(short, long, conflicts_with = "private_key")]
        path: Option<String>,
    },
}

impl Node {
    #[allow(unused_must_use)]
    pub fn parse(self) -> Result<String> {
        // Parse the command and get the private key.
        let private_key = match self {
            Self::Start { key, path } => match (key, path) {
                (Some(_), Some(_)) => unreachable!("Clap prevents conflicting options from being enabled"),
                (None, None) => panic!("Please specify either a private key or a manifest file"),
                (Some(key), None) => PrivateKey::<Network>::from_str(&key)?,
                (None, Some(path)) => {
                    // Instantiate a path to the directory containing the manifest file.
                    let directory = PathBuf::from_str(&path)?;
                    // Ensure the directory path exists.
                    ensure!(directory.exists(), "The program directory does not exist: {}", directory.display());
                    // Ensure the manifest file exists.
                    ensure!(
                        Manifest::<Network>::exists_at(&directory),
                        "Please ensure that the manifest file exists in the Aleo program directory (missing '{}' at '{}')",
                        Manifest::<Network>::file_name(),
                        directory.display()
                    );

                    // Open the manifest file.
                    let manifest = Manifest::open(&directory)?;

                    *manifest.development_private_key()
                }
            },
        };

        // Construct the REST IP address.
        // TODO: Input via CLI
        let rest_ip = Some(SocketAddr::from_str("127.0.0.1:4180")?);

        // Initialize an (insecure) fixed RNG.
        // TODO: Input via CLI
        let mut rng = ChaChaRng::seed_from_u64(1234567890u64);

        println!("⏳ Starting a local development node (in-memory)...\n",);

        // Initialize the runtime.
        Self::runtime().block_on(async move {
            // Initialize the consensus store.
            let store = ConsensusStore::<Network, ConsensusMemory<Network>>::open(None)
                .expect("Failed to initialize the consensus store");

            // Initialize a new VM.
            let vm = VM::from(store).expect("Failed to initialize the VM");

            // Initialize the genesis block.
            println!("⏳ Initializing the genesis block...");
            let genesis = Some(
                Block::<Network>::genesis(&vm, &private_key, &mut rng).expect("Failed to initialize the genesis block"),
            );
            println!();

            // Start the development node.
            DevelopmentBeacon::new(rest_ip, private_key, genesis, None)
                .await
                .expect("Failed to start the development node");
            // Note: Do not move this. The pending await must be here otherwise
            // other slingshot commands will not exit.
            std::future::pending::<()>().await;
        });

        Ok(String::new())
    }

    /// Returns a runtime for the node.
    fn runtime() -> Runtime {
        // TODO: This should be supplied by a config file. Think infrastruct as code tool.
        // let (num_tokio_worker_threads, max_tokio_blocking_threads, num_rayon_cores_global) = if !Self::node_type().is_beacon() {
        //     ((num_cpus::get() / 8 * 2).max(1), num_cpus::get(), (num_cpus::get() / 8 * 5).max(1))
        // } else {
        //     (num_cpus::get(), 512, num_cpus::get()) // 512 is tokio's current default
        // };
        let (num_tokio_worker_threads, max_tokio_blocking_threads, num_rayon_cores_global) =
            // { ((num_cpus::get() / 2).max(1), num_cpus::get(), (num_cpus::get() / 4 * 3).max(1)) };
            // { (num_cpus::get().min(8), 512, num_cpus::get().saturating_sub(8).max(1)) };
            { (1, 512, 4) };

        // Initialize the parallelization parameters.
        rayon::ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(num_rayon_cores_global)
            .build_global()
            .unwrap();

        // Initialize the runtime configuration.
        runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(8 * 1024 * 1024)
            .worker_threads(num_tokio_worker_threads)
            .max_blocking_threads(max_tokio_blocking_threads)
            .build()
            .expect("Failed to initialize a runtime for the router")
    }
}
