use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
pub enum LocaleCode {
    Ar,
    Az,
    Bg,
    Bn,
    Ca,
    Cs,
    Da,
    De,
    El,
    En,
    Es,
    EsCO,
    Et,
    Fa,
    Fr,
    Hi,
    Hu,
    Hy,
    Id,
    Is,
    It,
    Ja,
    Ko,
    Mn,
    Nb,
    Nl,
    Pl,
    Pt,
    PtBR,
    Ro,
    Ru,
    Sk,
    Sl,
    Sq,
    Sr,
    Sv,
    Th,
    Tr,
    Uk,
    Vi,
    ZhHans,
    ZhHant,
}

impl Default for LocaleCode {
    fn default() -> Self {
        Self::En
    }
}
