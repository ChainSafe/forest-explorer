// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;
use fvm_shared::address::{Address, Network};

#[derive(Clone)]
pub struct AddressLotusJson(Address);

impl HasLotusJson for Address {
    type LotusJson = AddressLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        AddressLotusJson(self)
    }

    fn from_lotus_json(AddressLotusJson(address): Self::LotusJson) -> Self {
        address
    }
}

fn parse_address(s: &str) -> anyhow::Result<Address> {
    Ok(Network::Testnet
        .parse_address(s)
        .or_else(|_| Network::Mainnet.parse_address(s))?)
}

impl Serialize for AddressLotusJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AddressLotusJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let address_str = String::deserialize(deserializer)?;
        parse_address(&address_str)
            .map(AddressLotusJson)
            .map_err(serde::de::Error::custom)
    }
}
