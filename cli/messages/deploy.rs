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

use snarkvm::prelude::{Network, PrivateKey, Program};

use anyhow::Result;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use warp::{reply::Response, Reply};

pub struct DeployRequest<N: Network> {
    private_key: PrivateKey<N>,
    program: Program<N>,
    additional_fee: u64,
}

impl<N: Network> DeployRequest<N> {
    /// Initializes a new instance of the deploy request.
    pub fn new(private_key: PrivateKey<N>, program: Program<N>, additional_fee: u64) -> Self {
        Self { private_key, program, additional_fee }
    }

    /// Sends the request to the given endpoint.
    pub fn send(&self, endpoint: &str) -> Result<DeployResponse<N>> {
        Ok(ureq::post(endpoint).send_json(self)?.into_json()?)
    }

    /// Returns the private key of the account deploying the program.
    pub const fn private_key(&self) -> &PrivateKey<N> {
        &self.private_key
    }

    /// Returns the program to be deployed.
    pub const fn program(&self) -> &Program<N> {
        &self.program
    }

    /// Returns the additional fee associated with the request.
    pub const fn additional_fee(&self) -> u64 {
        self.additional_fee
    }
}

impl<N: Network> Serialize for DeployRequest<N> {
    /// Serializes the deploy request into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut request = serializer.serialize_struct("DeployRequest", 3)?;
        // Serialize the private_key.
        request.serialize_field("private_key", &self.private_key)?;
        // Serialize the program.
        request.serialize_field("program", &self.program)?;
        // Serialize the additional_fee.
        request.serialize_field("additional_fee", &self.additional_fee)?;
        request.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for DeployRequest<N> {
    /// Deserializes the deploy request from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the request from a string into a value.
        let mut request = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(
            // Retrieve the private_key.
            serde_json::from_value(request["private_key"].take()).map_err(de::Error::custom)?,
            // Retrieve the program.
            serde_json::from_value(request["program"].take()).map_err(de::Error::custom)?,
            // Retrieve the additional_fee.
            serde_json::from_value(request["additional_fee"].take()).map_err(de::Error::custom)?,
        ))
    }
}

pub struct DeployResponse<N: Network> {
    transaction_id: N::TransactionID,
}

impl<N: Network> DeployResponse<N> {
    /// Initializes a new deploy response.
    pub const fn new(transaction_id: N::TransactionID) -> Self {
        Self { transaction_id }
    }

    /// Returns the associated deployment.
    pub const fn transaction_id(&self) -> &N::TransactionID {
        &self.transaction_id
    }
}

impl<N: Network> Serialize for DeployResponse<N> {
    /// Serializes the deploy response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("DeployResponse", 1)?;
        response.serialize_field("transaction_id", &self.transaction_id)?;
        response.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for DeployResponse<N> {
    /// Deserializes the deploy response from a string or bytes.
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

impl<N: Network> Reply for DeployResponse<N> {
    fn into_response(self) -> Response {
        warp::reply::json(&self).into_response()
    }
}
