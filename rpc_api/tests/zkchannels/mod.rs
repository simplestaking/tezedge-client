use crypto::hex;
use types::micheline::{Micheline, MichelinePrim, PrimType};

pub mod samples;
mod contract_code_forged;
pub use contract_code_forged::*;

pub struct InitialStorage {
    pub cid: String, // bls12_381_fr
    pub close_flag: String, // bls12_381_fr
    pub context_string: String, // string
    pub custAddr: String, // address
    pub custBal: String, // mutez
    pub custFunding: String, // mutez
    pub custPk: String, // key
    pub delayExpiry: String, // timestamp
    pub g2: String,
    pub merchAddr: String, // address
    pub merchBal: String, // mutez
    pub merchFunding: String, // mutez
    pub merchPk: String, // key
    pub merchPk0: String, // bls12_381_g2
    pub merchPk1: String, // bls12_381_g2
    pub merchPk2: String, // bls12_381_g2
    pub merchPk3: String, // bls12_381_g2
    pub merchPk4: String, // bls12_381_g2
    pub merchPk5: String, // bls12_381_g2
    pub revLock: String, // bytes
    pub selfDelay: String, // int
    pub status: String // nat
}

impl From<InitialStorage> for Micheline {
    fn from(data: InitialStorage) -> Self {
        MichelinePrim::new(PrimType::Pair).with_args(vec![
            MichelinePrim::new(PrimType::Pair).with_args(vec![
                MichelinePrim::new(PrimType::Pair).with_args(vec![
                    MichelinePrim::new(PrimType::Pair).with_args(vec![
                        Micheline::Bytes(hex::decode(data.cid).unwrap()),
                        Micheline::Bytes(hex::decode(data.close_flag).unwrap()),
                    ]).into(),
                    Micheline::String(data.context_string),
                    Micheline::String(data.custAddr),
                    Micheline::Int(data.custBal.parse().unwrap()),
                ]).into(),
                MichelinePrim::new(PrimType::Pair).with_args(vec![
                    Micheline::Int(data.custFunding.parse().unwrap()),
                    Micheline::String(data.custPk),
                    Micheline::String(data.delayExpiry),
                ]).into(),
                Micheline::Bytes(hex::decode(data.g2).unwrap()),
                Micheline::String(data.merchAddr),
                Micheline::Int(data.merchBal.parse().unwrap()),
            ]).into(),
            MichelinePrim::new(PrimType::Pair).with_args(vec![
                MichelinePrim::new(PrimType::Pair).with_args(vec![
                    Micheline::Int(data.merchFunding.parse().unwrap()),
                    Micheline::String(data.merchPk),
                ]).into(),
                Micheline::Bytes(hex::decode(data.merchPk0).unwrap()),
                Micheline::Bytes(hex::decode(data.merchPk1).unwrap()),
                Micheline::Bytes(hex::decode(data.merchPk2).unwrap()),
            ]).into(),
            MichelinePrim::new(PrimType::Pair).with_args(vec![
                Micheline::Bytes(hex::decode(data.merchPk3).unwrap()),
                Micheline::Bytes(hex::decode(data.merchPk4).unwrap()),
                Micheline::Bytes(hex::decode(data.merchPk5).unwrap()),
            ]).into(),
            Micheline::Bytes(hex::decode(data.revLock).unwrap()),
            Micheline::Int(data.selfDelay.parse().unwrap()),
            Micheline::Int(data.status.parse().unwrap()),
        ]).into()
    }
}
