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

use crate::{messages::ExecuteRequest, Network};

use snarkos::account::Account;

use snarkvm::prelude::{Address, Identifier, Locator, Value};

use anyhow::{ensure, Result};
use clap::Parser;
use colored::Colorize;
use core::str::FromStr;
use snarkvm::{file::Manifest, prelude::ProgramID};
use std::path::PathBuf;

/// Executes an Aleo program function on a development node.
#[derive(Debug, Parser)]
pub struct Execute {
    /// The program identifier.
    #[clap(parse(try_from_str))]
    program: ProgramID<Network>,
    /// The function name.
    #[clap(parse(try_from_str))]
    function: Identifier<Network>,
    /// The function inputs.
    #[clap(parse(try_from_str))]
    inputs: Vec<Value<Network>>,

    /// The additional fee.
    #[clap(short, long)]
    pub fee: Option<u64>,
    /// The endpoint to deploy to. Defaults to a local development node.
    #[clap(short, long)]
    pub endpoint: Option<String>,
    /// A path to a directory containing a manifest file. Defaults to the current working directory.
    #[clap(short, long)]
    pub path: Option<String>,
}

impl Execute {
    /// Executes an Aleo program function with the provided inputs.
    #[allow(clippy::format_in_format_args)]
    pub fn parse(self) -> Result<String> {
        // Setup the endpoint.
        let endpoint = self.endpoint.unwrap_or_else(|| "http://localhost:4180/testnet3/program/execute".to_string());

        // Instantiate a path to the directory containing the manifest file.
        let directory = match self.path {
            Some(path) => PathBuf::from_str(&path)?,
            None => std::env::current_dir()?,
        };

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
        let manifest = Manifest::<Network>::open(&directory)?;

        // Retrieve the private key.
        let private_key = manifest.development_private_key();

        // Create the execute request.
        let request = ExecuteRequest::new(*private_key, self.program, self.function, self.inputs, self.fee);

        // TODO: Log outputs
        // Log the outputs.
        //match response.outputs().len() {
        //    0 => (),
        //    1 => println!("\n➡️  Output\n"),
        //    _ => println!("\n➡️  Outputs\n"),
        //};
        //for output in response.outputs() {
        //    println!("{}", format!(" • {output}"));
        //}
        //println!();

        // Send the request and wait for the response.
        match request.send(&endpoint) {
            // TODO: Just send tx id?
            Ok(_) => {
                // Prepare the locator.
                let locator = Locator::<Network>::from_str(&format!("{}/{}", self.program, self.function))?;
                Ok(format!("✅ Executed '{}'", locator.to_string().bold()))
            }
            Err(error) => Err(error),
        }
    }
}
