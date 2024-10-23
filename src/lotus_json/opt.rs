// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;

// TODO(forest): https://github.com/ChainSafe/forest/issues/4032
//               Remove this - users should use `Option<LotusJson<T>>` instead
//               of LotusJson<Option<T>>
impl<T> HasLotusJson for Option<T>
where
    T: HasLotusJson,
{
    type LotusJson = Option<T::LotusJson>;

    fn into_lotus_json(self) -> Self::LotusJson {
        self.map(T::into_lotus_json)
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        lotus_json.map(T::from_lotus_json)
    }
}
