// RGB standard library for working with smart contracts on Bitcoin & Lightning
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2023 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2023 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bp::bc::stl::bitcoin_stl;
use strict_types::stl::std_stl;
use strict_types::typelib::{LibBuilder, TranslateError};
use strict_types::typesys::SystemBuilder;
use strict_types::{SemId, TypeLib, TypeSystem};

use super::{
    DivisibleAssetSpec, Error, MediaRegName, MediaType, RicardianContract, Timestamp,
    LIB_NAME_RGB_CONTRACT,
};
use crate::stl::ProofOfReserves;

/// Strict types id for the library providing standard data types which may be
/// used in RGB smart contracts.
pub const LIB_ID_RGB_CONTRACT: &str =
    "ricardo_cherry_protein_EcdesrJ9woezrhLGfgG6rnaJots2R9Acz5k38HDdDn2t";

fn _rgb_contract_stl() -> Result<TypeLib, TranslateError> {
    LibBuilder::new(libname!(LIB_NAME_RGB_CONTRACT))
        .transpile::<Timestamp>()
        .transpile::<DivisibleAssetSpec>()
        .transpile::<RicardianContract>()
        .transpile::<MediaType>()
        .transpile::<MediaRegName>()
        .transpile::<ProofOfReserves>()
        .compile(bset! {
            std_stl().to_dependency(),
            bitcoin_stl().to_dependency()
        })
}

/// Generates strict type library providing standard data types which may be
/// used in RGB smart contracts.
pub fn rgb_contract_stl() -> TypeLib {
    _rgb_contract_stl().expect("invalid strict type RGBContract library")
}

#[derive(Debug)]
pub struct StandardTypes(TypeSystem);

impl StandardTypes {
    pub fn new() -> Self {
        Self::try_with([std_stl(), bitcoin_stl(), rgb_contract_stl()])
            .expect("error in standard RGBContract type system")
    }

    pub fn with(lib: TypeLib) -> Self {
        Self::try_with([std_stl(), bitcoin_stl(), rgb_contract_stl(), lib])
            .expect("error in standard RGBContract type system")
    }

    fn try_with(libs: impl IntoIterator<Item = TypeLib>) -> Result<Self, Error> {
        let mut builder = SystemBuilder::new();
        for lib in libs.into_iter() {
            builder = builder.import(lib)?;
        }
        let sys = builder.finalize()?;
        Ok(Self(sys))
    }

    pub fn type_system(&self) -> TypeSystem { self.0.clone() }

    pub fn get(&self, name: &'static str) -> SemId {
        self.0.id_by_name(name).unwrap_or_else(|| {
            panic!("type '{name}' is absent in standard RGBContract type library")
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lib_id() {
        let lib = rgb_contract_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_RGB_CONTRACT);
    }
}
