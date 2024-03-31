// RGB standard library for working with smart contracts on Bitcoin & Lightning
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2024 LNP/BP Standards Association. All rights reserved.
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

use std::collections::HashMap;
use std::str::FromStr;

use bp::dbc::Method;
use chrono::Utc;
use invoice::{Amount, Precision};
use rgb::{AltLayer1, AssetTag, BlindingFactor, GenesisSeal, Occurrences, Types, XWitnessId};
use strict_encoding::InvalidIdent;
use strict_types::TypeLib;

use super::{
    AssignIface, BuilderError, ContractBuilder, GenesisIface, GlobalIface, Iface, IfaceClass,
    IfaceOp, IssuerClass, Modifier, OwnedIface, Req, RightsAllocation, SchemaIssuer, StateChange,
    TransitionIface, VerNo, WitnessFilter,
};
use crate::containers::Contract;
use crate::interface::builder::TxOutpoint;
use crate::interface::{ContractIface, FungibleAllocation, IfaceId, IfaceWrapper, OutpointFilter};
use crate::persistence::PersistedState;
use crate::stl::{
    rgb_contract_stl, AssetSpec, AssetTerms, Attachment, RicardianContract, StandardTypes,
};
use crate::LIB_NAME_RGB_STD;

pub fn named_asset() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        name: tn!("NamedAsset"),
        inherits: none!(),
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        global_state: tiny_bmap! {
            fname!("spec") => GlobalIface::required(types.get("RGBContract.AssetSpec")),
            fname!("terms") => GlobalIface::required(types.get("RGBContract.AssetTerms")),
        },
        assignments: none!(),
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Abstract,
            metadata: None,
            globals: tiny_bmap! {
                fname!("spec") => Occurrences::Once,
                fname!("terms") => Occurrences::Once,
            },
            assignments: none!(),
            valencies: none!(),
            errors: none!(),
        },
        transitions: none!(),
        extensions: none!(),
        errors: none!(),
        default_operation: None,
        types: Types::Strict(types.type_system()),
    }
}

pub fn renameable() -> Iface {
    Iface {
        version: VerNo::V1,
        inherits: tiny_bset![named_asset().iface_id()],
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        name: tn!("RenameableAsset"),
        global_state: none!(),
        assignments: tiny_bmap! {
            fname!("updateRight") => AssignIface::public(OwnedIface::Rights, Req::Required),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: none!(),
            assignments: tiny_bmap! {
                fname!("updateRight") => Occurrences::Once,
            },
            valencies: none!(),
            errors: none!(),
        },
        transitions: tiny_bmap! {
            fname!("rename") => TransitionIface {
                modifier: Modifier::Final,
                optional: false,
                metadata: None,
                globals: tiny_bmap! {
                    fname!("spec") => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    fname!("updateRight") => Occurrences::Once,
                },
                assignments: tiny_bmap! {
                    fname!("updateRight") => Occurrences::NoneOrOnce,
                },
                valencies: none!(),
                errors: none!(),
                default_assignment: Some(fname!("updateRight")),
            },
        },
        extensions: none!(),
        default_operation: None,
        errors: none!(),
        types: StandardTypes::new().type_system().into(),
    }
}

