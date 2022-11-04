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

use crate::ledger::{InternalLedger, InternalServer, InternalStorage, Ledger};

use snarkvm::prelude::{with, Address, Block, Network, PrivateKey, ProgramStore, ViewKey, VM};

use anyhow::Result;
use parking_lot::RwLock;
use std::{convert::TryFrom, sync::Arc};
use warp::{reply, Filter, Rejection};

impl<N: Network> Ledger<N> {
    /// Initializes a new instance of the ledger.
    pub fn load(private_key: &PrivateKey<N>) -> Result<Arc<Self>> {
        // Derive the view key and address.
        let view_key = ViewKey::try_from(private_key)?;
        let address = Address::try_from(&view_key)?;

        // Initialize an RNG.
        let rng = &mut ::rand::thread_rng();
        // Initialize the store.
        let store = ProgramStore::<_, InternalStorage<_>>::open(None)?;
        // Create a genesis block.
        let genesis = Block::genesis(&VM::new(store)?, private_key, rng)?;

        // Initialize the ledger.
        let ledger = Arc::new(RwLock::new(InternalLedger::new_with_genesis(&genesis, address, None)?));

        // Initialize the additional routes.
        let additional_routes = {
            // GET /testnet3/development/privateKey
            let get_development_private_key = warp::get()
                .and(warp::path!("testnet3" / "development" / "privateKey"))
                .and(snarkvm::rest::with(*private_key))
                .and_then(|private_key: PrivateKey<N>| async move {
                    Ok::<_, Rejection>(reply::json(&private_key.to_string()))
                });

            // GET /testnet3/development/viewKey
            let get_development_view_key = warp::get()
                .and(warp::path!("testnet3" / "development" / "viewKey"))
                .and(snarkvm::rest::with(view_key))
                .and_then(|view_key: ViewKey<N>| async move { Ok::<_, Rejection>(reply::json(&view_key.to_string())) });

            // GET /testnet3/development/address
            let get_development_address = warp::get()
                .and(warp::path!("testnet3" / "development" / "address"))
                .and(snarkvm::rest::with(address))
                .and_then(|address: Address<N>| async move { Ok::<_, Rejection>(reply::json(&address.to_string())) });

            // POST /testnet3/faucet/pour
            let faucet_pour = warp::post()
                .and(warp::path!("testnet3" / "faucet" / "pour"))
                .and(warp::body::json())
                .and(with(ledger.clone()))
                .and(with(*private_key))
                .and_then(Self::faucet_pour);

            // POST /testnet3/program/deploy
            let program_deploy = warp::post()
                .and(warp::path!("testnet3" / "program" / "deploy"))
                .and(warp::body::json())
                .and(with(ledger.clone()))
                .and_then(Self::program_deploy);

            // POST /testnet3/program/execute
            let program_execute = warp::post()
                .and(warp::path!("testnet3" / "program" / "execute"))
                .and(warp::body::json())
                .and(with(ledger.clone()))
                .and_then(Self::program_execute);

            get_development_private_key
                .or(get_development_view_key)
                .or(get_development_address)
                .or(faucet_pour)
                .or(program_deploy)
                .or(program_execute)
        };

        // Initialize a runtime.
        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(8 * 1024 * 1024).build()?;

        // Initialize the server.
        let ledger_clone = ledger.clone();
        let server = runtime.block_on(async move {
            // Start the server.
            InternalServer::<N>::start(ledger_clone, Some(additional_routes), Some(4180))
        })?;

        // Return the ledger.
        Ok(Arc::new(Self { ledger, runtime, server, private_key: *private_key, view_key, address }))
    }
}
