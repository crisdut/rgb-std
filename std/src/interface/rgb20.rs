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

use amplify::confinement::LargeVec;
use bp::bc::stl::bp_tx_stl;
use strict_types::{CompileError, LibBuilder, TypeLib};

use crate::interface::contract::OutpointFilter;
use crate::interface::{
    ArgSpec, AssignIface, ContractIface, FungibleAllocation, GenesisIface, GlobalIface, Iface,
    OwnedIface, Req, TransitionIface, VerNo,
};
use crate::stl::{
    rgb_contract_stl, Amount, ContractData, DivisibleAssetSpec, StandardTypes, Timestamp,
};

pub const LIB_NAME_RGB20: &str = "RGB20";
/// Strict types id for the library providing data types for RGB20 interface.
pub const LIB_ID_RGB20: &str =
    "urn:ubideco:stl:GVz4mvYE94aQ9q2HPtV9VuoppcDdduP54BMKffF7YoFH#prince-scarlet-ringo";

const SUPPLY_MISMATCH: u8 = 1;
const NON_EQUAL_AMOUNTS: u8 = 2;
const INVALID_PROOF: u8 = 3;
const INSUFFICIENT_RESERVES: u8 = 4;
const INSUFFICIENT_COVERAGE: u8 = 5;
const ISSUE_EXCEEDS_ALLOWANCE: u8 = 6;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB20, tags = repr, into_u8, try_from_u8)]
#[repr(u8)]
pub enum Error {
    #[strict_type(dumb)]
    SupplyMismatch = SUPPLY_MISMATCH,
    NonEqualAmounts = NON_EQUAL_AMOUNTS,
    InvalidProof = INVALID_PROOF,
    InsufficientReserves = INSUFFICIENT_RESERVES,
    InsufficientCoverage = INSUFFICIENT_COVERAGE,
    IssueExceedsAllowance = ISSUE_EXCEEDS_ALLOWANCE,
}

fn _rgb20_stl() -> Result<TypeLib, CompileError> {
    LibBuilder::new(libname!(LIB_NAME_RGB20), tiny_bset! {
        bp_tx_stl().to_dependency(),
        rgb_contract_stl().to_dependency()
    })
    .transpile::<Error>()
    .compile()
}

/// Generates strict type library providing data types for RGB20 interface.
pub fn rgb20_stl() -> TypeLib { _rgb20_stl().expect("invalid strict type RGB20 library") }