pub fn fungible() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        name: tn!("FungibleAsset"),
        inherits: none!(),
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        global_state: tiny_bmap! {
            fname!("issuedSupply") => GlobalIface::required(types.get("RGBContract.Amount")),
        },
        assignments: tiny_bmap! {
            fname!("assetOwner") => AssignIface::private(OwnedIface::Amount, Req::NoneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: tiny_bmap! {
                fname!("issuedSupply") => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                fname!("assetOwner") => Occurrences::NoneOrMore,
            },
            valencies: none!(),
            errors: tiny_bset! {
                vname!("supplyMismatch"),
                vname!("invalidProof"),
                vname!("insufficientReserves")
            },
        },
        transitions: tiny_bmap! {
            fname!("transfer") => TransitionIface {
                modifier: Modifier::Abstract,
                optional: false,
                metadata: None,
                globals: none!(),
                inputs: tiny_bmap! {
                    fname!("assetOwner") => Occurrences::OnceOrMore,
                },
                assignments: tiny_bmap! {
                    fname!("assetOwner") => Occurrences::OnceOrMore,
                },
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("nonEqualAmounts")
                },
                default_assignment: Some(fname!("assetOwner")),
            },
        },
        extensions: none!(),
        errors: tiny_bmap! {
            vname!("supplyMismatch")
                => tiny_s!("supply specified as a global parameter doesn't match the issued supply allocated to the asset owners"),

            vname!("nonEqualAmounts")
                => tiny_s!("the sum of spent assets doesn't equal to the sum of assets in outputs"),
        },
        default_operation: Some(fname!("transfer")),
        types: Types::Strict(types.type_system()),
    }
}

pub fn reservable() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        name: tn!("ReservableAsset"),
        inherits: none!(),
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        global_state: none!(),
        assignments: none!(),
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: Some(types.get("RGBContract.IssueMeta")),
            globals: none!(),
            assignments: none!(),
            valencies: none!(),
            errors: tiny_bset! {
                vname!("invalidProof"),
                vname!("insufficientReserves")
            },
        },
        transitions: tiny_bmap! {
            fname!("issue") => TransitionIface {
                modifier: Modifier::Override,
                optional: true,
                metadata: Some(types.get("RGBContract.IssueMeta")),
                globals: none!(),
                inputs: none!(),
                assignments: none!(),
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("invalidProof"),
                    vname!("insufficientReserves")
                },
                default_assignment: Some(fname!("assetOwner")),
            },
        },
        extensions: none!(),
        errors: tiny_bmap! {
            vname!("invalidProof")
                => tiny_s!("the provided proof is invalid"),

            vname!("insufficientReserves")
                => tiny_s!("reserve is insufficient to cover the issued assets"),
        },
        default_operation: None,
        types: Types::Strict(types.type_system()),
    }
}

pub fn fixed() -> Iface {
    Iface {
        version: VerNo::V1,
        name: tn!("FixedAsset"),
        inherits: tiny_bset![fungible().iface_id()],
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        global_state: none!(),
        assignments: tiny_bmap! {
            fname!("assetOwner") => AssignIface::private(OwnedIface::Amount, Req::OneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: none!(),
            assignments: tiny_bmap! {
                fname!("assetOwner") => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            errors: tiny_bset! {
                vname!("supplyMismatch"),
                vname!("invalidProof"),
                vname!("insufficientReserves")
            },
        },
        transitions: none!(),
        extensions: none!(),
        errors: none!(),
        default_operation: None,
        types: StandardTypes::new().type_system().into(),
    }
}

pub fn inflatable() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        inherits: tiny_bset![fungible().iface_id()],
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        name: tn!("InflatableAsset"),
        global_state: tiny_bmap! {
            fname!("issuedSupply") => GlobalIface::one_or_many(types.get("RGBContract.Amount")),
        },
        assignments: tiny_bmap! {
            fname!("inflationAllowance") => AssignIface::public(OwnedIface::Amount, Req::NoneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: none!(),
            assignments: tiny_bmap! {
                fname!("inflationAllowance") => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            errors: none!(),
        },
        transitions: tiny_bmap! {
            fname!("issue") => TransitionIface {
                modifier: Modifier::Abstract,
                optional: false,
                metadata: None,
                globals: tiny_bmap! {
                    fname!("issuedSupply") => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    fname!("inflationAllowance") => Occurrences::OnceOrMore,
                },
                assignments: tiny_bmap! {
                    fname!("assetOwner") => Occurrences::NoneOrMore,
                    fname!("inflationAllowance") => Occurrences::NoneOrMore,
                },
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("supplyMismatch"),
                    vname!("issueExceedsAllowance"),
                },
                default_assignment: Some(fname!("assetOwner")),
            },
        },
        extensions: none!(),
        default_operation: None,
        errors: tiny_bmap! {
            vname!("issueExceedsAllowance")
                => tiny_s!("you try to issue more assets than allowed by the contract terms"),
        },
        types: Types::Strict(types.type_system()),
    }
}

pub fn burnable() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        inherits: tiny_bset![fungible().iface_id()],
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        name: tn!("BurnableAsset"),
        global_state: tiny_bmap! {
            fname!("burnedSupply") => GlobalIface::none_or_many(types.get("RGBContract.Amount")),
        },
        assignments: tiny_bmap! {
            fname!("burnRight") => AssignIface::public(OwnedIface::Rights, Req::OneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: none!(),
            assignments: tiny_bmap! {
                fname!("burnRight") => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            errors: none!(),
        },
        transitions: tiny_bmap! {
            fname!("burn") => TransitionIface {
                modifier: Modifier::Final,
                optional: false,
                metadata: Some(types.get("RGBContract.BurnMeta")),
                globals: tiny_bmap! {
                    fname!("burnedSupply") => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    fname!("burnRight") => Occurrences::Once,
                },
                assignments: tiny_bmap! {
                    fname!("burnRight") => Occurrences::NoneOrMore,
                },
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("supplyMismatch"),
                    vname!("invalidProof"),
                    vname!("insufficientCoverage")
                },
                default_assignment: None,
            },
        },
        extensions: none!(),
        default_operation: None,
        errors: tiny_bmap! {
            vname!("insufficientCoverage")
                => tiny_s!("the claimed amount of burned assets is not covered by the assets in the operation inputs"),
        },
        types: Types::Strict(types.type_system()),
    }
}

