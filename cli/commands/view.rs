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
        /// Return only the spent records.
        #[clap(short, long, conflicts_with = "unspent")]
        spent: bool,
        /// Return only the unspent records.
        #[clap(short, long, conflicts_with = "spent")]
        unspent: bool,
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
            Self::Record { key, path, spent, unspent, endpoint } => {
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

                // Get the record filter.
                let filter = match (spent, unspent) {
                    (true, true) => unreachable!("Clap prevents conflicting options from being enabled"),
                    (true, false) => "spent",
                    (false, true) => "unspent",
                    (false, false) => "all",
                };

                // Use the provided endpoint, or default to a local endpoints.
                let endpoint = match endpoint {
                    Some(endpoint) => endpoint,
                    None => format!("http://localhost:4180/testnet3/records/{filter}"),
                };

                // Construct the request.
                let account = Account::<Network>::try_from(&private_key)?;
                let request = RecordViewRequest::new(*account.view_key());

                // Send the request and wait for the response.
                match request.send(&endpoint) {
                    Ok(response) => {
                        let mut message = match (spent, unspent) {
                            (false, false) => format!(
                                "✅ Found {} record(s) for the account {}.\n\n",
                                response.records().len(),
                                account.address()
                            ),
                            _ => format!(
                                "✅ Found {} {} record(s) for the account {}.\n\n",
                                response.records().len(),
                                filter,
                                account.address()
                            ),
                        };
                        for (commitment, record) in response.records().iter() {
                            message.push_str(&format!("Commitment: {commitment}\nRecord: {record}\n\n"));
                        }
                        Ok(message)
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }
}
