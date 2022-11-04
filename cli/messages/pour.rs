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

use snarkvm::prelude::{Address, Network, Value};

use crate::commands::Pour;
use anyhow::Result;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use warp::{reply::Response, Reply};

pub struct PourRequest<N: Network> {
    address: Address<N>,
    amount: u64,
}

impl<N: Network> PourRequest<N> {
    /// Initializes a new instance of a pour request.
    pub fn new(address: Address<N>, amount: u64) -> Self {
        Self { address, amount }
    }

    /// Sends the request to the given endpoint.
    pub fn send(&self, endpoint: &str) -> Result<PourResponse<N>> {
        Ok(ureq::post(endpoint).send_json(self)?.into_json()?)
    }

    /// Returns the recipient address.
    pub const fn address(&self) -> &Address<N> {
        &self.address
    }

    /// Returns the amount to be received.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}

impl<N: Network> Serialize for PourRequest<N> {
    /// Serializes the pour request into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut request = serializer.serialize_struct("PourRequest", 2)?;
        // Serialize the address.
        request.serialize_field("address", &self.address)?;
        // Serialize the amount.
        request.serialize_field("amount", &self.amount)?;
        request.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for PourRequest<N> {
    /// Deserializes the pour request from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the request from a string into a value.
        let mut request = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(
            // Retrieve the address.
            serde_json::from_value(request["address"].take()).map_err(de::Error::custom)?,
            // Retrieve the amount.
            serde_json::from_value(request["amount"].take()).map_err(de::Error::custom)?,
        ))
    }
}

pub struct PourResponse<N: Network> {
    transaction_id: N::TransactionID,
}

impl<N: Network> PourResponse<N> {
    /// Initializes a new pour response.
    pub const fn new(transaction_id: N::TransactionID) -> Self {
        Self { transaction_id }
    }

    /// Returns the transaction ID associated with the pour request.
    pub const fn transaction_id(&self) -> &N::TransactionID {
        &self.transaction_id
    }
}

impl<N: Network> Serialize for PourResponse<N> {
    /// Serializes the pour response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("PourResponse", 1)?;
        response.serialize_field("transaction_id", &self.transaction_id)?;
        response.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for PourResponse<N> {
    /// Deserializes the pour response from a string or bytes.
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

impl<N: Network> Reply for PourResponse<N> {
    fn into_response(self) -> Response {
        warp::reply::json(&self).into_response()
    }
}