pub fn replaceable() -> Iface {
    let types = StandardTypes::new();
    Iface {
        version: VerNo::V1,
        inherits: tiny_bset![inflatable().iface_id()],
        developer: none!(), // TODO: Add LNP/BP Standards Association
        timestamp: 1711405444,
        name: tn!("ReplaceableAsset"),
        global_state: tiny_bmap! {
            fname!("burnedSupply") => GlobalIface::none_or_many(types.get("RGBContract.Amount")),
            fname!("replacedSupply") => GlobalIface::none_or_many(types.get("RGBContract.Amount")),
        },
        assignments: tiny_bmap! {
            fname!("burnEpoch") => AssignIface::public(OwnedIface::Rights, Req::OneOrMore),
            fname!("burnRight") => AssignIface::public(OwnedIface::Rights, Req::NoneOrMore),
        },
        valencies: none!(),
        genesis: GenesisIface {
            modifier: Modifier::Override,
            metadata: None,
            globals: none!(),
            assignments: tiny_bmap! {
                fname!("burnEpoch") => Occurrences::Once,
            },
            valencies: none!(),
            errors: none!(),
        },
        transitions: tiny_bmap! {
            fname!("openEpoch") => TransitionIface {
                modifier: Modifier::Final,
                optional: false,
                metadata: None,
                globals: none!(),
                inputs: tiny_bmap! {
                    fname!("burnEpoch") => Occurrences::Once,
                },
                assignments: tiny_bmap! {
                    fname!("burnEpoch") => Occurrences::NoneOrOnce,
                    fname!("burnRight") => Occurrences::Once,
                },
                valencies: none!(),
                errors: none!(),
                default_assignment: Some(fname!("burnRight")),
            },
            fname!("burn") => TransitionIface {
                modifier: Modifier::Final,
                optional: false,
                metadata: Some(types.get("RGBContract.BurnMeta")),
                globals: tiny_bmap! {
                    fname!("burnedSupply") => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    fname!("burnRight") => Occurrences::Once,
                },
                assignments: tiny_bmap! {
                    fname!("burnRight") => Occurrences::NoneOrOnce,
                },
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("supplyMismatch"),
                    vname!("invalidProof"),
                    vname!("insufficientCoverage")
                },
                default_assignment: None,
            },
            fname!("replace") => TransitionIface {
                modifier: Modifier::Final,
                optional: false,
                metadata: Some(types.get("RGBContract.BurnMeta")),
                globals: tiny_bmap! {
                    fname!("replacedSupply") => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    fname!("burnRight") => Occurrences::Once,
                },
                assignments: tiny_bmap! {
                    fname!("assetOwner") => Occurrences::NoneOrMore,
                    fname!("burnRight") => Occurrences::NoneOrOnce,
                },
                valencies: none!(),
                errors: tiny_bset! {
                    vname!("nonEqualAmounts"),
                    vname!("supplyMismatch"),
                    vname!("invalidProof"),
                    vname!("insufficientCoverage")
                },
                default_assignment: Some(fname!("assetOwner")),
            },
        },
        extensions: none!(),
        default_operation: None,
        errors: tiny_bmap! {
            vname!("insufficientCoverage")
                => tiny_s!("the claimed amount of burned assets is not covered by the assets in the operation inputs"),
        },
        types: Types::Strict(types.type_system()),
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub enum Inflation {
    #[default]
    Fixed,
    Burnable,
    Inflatible,
    InflatibleBurnable,
    Replaceable,
}

impl Inflation {
    pub fn is_fixed(self) -> bool { self == Self::Fixed }
    pub fn is_inflatible(self) -> bool {
        self == Self::Inflatible || self == Self::InflatibleBurnable || self == Self::Replaceable
    }
    pub fn is_replacable(self) -> bool { self == Self::Replaceable }
    pub fn is_burnable(self) -> bool { self == Self::Burnable || self == Self::Replaceable }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Features {
    pub renaming: bool,
    pub reserves: bool,
    pub inflation: Inflation,
}

impl Features {
    pub fn none() -> Self {
        Features {
            renaming: false,
            reserves: false,
            inflation: Inflation::Fixed,
        }
    }
    pub fn all() -> Self {
        Features {
            renaming: true,
            reserves: true,
            inflation: Inflation::Replaceable,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD, tags = custom)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum AmountChange {
    #[display("-{0}")]
    #[strict_type(tag = 0xFF)]
    Dec(Amount),

    #[display("0")]
    #[strict_type(tag = 0, dumb)]
    Zero,

    #[display("+{0}")]
    #[strict_type(tag = 0x01)]
    Inc(Amount),
}

impl StateChange for AmountChange {
    type State = Amount;

    fn from_spent(state: Self::State) -> Self { AmountChange::Dec(state) }

    fn from_received(state: Self::State) -> Self { AmountChange::Inc(state) }

    fn merge_spent(&mut self, sub: Self::State) {
        *self = match self {
            AmountChange::Dec(neg) => AmountChange::Dec(*neg + sub),
            AmountChange::Zero => AmountChange::Dec(sub),
            AmountChange::Inc(pos) if *pos > sub => AmountChange::Inc(*pos - sub),
            AmountChange::Inc(pos) if *pos == sub => AmountChange::Zero,
            AmountChange::Inc(pos) if *pos < sub => AmountChange::Dec(sub - *pos),
            AmountChange::Inc(_) => unreachable!(),
        };
    }

    fn merge_received(&mut self, add: Self::State) {
        *self = match self {
            AmountChange::Inc(pos) => AmountChange::Inc(*pos + add),
            AmountChange::Zero => AmountChange::Inc(add),
            AmountChange::Dec(neg) if *neg > add => AmountChange::Dec(*neg - add),
            AmountChange::Dec(neg) if *neg == add => AmountChange::Zero,
            AmountChange::Dec(neg) if *neg < add => AmountChange::Inc(add - *neg),
            AmountChange::Dec(_) => unreachable!(),
        };
    }
}

#[derive(Wrapper, WrapperMut, Clone, Eq, PartialEq, Debug)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
pub struct Rgb20(ContractIface);

impl From<ContractIface> for Rgb20 {
    fn from(iface: ContractIface) -> Self {
        if iface.iface.iface_id != Rgb20::IFACE_ID {
            panic!("the provided interface is not RGB20 interface");
        }
        Self(iface)
    }
}

impl IfaceWrapper for Rgb20 {
    const IFACE_NAME: &'static str = "RGB20";
    const IFACE_ID: IfaceId = IfaceId::from_array([
        0xd3, 0xa5, 0x1e, 0x8c, 0x19, 0xad, 0x05, 0x4f, 0xc8, 0x95, 0x0d, 0x13, 0x0f, 0xc9, 0x54,
        0xbd, 0xba, 0x2b, 0x27, 0xe9, 0x08, 0x6d, 0xf3, 0xcd, 0x7e, 0x34, 0x18, 0x60, 0xf9, 0x4f,
        0x73, 0x8e,
    ]);
}

impl IfaceClass for Rgb20 {
    type Features = Features;
    fn iface(features: Features) -> Iface {
        let mut iface = named_asset().expect_extended(fungible());
        if features.renaming {
            iface = iface.expect_extended(renameable());
        }
        if features.inflation.is_fixed() {
            iface = iface.expect_extended(fixed());
        }
        if features.inflation.is_inflatible() {
            iface = iface.expect_extended(inflatable());
        }
        if features.inflation.is_replacable() {
            iface = iface.expect_extended(replaceable());
        } else if features.inflation.is_burnable() {
            iface = iface.expect_extended(burnable());
        }
        if features.reserves {
            iface = iface.expect_extended(reservable());
        }
        iface.name = Self::IFACE_NAME.into();
        iface
    }
    fn stl() -> TypeLib { rgb_contract_stl() }
}

impl Rgb20 {
    pub fn testnet<C: IssuerClass<IssuingIface = Self>>(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        features: Features,
    ) -> Result<PrimaryIssue, InvalidIdent> {
        PrimaryIssue::testnet::<C>(ticker, name, details, precision, features)
    }

    pub fn testnet_det<C: IssuerClass<IssuingIface = Self>>(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        features: Features,
        asset_tag: AssetTag,
    ) -> Result<PrimaryIssue, InvalidIdent> {
        PrimaryIssue::testnet_det::<C>(ticker, name, details, precision, features, asset_tag)
    }

    pub fn spec(&self) -> AssetSpec {
        let strict_val = &self
            .0
            .global("spec")
            .expect("RGB20 interface requires global state `spec`")[0];
        AssetSpec::from_strict_val_unchecked(strict_val)
    }

    pub fn balance(&self, filter: impl OutpointFilter) -> Amount {
        self.allocations(filter)
            .map(|alloc| alloc.state)
            .sum::<Amount>()
    }

    pub fn allocations<'c>(
        &'c self,
        filter: impl OutpointFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0
            .fungible("assetOwner", filter)
            .expect("RGB20 interface requires `assetOwner` state")
    }

    pub fn inflation_allowance_allocations<'c>(
        &'c self,
        filter: impl OutpointFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0
            .fungible("inflationAllowance", filter)
            .expect("RGB20 interface requires `inflationAllowance` state")
    }

    pub fn update_right<'c>(
        &'c self,
        filter: impl OutpointFilter + 'c,
    ) -> impl Iterator<Item = RightsAllocation> + 'c {
        self.0
            .rights("updateRight", filter)
            .expect("RGB20 interface requires `updateRight` state")
    }

    pub fn burn_epoch<'c>(
        &'c self,
        filter: impl OutpointFilter + 'c,
    ) -> impl Iterator<Item = RightsAllocation> + 'c {
        self.0
            .rights("burnEpoch", filter)
            .expect("RGB20 interface requires `burnEpoch` state")
    }

    pub fn burn_right<'c>(
        &'c self,
        filter: impl OutpointFilter + 'c,
    ) -> impl Iterator<Item = RightsAllocation> + 'c {
        self.0
            .rights("burnRight", filter)
            .expect("RGB20 interface requires `updateRight` state")
    }

    pub fn contract_terms(&self) -> AssetTerms {
        let strict_val = &self
            .0
            .global("terms")
            .expect("RGB20 interface requires global `terms`")[0];
        AssetTerms::from_strict_val_unchecked(strict_val)
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

    pub fn transfer_history(
        &self,
        witness_filter: impl WitnessFilter + Copy,
        outpoint_filter: impl OutpointFilter + Copy,
    ) -> HashMap<XWitnessId, IfaceOp<AmountChange>> {
        self.0
            .fungible_ops("assetOwner", witness_filter, outpoint_filter)
            .expect("state name is not correct")
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(doc_comments)]
pub enum AllocationError {
    /// contract genesis doesn't support allocating to liquid seals; request
    /// liquid support first.
    NoLiquidSupport,
    /// overflow in the amount of the issued assets: the total amount must not
    /// exceed 2^64.
    AmountOverflow,
}

impl From<BuilderError> for AllocationError {
    fn from(err: BuilderError) -> Self {
        match err {
            BuilderError::InvalidLayer1(_) => AllocationError::NoLiquidSupport,
            _ => panic!("invalid RGB20 schema (assetOwner mismatch)"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrimaryIssue {
    builder: ContractBuilder,
    issued: Amount,
    terms: AssetTerms,
    deterministic: bool,
}

impl PrimaryIssue {
    fn testnet_int(
        issuer: SchemaIssuer<Rgb20>,
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
    ) -> Result<Self, InvalidIdent> {
        let spec = AssetSpec::with(ticker, name, precision, details)?;
        let terms = AssetTerms {
            text: RicardianContract::default(),
            media: None,
        };

        let (schema, main_iface_impl, features) = issuer.into_split();
        let builder = ContractBuilder::testnet(Rgb20::iface(features), schema, main_iface_impl)
            .expect("schema interface mismatch")
            .add_global_state("spec", spec)
            .expect("invalid RGB20 schema (token specification mismatch)");

        Ok(Self {
            builder,
            terms,
            issued: Amount::ZERO,
            deterministic: false,
        })
    }

    pub fn testnet<C: IssuerClass<IssuingIface = Rgb20>>(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        features: Features,
    ) -> Result<Self, InvalidIdent> {
        Self::testnet_int(C::issuer(features), ticker, name, details, precision)
    }

    pub fn testnet_with(
        issuer: SchemaIssuer<Rgb20>,
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
    ) -> Result<Self, InvalidIdent> {
        Self::testnet_int(issuer, ticker, name, details, precision)
    }

    pub fn testnet_det<C: IssuerClass<IssuingIface = Rgb20>>(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        features: Features,
        asset_tag: AssetTag,
    ) -> Result<Self, InvalidIdent> {
        let mut me = Self::testnet_int(C::issuer(features), ticker, name, details, precision)?;
        me.builder = me
            .builder
            .add_asset_tag("assetOwner", asset_tag)
            .expect("invalid RGB20 schema (assetOwner mismatch)");
        me.deterministic = true;
        Ok(me)
    }

    pub fn support_liquid(mut self) -> Self {
        self.builder = self
            .builder
            .add_layer1(AltLayer1::Liquid)
            .expect("only one layer1 can be added");
        self
    }

    pub fn add_terms(
        mut self,
        contract: &str,
        media: Option<Attachment>,
    ) -> Result<Self, InvalidIdent> {
        let terms = RicardianContract::from_str(contract)?;
        self.terms = AssetTerms { text: terms, media };
        Ok(self)
    }

    pub fn allocate<O: TxOutpoint>(
        mut self,
        method: Method,
        beneficiary: O,
        amount: Amount,
    ) -> Result<Self, AllocationError> {
        debug_assert!(
            !self.deterministic,
            "for creating deterministic contracts please use allocate_det method"
        );

        let beneficiary = beneficiary.map_to_xchain(|outpoint| {
            GenesisSeal::new_random(method, outpoint.txid, outpoint.vout)
        });
        self.issued
            .checked_add_assign(amount)
            .ok_or(AllocationError::AmountOverflow)?;
        self.builder =
            self.builder
                .add_fungible_state("assetOwner", beneficiary, amount.value())?;
        Ok(self)
    }

    pub fn allocate_all<O: TxOutpoint>(
        mut self,
        method: Method,
        allocations: impl IntoIterator<Item = (O, Amount)>,
    ) -> Result<Self, AllocationError> {
        for (beneficiary, amount) in allocations {
            self = self.allocate(method, beneficiary, amount)?;
        }
        Ok(self)
    }

    /// Add asset allocation in a deterministic way.
    pub fn allocate_det<O: TxOutpoint>(
        mut self,
        method: Method,
        beneficiary: O,
        seal_blinding: u64,
        amount: Amount,
        amount_blinding: BlindingFactor,
    ) -> Result<Self, AllocationError> {
        debug_assert!(
            self.deterministic,
            "to add asset allocation in deterministic way the contract builder has to be created \
             using `*_det` constructor"
        );

        let tag = self
            .builder
            .asset_tag("assetOwner")
            .expect("internal library error: asset tag is unassigned");
        let beneficiary = beneficiary.map_to_xchain(|outpoint| {
            GenesisSeal::with_blinding(method, outpoint.txid, outpoint.vout, seal_blinding)
        });
        self.issued
            .checked_add_assign(amount)
            .ok_or(AllocationError::AmountOverflow)?;
        self.builder = self.builder.add_owned_state_det(
            "assetOwner",
            beneficiary,
            PersistedState::Amount(amount, amount_blinding, tag),
        )?;
        Ok(self)
    }

    // TODO: implement when bulletproofs are supported
    /*
    pub fn conceal_allocations(mut self) -> Self {

    }
     */

    #[allow(clippy::result_large_err)]
    pub fn issue_contract(self) -> Result<Contract, BuilderError> {
        debug_assert!(
            !self.deterministic,
            "to add asset allocation in deterministic way you must use issue_contract_det method"
        );
        self.issue_contract_int(Utc::now().timestamp())
    }

    #[allow(clippy::result_large_err)]
    pub fn issue_contract_det(self, timestamp: i64) -> Result<Contract, BuilderError> {
        debug_assert!(
            self.deterministic,
            "to add asset allocation in deterministic way the contract builder has to be created \
             using `*_det` constructor"
        );
        self.issue_contract_int(timestamp)
    }

    #[allow(clippy::result_large_err)]
    fn issue_contract_int(self, timestamp: i64) -> Result<Contract, BuilderError> {
        self.builder
            .add_global_state("issuedSupply", self.issued)
            .expect("invalid RGB20 schema (issued supply mismatch)")
            .add_global_state("terms", self.terms)
            .expect("invalid RGB20 schema (contract terms mismatch)")
            .issue_contract_det(timestamp)
    }

    // TODO: Add secondary issuance and other methods
}

#[cfg(test)]
mod test {
    use armor::AsciiArmor;

    use super::*;

    const RGB20: &str = include_str!("../../tests/data/rgb20.rgba");

    #[test]
    fn iface_id_all() {
        let iface_id = Rgb20::iface(Features::all()).iface_id();
        eprintln!("{:#04x?}", iface_id.to_byte_array());
        assert_eq!(Rgb20::IFACE_ID, iface_id);
    }

    #[test]
    fn iface_bindle() {
        assert_eq!(format!("{}", Rgb20::iface(Features::all()).to_ascii_armored_string()), RGB20);
    }

    #[test]
    fn iface_check() {
        // TODO: test other features
        if let Err(err) = Rgb20::iface(Features::all()).check() {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid RGB20 interface definition");
        }
    }
}
