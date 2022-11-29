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

use snarkvm::prelude::{Field, Network, Plaintext, PrivateKey, Program, Record, ViewKey, Visibility};

use anyhow::{bail, Result};
use indexmap::IndexMap;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use warp::{hyper::body::HttpBody, reply::Response, Reply};

pub struct RecordViewRequest<N: Network> {
    view_key: ViewKey<N>,
}

impl<N: Network> RecordViewRequest<N> {
    /// Initializes a new instance of the view record request.
    pub fn new(view_key: ViewKey<N>) -> Self {
        Self { view_key }
    }

    /// Sends the request to the given endpoint.
    pub fn send(&self, endpoint: &str) -> Result<RecordViewResponse<N>> {
        Ok(ureq::post(endpoint).send_json(self)?.into_json()?)
    }

    /// Gets the view key associated with the request.
    pub fn view_key(&self) -> &ViewKey<N> {
        &self.view_key
    }
}

impl<N: Network> Serialize for RecordViewRequest<N> {
    /// Serializes the view request into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut request = serializer.serialize_struct("RecordViewRequest", 1)?;
        // Serialize the view_key.
        request.serialize_field("view_key", &self.view_key)?;
        request.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for RecordViewRequest<N> {
    /// Deserializes the view request from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the record view request from a string into a value.
        let mut request = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(serde_json::from_value(request["view_key"].take()).map_err(de::Error::custom)?))
    }
}

pub struct RecordViewResponse<N: Network> {
    records: IndexMap<Field<N>, Record<N, Plaintext<N>>>,
}

impl<N: Network> RecordViewResponse<N> {
    /// Initializes a new record view response.
    pub const fn new(records: IndexMap<Field<N>, Record<N, Plaintext<N>>>) -> Self {
        Self { records }
    }

    /// Returns the associated records.
    pub fn records(&self) -> &IndexMap<Field<N>, Record<N, Plaintext<N>>> {
        &self.records
    }
}

impl<N: Network> Serialize for RecordViewResponse<N> {
    /// Serializes the record view response into string or bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut response = serializer.serialize_struct("RecordViewResponse", 1)?;
        response.serialize_field("records", &self.records)?;
        response.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for RecordViewResponse<N> {
    /// Deserializes the record view response from a string or bytes.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Parse the response from a string into a value.
        let mut response = serde_json::Value::deserialize(deserializer)?;
        // Recover the leaf.
        Ok(Self::new(
            // Retrieve the transaction_id.
            serde_json::from_value(response["records"].take()).map_err(de::Error::custom)?,
        ))
    }
}

impl<N: Network> Reply for RecordViewResponse<N> {
    fn into_response(self) -> Response {
        warp::reply::json(&self).into_response()
    }
}
