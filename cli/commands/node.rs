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

use crate::{ledger::Ledger, Network};

use snarkvm::file::Manifest;

use anyhow::{bail, ensure, Result};
use clap::Parser;
use colored::*;
use snarkvm::prelude::PrivateKey;
use std::{path::PathBuf, str::FromStr, sync::Arc};

/// Commands to operate a local development node.
#[derive(Debug, Parser)]
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
    pub fn parse(self) -> Result<String> {
        match self {
            Self::Start { key, path } => {
                let private_key = match (key, path) {
                    (Some(_), Some(_)) => unreachable!("clap prevents conflicting options from being enabled"),
                    (None, None) => bail!("Please specify either a private key or a manifest file"),
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

                println!("⏳ Starting a local development node (in-memory)...\n",);

                // Initialize the ledger.
                let ledger = Arc::new(Ledger::<Network>::load(&private_key)?);

                loop {
                    // Create a transfer transaction.
                    let transaction =
                        Ledger::create_transfer(ledger.ledger.clone(), ledger.private_key(), ledger.address(), 1)?;
                    // Add the transaction to the memory pool.
                    ledger.add_to_memory_pool(transaction)?;

                    // Advance to the next block.
                    let next_block = ledger.advance_to_next_block()?;
                    println!(
                        "\n🛡️  Produced block {} ({})\n\n{}\n",
                        next_block.height(),
                        next_block.hash(),
                        serde_json::to_string_pretty(&next_block.header())?.dimmed()
                    );
                }
            }
        }
    }
}
