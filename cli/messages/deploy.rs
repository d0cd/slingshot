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

use snarkvm::{
    compiler::{Deployment, Program},
    prelude::{Address, Network},
};

use anyhow::Result;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

pub struct DeployRequest<N: Network> {
    caller: Address<N>,
    program: Program<N>,
}

impl<N: Network> DeployRequest<N> {
    /// Initializes a new instance of the deploy request.
    pub fn new(caller: Address<N>, program: Program<N>) -> Self {
        Self { caller, program }
    }

    /// Sends the request to the given endpoint.
    pub fn send(&self, endpoint: &str) -> Result<DeployResponse<N>> {
        Ok(ureq::post(endpoint).send_json(self)?.into_json()?)
    }

    /// Returns the program address.
    pub const fn caller(&self) -> &Address<N> {
        &self.caller
    }

    /// Returns the imports.
    pub const fn program(&self) -> &Program<N> {
        &self.program
    }
}

impl<N: Network> Serialize for DeployRequest<N> {
    /// Serializes the deploy request into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut request = serializer.serialize_struct("DeployRequest", 2)?;
        // Serialize the caller.
        request.serialize_field("caller", &self.caller)?;
        // Serialize the program.
        request.serialize_field("program", &self.program)?;
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
            // Retrieve the caller.
            serde_json::from_value(request["caller"].take()).map_err(de::Error::custom)?,
            // Retrieve the program.
            serde_json::from_value(request["program"].take()).map_err(de::Error::custom)?,
        ))
    }
}

pub struct DeployResponse<N: Network> {
    deployment: Deployment<N>,
}

impl<N: Network> DeployResponse<N> {
    /// Initializes a new deploy response.
    pub const fn new(deployment: Deployment<N>) -> Self {
        Self { deployment }
    }

    /// Returns the associated deployment.
    pub const fn deployment(&self) -> &Deployment<N> {
        &self.deployment
    }
}

impl<N: Network> Serialize for DeployResponse<N> {
    /// Serializes the deploy response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("DeployResponse", 1)?;
        response.serialize_field("deployment", &self.deployment)?;
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
            // Retrieve the program ID.
            serde_json::from_value(response["deployment"].take()).map_err(de::Error::custom)?,
        ))
    }
}
