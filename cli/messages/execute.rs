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

use crate::messages::{PourRequest, PourResponse};

use snarkvm::prelude::{Address, Network, ProgramID, Value};

use anyhow::Result;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use warp::{reply::Response, Reply};

pub struct ExecuteRequest<N: Network> {
    program_id: ProgramID<N>,
}

impl<N: Network> ExecuteRequest<N> {
    /// Initializes a new instance of a execute request.
    pub fn new(program_id: ProgramID<N>) -> Self {
        Self { program_id }
    }

    /// Sends the request to the given endpoint.
    pub fn send(&self, endpoint: &str) -> Result<ExecuteResponse<N>> {
        Ok(ureq::post(endpoint).send_json(self)?.into_json()?)
    }

    /// Returns the program_id.
    pub const fn program_id(&self) -> &ProgramID<N> {
        &self.program_id
    }
}

impl<N: Network> Serialize for ExecuteRequest<N> {
    /// Serializes the execute request into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut request = serializer.serialize_struct("ExecuteRequest", 2)?;
        // Serialize the program_id.
        request.serialize_field("program_id", &self.program_id)?;
        request.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for ExecuteRequest<N> {
    /// Deserializes the execute request from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the request from a string into a value.
        let mut request = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(
            // Retrieve the program_id.
            serde_json::from_value(request["program_id"].take()).map_err(de::Error::custom)?,
        ))
    }
}

pub struct ExecuteResponse<N: Network> {
    transaction_id: N::TransactionID,
}

impl<N: Network> ExecuteResponse<N> {
    /// Initializes a new execute response.
    pub const fn new(transaction_id: N::TransactionID) -> Self {
        Self { transaction_id }
    }

    /// Returns the transaction ID associated with the exeucte request.
    pub const fn transaction_id(&self) -> &N::TransactionID {
        &self.transaction_id
    }
}

impl<N: Network> Serialize for ExecuteResponse<N> {
    /// Serializes the execute response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("ExecuteResponse", 1)?;
        response.serialize_field("transaction_id", &self.transaction_id)?;
        response.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for ExecuteResponse<N> {
    /// Deserializes the execute response from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the response from a string into a value.
        let mut response = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(
            // Retrieve the transaction_id.
            serde_json::from_value(response["transaction_id"].take()).map_err(de::Error::custom)?,
        ))
    }
}

impl<N: Network> Reply for ExecuteResponse<N> {
    /// Converts the execute response into a response.
    fn into_response(self) -> Response {
        warp::reply::json(&self).into_response()
    }
}
