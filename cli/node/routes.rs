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

use crate::node::{Ledger, Rest, SingleNodeConsensus};

use snarkos::node::{
    ledger::RecordsFilter,
    rest::{with, OrReject, RestError},
};

use snarkvm::prelude::{
    cfg_into_iter,
    Address,
    ConsensusStorage,
    Field,
    Network,
    PrivateKey,
    Program,
    ProgramID,
    ViewKey,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use warp::{http::StatusCode, reject, reply, Filter, Rejection, Reply};

use crate::messages::{
    DeployRequest,
    DeployResponse,
    ExecuteRequest,
    ExecuteResponse,
    PourRequest,
    PourResponse,
    RecordViewRequest,
    RecordViewResponse,
};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// The `get_blocks` query object.
#[derive(Deserialize, Serialize)]
struct BlockRange {
    /// The starting block height (inclusive).
    start: u32,
    /// The ending block height (exclusive).
    end: u32,
}

impl<N: Network, C: ConsensusStorage<N>> Rest<N, C> {
    /// Initializes the routes, given the ledger and ledger sender.
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // GET /testnet3/latest/height
        let latest_height = warp::get()
            .and(warp::path!("testnet3" / "latest" / "height"))
            .and(with(self.ledger.clone()))
            .and_then(Self::latest_height);

        // GET /testnet3/latest/hash
        let latest_hash = warp::get()
            .and(warp::path!("testnet3" / "latest" / "hash"))
            .and(with(self.ledger.clone()))
            .and_then(Self::latest_hash);

        // GET /testnet3/latest/block
        let latest_block = warp::get()
            .and(warp::path!("testnet3" / "latest" / "block"))
            .and(with(self.ledger.clone()))
            .and_then(Self::latest_block);

        // GET /testnet3/latest/stateRoot
        let latest_state_root = warp::get()
            .and(warp::path!("testnet3" / "latest" / "stateRoot"))
            .and(with(self.ledger.clone()))
            .and_then(Self::latest_state_root);

        // GET /testnet3/block/{height}
        let get_block = warp::get()
            .and(warp::path!("testnet3" / "block" / u32))
            .and(with(self.ledger.clone()))
            .and_then(Self::get_block);

        // GET /testnet3/blocks?start={start_height}&end={end_height}
        let get_blocks = warp::get()
            .and(warp::path!("testnet3" / "blocks"))
            .and(warp::query::<BlockRange>())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_blocks);

        // GET /testnet3/block/{blockHash}
        let get_block_by_hash = warp::get()
            .and(warp::path!("testnet3" / "block" / ..))
            .and(warp::path::param::<N::BlockHash>())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_block_by_hash);

        // GET /testnet3/height/{blockHash}
        let get_block_height_by_hash = warp::get()
            .and(warp::path!("testnet3" / "height" / ..))
            .and(warp::path::param::<N::BlockHash>())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_block_height_by_hash);

        // GET /testnet3/block/{height}/transactions
        let get_block_transactions = warp::get()
            .and(warp::path!("testnet3" / "block" / u32 / "transactions"))
            .and(with(self.ledger.clone()))
            .and_then(Self::get_block_transactions);

        // GET /testnet3/transaction/{transactionID}
        let get_transaction = warp::get()
            .and(warp::path!("testnet3" / "transaction" / ..))
            .and(warp::path::param::<N::TransactionID>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_transaction);

        // GET /testnet3/memoryPool/transactions
        let get_memory_pool_transactions = warp::get()
            .and(warp::path!("testnet3" / "memoryPool" / "transactions"))
            .and(with(self.consensus.clone()))
            .and_then(Self::get_memory_pool_transactions);

        // GET /testnet3/program/{programID}
        let get_program = warp::get()
            .and(warp::path!("testnet3" / "program" / ..))
            .and(warp::path::param::<ProgramID<N>>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_program);

        // GET /testnet3/statePath/{commitment}
        let get_state_path_for_commitment = warp::get()
            .and(warp::path!("testnet3" / "statePath" / ..))
            .and(warp::path::param::<Field<N>>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::get_state_path_for_commitment);

        // GET /testnet3/node/address
        let get_node_address = warp::get()
            .and(warp::path!("testnet3" / "node" / "address"))
            .and(with(self.account.address()))
            .and_then(|address: Address<N>| async move { Ok::<_, Rejection>(reply::json(&address.to_string())) });

        // GET /testnet3/find/blockHash/{transactionID}
        let find_block_hash = warp::get()
            .and(warp::path!("testnet3" / "find" / "blockHash" / ..))
            .and(warp::path::param::<N::TransactionID>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::find_block_hash);

        // GET /testnet3/find/deploymentID/{programID}
        let find_deployment_id = warp::get()
            .and(warp::path!("testnet3" / "find" / "deploymentID" / ..))
            .and(warp::path::param::<ProgramID<N>>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::find_deployment_id);

        // GET /testnet3/find/transactionID/{transitionID}
        let find_transaction_id = warp::get()
            .and(warp::path!("testnet3" / "find" / "transactionID" / ..))
            .and(warp::path::param::<N::TransitionID>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::find_transaction_id);

        // GET /testnet3/find/transitionID/{inputOrOutputID}
        let find_transition_id = warp::get()
            .and(warp::path!("testnet3" / "find" / "transitionID" / ..))
            .and(warp::path::param::<Field<N>>())
            .and(warp::path::end())
            .and(with(self.ledger.clone()))
            .and_then(Self::find_transition_id);

        // POST /testnet3/records/all
        let records_all = warp::post()
            .and(warp::path!("testnet3" / "records" / "all"))
            .and(warp::body::content_length_limit(128))
            .and(warp::body::json())
            .and(with(self.ledger.clone()))
            .and_then(Self::records_all);

        // POST /testnet3/records/spent
        let records_spent = warp::post()
            .and(warp::path!("testnet3" / "records" / "spent"))
            .and(warp::body::content_length_limit(128))
            .and(warp::body::json())
            .and(with(self.ledger.clone()))
            .and_then(Self::records_spent);

        // POST /testnet3/records/unspent
        let records_unspent = warp::post()
            .and(warp::path!("testnet3" / "records" / "unspent"))
            .and(warp::body::content_length_limit(128))
            .and(warp::body::json())
            .and(with(self.ledger.clone()))
            .and_then(Self::records_unspent);

        // POST /testnet3/faucet/pour
        let faucet_pour = warp::post()
            .and(warp::path!("testnet3" / "faucet" / "pour"))
            .and(warp::body::content_length_limit(128))
            .and(warp::body::json())
            .and(with(*self.account.private_key()))
            .and(with(self.ledger.clone()))
            .and(with(self.consensus.clone()))
            .and_then(Self::faucet_pour);

        // TODO: Faucet total.

        // Determine Content Length based on Input Size supported by the Network.
        let max_data_size = N::MAX_DATA_SIZE_IN_FIELDS * Field::<N>::SIZE_IN_DATA_BITS as u32;
        let max_data_inputs = N::MAX_DATA_DEPTH * N::MAX_DATA_ENTRIES * N::MAX_INPUTS;
        let max_content_length = (max_data_inputs as u32 * max_data_size) as u64;

        // POST /testnet3/program/deploy
        let program_deploy = warp::post()
            .and(warp::path!("testnet3" / "program" / "deploy"))
            .and(warp::body::content_length_limit(max_content_length))
            .and(warp::body::json())
            .and(with(self.ledger.clone()))
            .and(with(self.consensus.clone()))
            .and_then(Self::program_deploy);

        let program_execute = warp::post()
            .and(warp::path!("testnet3" / "program" / "execute"))
            .and(warp::body::content_length_limit(max_content_length))
            .and(warp::body::json())
            .and(with(self.ledger.clone()))
            .and(with(self.consensus.clone()))
            .and_then(Self::program_execute);

        // Return the list of routes.
        latest_height
            .or(latest_hash)
            .or(latest_block)
            .or(latest_state_root)
            .or(get_block)
            .or(get_blocks)
            .or(get_block_by_hash)
            .or(get_block_height_by_hash)
            .or(get_block_transactions)
            .or(get_transaction)
            .or(get_memory_pool_transactions)
            .or(get_program)
            .or(get_state_path_for_commitment)
            .or(get_node_address)
            .or(find_block_hash)
            .or(find_deployment_id)
            .or(find_transaction_id)
            .or(find_transition_id)
            .or(records_all)
            .or(records_spent)
            .or(records_unspent)
            .or(faucet_pour)
            .or(program_deploy)
            .or(program_execute)
    }
}