pub fn rgb20() -> Iface {
    let types = StandardTypes::with(rgb20_stl());

    Iface {
        version: VerNo::V1,
        name: tn!("RGB20"),
        global_state: tiny_bmap! {
            fname!("spec") => GlobalIface::required(types.get("RGBContract.DivisibleAssetSpec")),
            fname!("data") => GlobalIface::required(types.get("RGBContract.ContractData")),
            fname!("created") => GlobalIface::required(types.get("RGBContract.Timestamp")),
            fname!("issuedSupply") => GlobalIface::one_or_many(types.get("RGBContract.Amount")),
            fname!("burnedSupply") => GlobalIface::none_or_many(types.get("RGBContract.Amount")),
            fname!("replacedSupply") => GlobalIface::none_or_many(types.get("RGBContract.Amount")),
        },
        assignments: tiny_bmap! {
            fname!("inflationAllowance") => AssignIface::public(OwnedIface::Amount, Req::NoneOrMore),
            fname!("updateRight") => AssignIface::public(OwnedIface::Rights, Req::Optional),
            fname!("burnEpoch") => AssignIface::public(OwnedIface::Rights, Req::Optional),
            fname!("burnRight") => AssignIface::public(OwnedIface::Rights, Req::NoneOrMore),
            fname!("assetOwner") => AssignIface::private(OwnedIface::Amount, Req::NoneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            metadata: Some(types.get("RGBContract.IssueMeta")),
            global: tiny_bmap! {
                fname!("spec") => ArgSpec::required(),
                fname!("data") => ArgSpec::required(),
                fname!("created") => ArgSpec::required(),
                fname!("issuedSupply") => ArgSpec::required(),
            },
            assignments: tiny_bmap! {
                fname!("assetOwner") => ArgSpec::many(),
                fname!("inflationAllowance") => ArgSpec::many(),
                fname!("updateRight") => ArgSpec::optional(),
                fname!("burnEpoch") => ArgSpec::optional(),
            },
            valencies: none!(),
            errors: tiny_bset! {
                SUPPLY_MISMATCH,
                INVALID_PROOF,
                INSUFFICIENT_RESERVES
            },
        },
        transitions: tiny_bmap! {
            tn!("Transfer") => TransitionIface {
                optional: false,
                metadata: None,
                globals: none!(),
                inputs: tiny_bmap! {
                    fname!("previous") => ArgSpec::from_non_empty("assetOwner"),
                },
                assignments: tiny_bmap! {
                    fname!("beneficiary") => ArgSpec::from_non_empty("assetOwner"),
                },
                valencies: none!(),
                errors: tiny_bset! {
                    NON_EQUAL_AMOUNTS
                },
                default_assignment: Some(fname!("beneficiary")),
            },
            tn!("Issue") => TransitionIface {
                optional: true,
                metadata: Some(types.get("RGBContract.IssueMeta")),
                globals: tiny_bmap! {
                    fname!("issuedSupply") => ArgSpec::required(),
                },
                inputs: tiny_bmap! {
                    fname!("used") => ArgSpec::from_non_empty("inflationAllowance"),
                },
                assignments: tiny_bmap! {
                    fname!("beneficiary") => ArgSpec::from_many("assetOwner"),
                    fname!("future") => ArgSpec::from_many("inflationAllowance"),
                },
                valencies: none!(),
                errors: tiny_bset! {
                    SUPPLY_MISMATCH,
                    INVALID_PROOF,
                    ISSUE_EXCEEDS_ALLOWANCE,
                    INSUFFICIENT_RESERVES
                },
                default_assignment: Some(fname!("beneficiary")),
            },
            tn!("OpenEpoch") => TransitionIface {
                optional: true,
                metadata: None,
                globals: none!(),
                inputs: tiny_bmap! {
                    fname!("used") => ArgSpec::from_required("burnEpoch"),
                },
                assignments: tiny_bmap! {
                    fname!("next") => ArgSpec::from_optional("burnEpoch"),
                    fname!("burnRight") => ArgSpec::required()
                },
                valencies: none!(),
                errors: none!(),
                default_assignment: Some(fname!("burnRight")),
            },
            tn!("Burn") => TransitionIface {
                optional: true,
                metadata: Some(types.get("RGBContract.BurnMeta")),
                globals: tiny_bmap! {
                    fname!("burnedSupply") => ArgSpec::required(),
                },
                inputs: tiny_bmap! {
                    fname!("used") => ArgSpec::from_required("burnRight"),
                },
                assignments: tiny_bmap! {
                    fname!("future") => ArgSpec::from_optional("burnRight"),
                },
                valencies: none!(),
                errors: tiny_bset! {
                    SUPPLY_MISMATCH,
                    INVALID_PROOF,
                    INSUFFICIENT_COVERAGE
                },
                default_assignment: None,
            },
            tn!("Replace") => TransitionIface {
                optional: true,
                metadata: Some(types.get("RGBContract.BurnMeta")),
                globals: tiny_bmap! {
                    fname!("replacedSupply") => ArgSpec::required(),
                },
                inputs: tiny_bmap! {
                    fname!("used") => ArgSpec::from_required("burnRight"),
                },
                assignments: tiny_bmap! {
                    fname!("beneficiary") => ArgSpec::from_many("assetOwner"),
                    fname!("future") => ArgSpec::from_optional("burnRight"),
                },
                valencies: none!(),
                errors: tiny_bset! {
                    NON_EQUAL_AMOUNTS,
                    SUPPLY_MISMATCH,
                    INVALID_PROOF,
                    INSUFFICIENT_COVERAGE
                },
                default_assignment: Some(fname!("beneficiary")),
            },
            tn!("Rename") => TransitionIface {
                optional: true,
                metadata: None,
                globals: tiny_bmap! {
                    fname!("new") => ArgSpec::from_required("spec"),
                },
                inputs: tiny_bmap! {
                    fname!("used") => ArgSpec::from_required("updateRight"),
                },
                assignments: tiny_bmap! {
                    fname!("future") => ArgSpec::from_optional("updateRight"),
                },
                valencies: none!(),
                errors: none!(),
                default_assignment: Some(fname!("future")),
            },
        },
        extensions: none!(),
        error_type: types.get("RGB20.Error"),
        default_operation: Some(tn!("Transfer")),
        type_system: types.type_system(),
    }
}

#[derive(Wrapper, WrapperMut, Clone, Eq, PartialEq, Debug)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
pub struct Rgb20(ContractIface);

impl From<ContractIface> for Rgb20 {
    fn from(iface: ContractIface) -> Self {
        if iface.iface.iface_id != rgb20().iface_id() {
            panic!("the provided interface is not RGB20 interface");
        }
        Self(iface)
    }
}

impl Rgb20 {
    pub fn spec(&self) -> DivisibleAssetSpec {
        let strict_val = &self
            .0
            .global("spec")
            .expect("RGB20 interface requires global state `spec`")[0];
        DivisibleAssetSpec::from_strict_val_unchecked(strict_val)
    }

    pub fn created(&self) -> Timestamp {
        let strict_val = &self
            .0
            .global("created")
            .expect("RGB20 interface requires global state `created`")[0];
        Timestamp::from_strict_val_unchecked(strict_val)
    }

    pub fn balance(&self, filter: &impl OutpointFilter) -> u64 {
        self.allocations(filter)
            .iter()
            .map(|alloc| alloc.value)
            .sum::<u64>()
    }

    pub fn allocations(&self, filter: &impl OutpointFilter) -> LargeVec<FungibleAllocation> {
        self.0
            .fungible("assetOwner", filter)
            .expect("RGB20 interface requires `assetOwner` state")
    }

    pub fn contract_data(&self) -> ContractData {
        let strict_val = &self
            .0
            .global("data")
            .expect("RGB20 interface requires global `data`")[0];
        ContractData::from_strict_val_unchecked(strict_val)
    }

    pub fn total_issued_supply(&self) -> Amount {
        self.0
            .global("issuedSupply")
            .expect("RGB20 interface requires global `issuedSupply`")
            .iter()
            .map(Amount::from_strict_val_unchecked)
            .sum()
    }

    pub fn total_burned_supply(&self) -> Amount {
        self.0
            .global("burnedSupply")
            .unwrap_or_default()
            .iter()
            .map(Amount::from_strict_val_unchecked)
            .sum()
    }

    pub fn total_replaced_supply(&self) -> Amount {
        self.0
            .global("replacedSupply")
            .unwrap_or_default()
            .iter()
            .map(Amount::from_strict_val_unchecked)
            .sum()
    }

    pub fn total_supply(&self) -> Amount { self.total_issued_supply() - self.total_burned_supply() }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::containers::BindleContent;

    const RGB20: &str = include_str!("../../tests/data/rgb20.rgba");

    #[test]
    fn lib_id() {
        let lib = rgb20_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_RGB20);
    }

    #[test]
    fn iface_creation() { rgb20(); }

    #[test]
    fn iface_bindle() {
        assert_eq!(format!("{}", rgb20().bindle()), RGB20);
    }
}
