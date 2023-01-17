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

use crate::{messages::DeployRequest, Network};

use snarkvm::{
    file::{AleoFile, Manifest},
    package::Package,
    prelude::ProgramID,
};

use anyhow::{bail, ensure, Result};
use clap::Parser;
use colored::Colorize;
use std::{path::PathBuf, str::FromStr};

// TODO: Prettify

/// Deploys an Aleo program.
#[derive(Debug, Parser)]
pub struct Deploy {
    /// The name of the program to deploy.
    #[clap(parse(try_from_str))]
    pub program: ProgramID<Network>,
    /// The deployment fee in gates.
    #[clap(short, long, help = "The deployment fee in gates, defaults to 0.")]
    pub fee: Option<u64>,
    /// The endpoint to deploy to. Defaults to a local development node.
    #[clap(short, long)]
    pub endpoint: Option<String>,
    /// A path to a directory containing a manifest file. Defaults to the current working directory.
    #[clap(short, long)]
    pub path: Option<String>,
}

impl Deploy {
    /// Deploys an Aleo program with the specified name.
    pub fn parse(self) -> Result<String> {
        // Setup the endpoint.
        let endpoint = self.endpoint.unwrap_or_else(|| "http://localhost:4180/testnet3/program/deploy".to_string());

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

        // Load the package.
        let package = Package::open(&directory)?;

        // Load the main program.
        let program = package.program();

        // Prepare the imports directory.
        let imports_directory = package.imports_directory();

        // Find the program that is being deployed.
        let program = match program.imports().keys().find(|program_id| **program_id == self.program) {
            Some(program_id) => {
                let file = AleoFile::open(&imports_directory, program_id, false)?;
                file.program().clone()
            }
            None => match self.program == *program.id() {
                true => program.clone(),
                false => bail!("The program '{}' does not exist in {}", self.program, directory.display()),
            },
        };

        let program_id = program.id().clone();
        println!("📦 Deploying '{}' to the local development node...\n", &program_id.to_string().bold());

        // Create a deployment request.
        let request = DeployRequest::new(*private_key, program, self.fee.unwrap_or(0));

        // Send the deployment request to the local development node.
        match request.send(&endpoint) {
            Ok(_) => println!("✅ Successfully deployed '{}' to the local development node.", program_id),
            Err(error) => {
                match error.downcast::<ureq::Error>() {
                    Ok(ureq::Error::Status(code, response)) => {
                        bail!(
                            "❌ Failed to deploy '{}' to the local development node: {} {:?}",
                            &program_id,
                            code,
                            response.into_string()
                        );
                    }
                    Ok(ureq::Error::Transport(error)) => {
                        bail!("❌ Failed to deploy '{}' to the local development node: {}", &program_id, error);
                    }
                    _ => {}
                }
                bail!("❌ Failed to deploy '{}' to the local development node", &program_id);
            }
        };

        Ok("".to_string())
    }
}
