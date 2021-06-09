use crate::{Address, NewTransactionParameters, ManagerParameter};
use super::{Forge, Forged};
use super::micheline::{Micheline, MichelineEntrypoint, MichelinePrim, PrimType};

fn prim(prim_type: PrimType) -> MichelinePrim {
    MichelinePrim::new(prim_type)
}

impl Forge for ManagerParameter {
    fn forge(&self) -> Forged {
        let mut res = MichelineEntrypoint::Do.forge().take();

        let mut value: Vec<Micheline> = vec![
            prim(PrimType::DROP).into(),
            prim(PrimType::NIL)
                .with_arg(prim(PrimType::operation).into())
                .into(),
        ];
        value.extend(match self {
            Self::SetDelegate(addr) => {
                let delegate = addr.forge().take();
                vec![
                    prim(PrimType::PUSH).with_args(vec![
                        prim(PrimType::key_hash).into(),
                        Micheline::Bytes(delegate),
                    ]).into(),
                    prim(PrimType::SOME).into(),
                    prim(PrimType::SET_DELEGATE).into(),
                ]
            }
            Self::CancelDelegate => {
                vec![
                    prim(PrimType::NONE)
                        .with_arg(prim(PrimType::key_hash).into())
                        .into(),
                    prim(PrimType::SET_DELEGATE).into(),
                ]
            }
            Self::Transfer { to, amount } => {
                let mut values = match to {
                    Address::Implicit(dest) => vec![
                        prim(PrimType::PUSH).with_args(vec![
                            prim(PrimType::key_hash).into(),
                            Micheline::Bytes(dest.forge().take()),
                        ]).into(),
                        prim(PrimType::IMPLICIT_ACCOUNT).into(),
                    ],
                    Address::Originated(_) => vec![
                        prim(PrimType::PUSH).with_args(vec![
                            prim(PrimType::address).into(),
                            Micheline::Bytes(to.forge().take()),
                        ]).into(),
                        prim(PrimType::CONTRACT)
                            .with_arg(prim(PrimType::unit).into())
                            .into(),
                        Micheline::Array(vec![
                            prim(PrimType::IF_NONE).with_args(vec![
                                Micheline::Array(vec![
                                    Micheline::Array(vec![
                                        prim(PrimType::UNIT).into(),
                                        prim(PrimType::FAILWITH).into(),
                                    ]),
                                ]),
                                Micheline::Array(vec![]),
                            ]).into(),
                        ]),
                    ],
                };

                values.extend(vec![
                    prim(PrimType::PUSH).with_args(vec![
                        prim(PrimType::mutez).into(),
                        Micheline::Int(*amount),
                    ]).into(),
                    prim(PrimType::UNIT).into(),
                    prim(PrimType::TRANSFER_TOKENS).into(),
                ]);

                values
            }
        });
        value.push(prim(PrimType::CONS).into());

        let value_bytes = Micheline::Array(value).forge().take();
        res.extend(value_bytes.forge());

        Forged(res)
    }
}

impl Forge for NewTransactionParameters {
    fn forge(&self) -> Forged {
        match self {
            Self::Manager(manager_params) => manager_params.forge(),
            Self::Custom { entrypoint, data } => {
                let mut res = entrypoint.forge().take();

                // TODO: maybe needs to be forged as array as well?
                res.extend(data.forge().take());

                Forged(res)
            }
        }
    }
}
