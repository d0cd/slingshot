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

use crate::{
    messages::{PourRequest, RecordViewRequest},
    node::DevelopmentBeacon,
};
use anyhow::{bail, ensure, Result};
use clap::Parser;
use colored::*;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use snarkos::account::Account;
use snarkvm::prelude::{Block, ConsensusMemory, ConsensusStore, PrivateKey, VM};
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};
use tokio::{runtime, runtime::Runtime};

// TODO: Quiet option
// TODO: Rethink CLI interface

/// Commands to query the local development node.
#[derive(Clone, Debug, Parser)]
pub enum View {
    /// Queries the local development node for records associated with the account.
    Record {
        /// A private key.
        #[clap(short, long, conflicts_with = "manifest_file")]
        key: Option<String>,
        /// A path to a directory containing a manifest file.
        #[clap(short, long, conflicts_with = "private_key")]
        path: Option<String>,
        /// Uses the specified endpoint.
        #[clap(short, long)]
        endpoint: Option<String>,
    },
}

impl View {
    #[allow(unused_must_use)]
    pub fn parse(self) -> Result<String> {
        match self {
            // Parse the command and get the private key.
            Self::Record { key, path, endpoint } => {
                let private_key = match (key, path) {
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
                };

                // Use the provided endpoint, or default to a local faucet.
                let endpoint = match endpoint {
                    Some(endpoint) => endpoint,
                    None => "http://localhost:4180/testnet3/records/all".to_string(),
                };

                // Construct the request.
                let account = Account::<Network>::try_from(&private_key)?;
                let request = RecordViewRequest::new(*account.view_key());

                // Send the request and wait for the response.
                match request.send(&endpoint) {
                    // TODO: Just send tx id?
                    Ok(response) => {
                        let mut response =
                            format!("✅ Found the following records for the account {}.", account.address());
                        todo!("Print the records");
                        Ok(response)
                    }
                    Err(error) => Err(error),
                }
            }
        };

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
