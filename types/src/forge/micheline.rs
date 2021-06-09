pub use super::prim_type::PrimType;
use super::{Forge, Forged};

//TODO: make sure logic is correct. Sample: https://github.com/baking-bad/netezos/blob/de88439f10bbdfc2c7942c00849756c4be864a81/Netezos/Forging/Local/LocalForge.Forgers.cs#L131
fn forge_micheline_uint(mut num: u64) -> Forged {
    let mut res = vec![(num & 0x3f) as u8];

    num >>= 6;

    while num > 0 {
        match res.last_mut() {
            Some(last) => *last |= 0x80,
            None => {}
        };
        res.push((num & 0x7F) as u8);
        num >>= 7;
    }

    Forged(res)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Micheline {
    /// technically this `Int` can be boundless integer (bigint).
    /// Also it can be negative, but for now only unsigned u64 is used,
    /// so forging bigger numbers or negative ones aren't implemented.
    Int(u64),
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<Micheline>),
    Prim(MichelinePrim),
}

impl Micheline {
    pub fn str<S: AsRef<str>>(value: S) -> Self {
        Self::String(value.as_ref().to_string())
    }
}

impl Forge for Micheline {
    fn forge(&self) -> Forged {
        Forged(match self {
            Self::Int(num) => [vec![0], forge_micheline_uint(*num).take()].concat(),
            Self::Bytes(bytes) => [vec![10], bytes.forge().take()].concat(),
            Self::String(s) => [vec![1], s.forge().take()].concat(),
            Self::Array(arr) => [vec![2], arr.forge().take()].concat(),
            Self::Prim(prim) => prim.forge().take(),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum MichelineEntrypoint {
    Default,
    Root,
    Do,
    SetDelegate,
    RemoveDelegate,
    // TODO: replace with newtype (not pub) to reject strings longer than
    // 255 to avoid integer overflow during forging when casting length
    // of this string to `u8`.
    Custom(String),
}

impl Forge for MichelineEntrypoint {
    fn forge(&self) -> Forged {
        Forged(match self {
            Self::Default => vec![0],
            Self::Root => vec![1],
            Self::Do => vec![2],
            Self::SetDelegate => vec![3],
            Self::RemoveDelegate => vec![4],
            Self::Custom(custom) => {
                debug_assert!(custom.len() <= u8::MAX as usize);
                [
                    vec![255],
                    (custom.len() as u8).to_be_bytes().to_vec(),
                    vec![0, 0, 0],
                    custom.as_bytes().to_vec(),
                ].concat()
            }
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Annotation {
    Type(String),
    Field(String),
    Variable(String),
}

impl ToString for Annotation {
    fn to_string(&self) -> String {
        let (prefix, value) = match self {
            Self::Type(val) => (":", val),
            Self::Field(val) => ("%", val),
            Self::Variable(val) => ("@", val),
        };

        format!("{}{}", prefix, value)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MichelinePrim {
    pub prim_type: PrimType,
    pub args: Option<Vec<Micheline>>,
    pub annots: Option<Vec<Annotation>>,
}

impl MichelinePrim {
    pub fn new(prim_type: PrimType) -> Self {
        Self {
            prim_type,
            args: None,
            annots: None,
        }
    }

    /// Replaces current args list with `args`.
    pub fn with_args(mut self, args: Vec<Micheline>) -> Self {
        self.args = Some(args);
        self
    }

    /// Adds arg to current `args` list.
    pub fn with_arg(mut self, arg: Micheline) -> Self {
        self.args
            .get_or_insert_with(|| vec![])
            .push(arg);
        self
    }

    /// Replaces current annotations list with `annots`.
    pub fn with_annots(mut self, annots: Vec<Annotation>) -> Self {
        self.annots = Some(annots);
        self
    }

    /// Adds annotation to current annotations list.
    pub fn with_annot(mut self, annotation: Annotation) -> Self {
        self.annots
            .get_or_insert_with(|| vec![])
            .push(annotation);
        self
    }
}

impl Forge for MichelinePrim {
    fn forge(&self) -> Forged {
        let mut res = vec![];

        let args_len = self.args.as_ref().map(Vec::len).unwrap_or(0);
        let annots_len = self.annots.as_ref().map(Vec::len).unwrap_or(0);

        let tag = 9.min(args_len * 2 + 3 + annots_len);

        res.push(tag as u8);
        res.push(self.prim_type.into());

        match &self.args {
            Some(args) if args.len() > 0 => {
                if args_len < 3 {
                    // if args_len is less than 3, don't prepend the size
                    // of the args to the forged bytes.
                    res.extend(args.iter().flat_map(|x| x.forge()));
                } else {
                    // if args_len is greater or equal to 3, prepend
                    // the size of the args to the forged bytes.
                    res.extend(args.forge());
                }
            }
            _ => {}
        }

        match &self.annots {
            Some(annots) if annots.len() > 0 => {
                res.extend(
                    annots.iter()
                        .map(|annot| annot.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                        .forge()
                        .take()
                );
            }
            _ => {}
        }

        if args_len >= 3 {
            res.extend(&[0, 0, 0, 0]);
        }

        Forged(res)
    }
}

impl From<MichelinePrim> for Micheline {
    fn from(prim: MichelinePrim) -> Self {
        Micheline::Prim(prim)
    }
}
