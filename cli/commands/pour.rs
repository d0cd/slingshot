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

use snarkvm::prelude::{Address, Locator, Value};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

/// Pours Aleo credits into an account.
#[derive(Debug, Parser)]
pub struct Pour {
    /// The function name.
    #[clap(parse(try_from_str))]
    address: Address<Network>,
    /// The amount to send.
    #[clap(parse(try_from_str))]
    input: Value<Network>,
    /// Uses the specified endpoint.
    #[clap(long, default_value = "http://localhost:4180/testnet3/faucet/pour")]
    endpoint: String,
}

impl Pour {
    /// Pours a specified number of Aleo credits into an address.
    #[allow(clippy::format_in_format_args)]
    pub fn parse(self) -> Result<String> {
        todo!();
        // Ok(format!("✅ Executed '{}' {}", locator.to_string().bold(), path_string.dimmed()))
    }
}
