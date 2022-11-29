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

#![forbid(unsafe_code)]
#![recursion_limit = "256"]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate tracing;

pub mod commands;
pub mod errors;
pub mod helpers;
// pub mod ledger;
pub mod messages;
pub mod node;

pub(crate) type Network = snarkvm::prelude::Testnet3;
pub(crate) type _Aleo = snarkvm::circuit::AleoV0;
