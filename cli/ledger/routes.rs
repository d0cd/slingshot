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

use crate::{
    ledger::{InternalLedger, Ledger},
    messages::{DeployRequest, ExecuteRequest, PourRequest},
};

use snarkvm::prelude::{BlockStorage, Network, ProgramStorage, Transaction};

use parking_lot::RwLock;
use std::sync::Arc;
use warp::{Rejection, Reply};

impl<N: Network> Ledger<N> {
    /// Pours a specified number of credits from the faucet to the recipient.
    pub(crate) async fn faucet_pour(
        request: PourRequest<N>,
        ledger: Arc<RwLock<InternalLedger<N>>>,
    ) -> Result<impl Reply, Rejection> {
        let transaction = ledger.cre
        todo!()
    }

    /// Deploys a program to the ledger.
    pub(crate) async fn program_deploy(
        request: DeployRequest<N>,
        ledger: Arc<RwLock<InternalLedger<N>>>,
    ) -> Result<impl Reply, Rejection> {
        todo!()
    }

    /// Executes a program on the ledger.
    pub(crate) async fn program_execute(
        request: ExecuteRequest<N>,
        ledger: Arc<RwLock<InternalLedger<N>>>,
    ) -> Result<impl Reply, Rejection> {
        todo!()
    }
}
