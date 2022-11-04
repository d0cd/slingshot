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

use crate::{Aleo, Network};
use snarkvm::package::Package;

use anyhow::{ensure, Result};
use clap::Parser;
use colored::Colorize;
use snarkvm::{
    file::{AleoFile, Manifest},
    prelude::Transaction,
};

/// Deploys an Aleo program.
#[derive(Debug, Parser)]
pub struct Deploy {
    /// The endpoint to deploy to. Defaults to a local development node at "http://localhost:4180/testnet3/transaction/broadcast".
    #[clap(short, long, default_value = "http://localhost:4180/testnet3/transaction/broadcast")]
    pub endpoint: String,
    // TODO: Optional path
}

impl Deploy {
    /// Deploys an Aleo program with the specified name.
    pub fn parse(self) -> Result<String> {
        todo!()
    }
}