impl<N: Network, C: ConsensusStorage<N>> Rest<N, C> {
    /// Returns the latest block height.
    async fn latest_height(ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.latest_height()))
    }

    /// Returns the latest block hash.
    async fn latest_hash(ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.latest_hash()))
    }

    /// Returns the latest block.
    async fn latest_block(ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.latest_block()))
    }

    /// Returns the latest state root.
    async fn latest_state_root(ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.latest_state_root()))
    }

    /// Returns the block for the given block height.
    async fn get_block(height: u32, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_block(height).or_reject()?))
    }

    /// Returns the blocks for the given block range.
    async fn get_blocks(block_range: BlockRange, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        let start_height = block_range.start;
        let end_height = block_range.end;

        const MAX_BLOCK_RANGE: u32 = 50;

        // Ensure the end height is greater than the start height.
        if start_height > end_height {
            return Err(reject::custom(RestError::Request("Invalid block range".to_string())));
        }
        // Ensure the block range is bounded.
        else if end_height - start_height > MAX_BLOCK_RANGE {
            return Err(reject::custom(RestError::Request(format!(
                "Cannot request more than {MAX_BLOCK_RANGE} blocks per call (requested {})",
                end_height - start_height
            ))));
        }

        let blocks = cfg_into_iter!((start_height..end_height))
            .map(|height| ledger.get_block(height).or_reject())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(reply::json(&blocks))
    }

    /// Returns the block for the given block hash.
    async fn get_block_by_hash(hash: N::BlockHash, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_block_by_hash(&hash).or_reject()?))
    }

    /// Returns the block height for the given block hash.
    async fn get_block_height_by_hash(hash: N::BlockHash, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_height(&hash).or_reject()?))
    }

    /// Returns the transactions for the given block height.
    async fn get_block_transactions(height: u32, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_transactions(height).or_reject()?))
    }

    /// Returns the transaction for the given transaction ID.
    async fn get_transaction(transaction_id: N::TransactionID, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_transaction(transaction_id).or_reject()?))
    }

    /// Returns the transactions in the memory pool.
    async fn get_memory_pool_transactions(
        consensus: Option<SingleNodeConsensus<N, C>>,
    ) -> Result<impl Reply, Rejection> {
        match consensus {
            Some(consensus) => Ok(reply::json(&consensus.memory_pool().unconfirmed_transactions())),
            None => Err(reject::custom(RestError::Request("Invalid endpoint".to_string()))),
        }
    }

    /// Returns the program for the given program ID.
    async fn get_program(program_id: ProgramID<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        let program = if program_id == ProgramID::<N>::from_str("credits.aleo").or_reject()? {
            Program::<N>::credits().or_reject()?
        } else {
            ledger.get_program(program_id).or_reject()?
        };

        Ok(reply::json(&program))
    }

    /// Returns the state path for the given commitment.
    async fn get_state_path_for_commitment(
        commitment: Field<N>,
        ledger: Ledger<N, C>,
    ) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.get_state_path_for_commitment(&commitment).or_reject()?))
    }

    /// Returns the block hash that contains the given `transaction ID`.
    async fn find_block_hash(transaction_id: N::TransactionID, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.find_block_hash(&transaction_id).or_reject()?))
    }

    /// Returns the transaction ID that contains the given `program ID`.
    async fn find_deployment_id(program_id: ProgramID<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.find_deployment_id(&program_id).or_reject()?))
    }

    /// Returns the transaction ID that contains the given `transition ID`.
    async fn find_transaction_id(
        transition_id: N::TransitionID,
        ledger: Ledger<N, C>,
    ) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.find_transaction_id(&transition_id).or_reject()?))
    }

    /// Returns the transition ID that contains the given `input ID` or `output ID`.
    async fn find_transition_id(input_or_output_id: Field<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        Ok(reply::json(&ledger.find_transition_id(&input_or_output_id).or_reject()?))
    }

    /// Returns all of the records for the given view key.
    async fn records_all(request: RecordViewRequest<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        // Fetch the records using the view key.
        let records: IndexMap<_, _> =
            ledger.find_records(request.view_key(), RecordsFilter::All).or_reject()?.collect();
        // Return the records.
        Ok(reply::with_status(RecordViewResponse::new(records), StatusCode::OK))
    }

    /// Returns the spent records for the given view key.
    async fn records_spent(request: RecordViewRequest<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        // Fetch the records using the view key.
        let records =
            ledger.find_records(request.view_key(), RecordsFilter::Spent).or_reject()?.collect::<IndexMap<_, _>>();
        // Return the records.
        Ok(reply::with_status(RecordViewResponse::new(records), StatusCode::OK))
    }

    /// Returns the unspent records for the given view key.
    async fn records_unspent(request: RecordViewRequest<N>, ledger: Ledger<N, C>) -> Result<impl Reply, Rejection> {
        // Fetch the records using the view key.
        let records =
            ledger.find_records(request.view_key(), RecordsFilter::Unspent).or_reject()?.collect::<IndexMap<_, _>>();
        // Return the records.
        Ok(reply::with_status(RecordViewResponse::new(records), StatusCode::OK))
    }

    /// Pours a specified number of credits from the faucet to the recipient.
    async fn faucet_pour(
        request: PourRequest<N>,
        private_key: PrivateKey<N>,
        ledger: Ledger<N, C>,
        consensus: Option<SingleNodeConsensus<N, C>>,
    ) -> Result<impl Reply, Rejection> {
        // Construct the transaction.
        let transaction = match Ledger::create_transfer(&ledger, &private_key, *request.address(), request.amount()) {
            Ok(transaction) => transaction,
            Err(_) => {
                return Err(reject::custom(RestError::Request(String::from("failed to construct the transaction"))));
            }
        };

        // Construct the response.
        let response = PourResponse::<N>::new(transaction.id());

        // Add the transaction to the memory pool.
        match consensus {
            Some(consensus) => match consensus.add_unconfirmed_transaction(transaction) {
                Ok(_) => Ok(response),
                Err(_) => Err(reject::custom(RestError::Request(String::from(
                    "failed to add the transaction to the memory pool",
                )))),
            },
            None => Err(reject::custom(RestError::Request(String::from("no memory pool available")))),
        }
    }

    /// Deploys a program to the ledger.
    async fn program_deploy(
        request: DeployRequest<N>,
        ledger: Ledger<N, C>,
        consensus: Option<SingleNodeConsensus<N, C>>,
    ) -> Result<impl Reply, Rejection> {
        // Construct the transaction.
        let transaction =
            match Ledger::create_deploy(&ledger, request.private_key(), request.program(), request.additional_fee()) {
                Ok(transaction) => transaction,
                Err(_) => {
                    return Err(reject::custom(RestError::Request(String::from(
                        "failed to construct the transaction",
                    ))));
                }
            };

        // Construct the response.
        let response = DeployResponse::<N>::new(transaction.id());

        // Add the transaction to the memory pool.
        match consensus {
            Some(consensus) => match consensus.add_unconfirmed_transaction(transaction) {
                Ok(_) => Ok(response),
                Err(_) => Err(reject::custom(RestError::Request(String::from(
                    "failed to add the transaction to the memory pool",
                )))),
            },
            None => Err(reject::custom(RestError::Request(String::from("no memory pool available")))),
        }
    }

    /// Executes a program on the ledger.
    async fn program_execute(
        request: ExecuteRequest<N>,
        ledger: Ledger<N, C>,
        consensus: Option<SingleNodeConsensus<N, C>>,
    ) -> Result<impl Reply, Rejection> {
        // Construct the transaction.
        let transaction = match Ledger::create_execute(
            &ledger,
            request.private_key(),
            request.program_id(),
            request.function_name(),
            request.inputs(),
            request.additional_fee(),
        ) {
            Ok(transaction) => transaction,
            Err(_) => {
                return Err(reject::custom(RestError::Request(String::from("failed to construct the transaction"))));
            }
        };

        // Construct the response.
        let response = ExecuteResponse::<N>::new(transaction.id());

        // Add the transaction to the memory pool.
        match consensus {
            Some(consensus) => match consensus.add_unconfirmed_transaction(transaction) {
                Ok(_) => Ok(response),
                Err(_) => Err(reject::custom(RestError::Request(String::from(
                    "failed to add the transaction to the memory pool",
                )))),
            },
            None => Err(reject::custom(RestError::Request(String::from("no memory pool available")))),
        }
    }
}
