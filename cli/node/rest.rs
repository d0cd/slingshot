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

use crate::node::{Ledger, SingleNodeConsensus};

use snarkos_account::Account;
use snarkvm::{console::account::Address, prelude::Network, synthesizer::ConsensusStorage};

use anyhow::Result;
use colored::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::task::JoinHandle;
use warp::{http::header::HeaderName, Filter};

/// A REST API server for the ledger.
#[derive(Clone)]
pub struct Rest<N: Network, C: ConsensusStorage<N>> {
    /// The node account.
    pub(crate) account: Account<N>,
    /// The consensus module.
    pub(crate) consensus: Option<SingleNodeConsensus<N, C>>,
    /// The ledger.
    pub(crate) ledger: Ledger<N, C>,
    /// The server handles.
    pub(crate) handles: Vec<Arc<JoinHandle<()>>>,
}

impl<N: Network, C: 'static + ConsensusStorage<N>> Rest<N, C> {
    /// Initializes a new instance of the server.
    pub fn start(
        rest_ip: SocketAddr,
        account: Account<N>,
        consensus: Option<SingleNodeConsensus<N, C>>,
        ledger: Ledger<N, C>,
    ) -> Result<Self> {
        // Initialize the server.
        let mut server = Self { account, consensus, ledger, handles: vec![] };
        // Spawn the server.
        server.spawn_server(rest_ip);
        // Return the server.
        Ok(server)
    }
}

impl<N: Network, C: ConsensusStorage<N>> Rest<N, C> {
    /// Returns the ledger.
    pub const fn ledger(&self) -> &Ledger<N, C> {
        &self.ledger
    }

    /// Returns the handles.
    pub const fn handles(&self) -> &Vec<Arc<JoinHandle<()>>> {
        &self.handles
    }
}

impl<N: Network, C: 'static + ConsensusStorage<N>> Rest<N, C> {
    /// Initializes the server.
    fn spawn_server(&mut self, rest_ip: SocketAddr) {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_header(HeaderName::from_static("content-type"))
            .allow_methods(vec!["GET", "POST", "OPTIONS"]);

        // Initialize the routes.
        let routes = self.routes();

        // Add custom logging for each request.
        let custom_log = warp::log::custom(|info| match info.remote_addr() {
            Some(addr) => debug!("Received '{} {}' from '{addr}' ({})", info.method(), info.path(), info.status()),
            None => debug!("Received '{} {}' ({})", info.method(), info.path(), info.status()),
        });

        // Spawn the server.
        self.handles.push(Arc::new(tokio::spawn(async move {
            println!("🌐 Starting the REST server at {}.\n", rest_ip.to_string().bold());

            // Start the server.
            warp::serve(routes.with(cors).with(custom_log)).run(rest_ip).await
        })))
    }
}
