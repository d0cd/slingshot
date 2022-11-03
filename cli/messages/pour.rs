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

use anyhow::Result;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

pub struct PourRequest<N: Network> {
    address: Address<N>,
    amount: Value<N>,
}

impl<N: Network> PourRequest<N> {
    /// Initializes a new instance of a pour request.
    pub fn new(address: Address<N>, amount: Value<N>) -> Self {
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
    pub const fn amount(&self) -> &Value<N> {
        &self.amount
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
    address: Address<N>,
    balance: Value<N>,
}

impl<N: Network> PourResponse<N> {
    /// Initializes a new pour response.
    pub const fn new(address: Address<N>, balance: Value<N>) -> Self {
        Self { address, balance }
    }

    /// Returns the address that received credits from the faucet.
    pub const fn address(&self) -> &Address<N> {
        &self.address
    }

    /// Returns the balance of the address that received credits from the faucet.
    pub const fn balance(&self) -> &Value<N> {
        &self.balance
    }
}

impl<N: Network> Serialize for PourResponse<N> {
    /// Serializes the pour response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("PourResponse", 2)?;
        response.serialize_field("address", &self.address)?;
        response.serialize_field("balance", &self.balance)?;
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
            // Retrieve the address.
            serde_json::from_value(response["address"].take()).map_err(de::Error::custom)?,
            // Retrieve the balance.
            serde_json::from_value(response["balance"].take()).map_err(de::Error::custom)?,
        ))
    }
}
