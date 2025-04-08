#[cfg(test)]
use crate::tests::UrlFaker;
#[cfg(test)]
use fake::Rng;
#[cfg(test)]
use fake::{Dummy, Fake, faker::lorem::en::Word, faker::lorem::en::Words, faker::name::en::Name};
use schema::{XmlSchemaValidationError, validate_xml};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;
#[cfg(test)]
use url::Url;

pub mod schema;
#[cfg(test)]
pub mod tests;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
#[serde(rename = "SHOP")]
pub struct Shop {
    #[cfg_attr(test, dummy(expr = "fake::vec![ShopItem;0..1000]"))]
    #[serde(rename = "SHOPITEM")]
    pub shop_item: Vec<ShopItem>,
}

impl Shop {
    pub fn validate(&self) -> Result<(), XmlSchemaValidationError> {
        let mut buf = String::new();
        let mut s = quick_xml::se::Serializer::new(&mut buf);
        s.indent(' ', 4);
        self.serialize(s).unwrap();
        Ok(validate_xml(&buf)?)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
#[serde(rename = "SHOPITEM")]
///Guidelines to filling these out:  https://sluzby.heureka.sk/napoveda/xml-feed/
/// All text should serialize like:
/// `
/// <MANUFACTURER><![CDATA[Black & Decker]]></MANUFACTURER>
/// `
pub struct ShopItem {
    /// only [ _ - 0-9 a-z A-Z ]
    #[cfg_attr(test, dummy(faker = "fake::uuid::UUIDv7"))]
    pub item_id: String,
    /// max 200 char
    #[cfg_attr(test, dummy(faker = "Name()"))]
    pub productname: String,
    #[cfg_attr(test, dummy(faker = "Name()"))]
    #[cfg_attr(test, serde(skip_serializing_if = "Option::is_none"))]
    pub product: Option<String>,
    #[cfg_attr(test, dummy(faker = "Name()"))]
    #[cfg_attr(test, serde(skip_serializing_if = "Option::is_none"))]
    /// limits to 200 char displayed at once
    pub description: Option<String>,
    #[cfg_attr(test, serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(test, dummy(faker = "UrlFaker"))]
    /// max 300 char
    pub url: Option<url::Url>,
    #[cfg_attr(test, dummy(faker = "UrlFaker"))]
    /// max 255 char
    pub imgurl: url::Url,
    /// max 255 char
    #[cfg_attr(test, dummy(faker = "UrlFaker"))]
    pub imgurl_alternative: Vec<url::Url>,
    #[cfg_attr(test, dummy(faker = "UrlFaker"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// only youtube.com
    pub video_url: Option<url::Url>,
    /// max 2 decimal places, if in czk has to be rounded to full number for marketplace
    pub price_vat: rust_decimal::Decimal,
    #[cfg_attr(
        test,
        dummy(
            expr = "rand::random_bool(0.5).then(|| (0..100).fake::<usize>().to_string() + \"%\")"
        )
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// eg. 21%
    pub vat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// For items that aren't new, maybe there's an enum for this?
    pub item_type: Option<String>,
    pub param: Vec<Param>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// download xml from here and typecheck? https://www.heureka.sk/direct/xml-export/shops/heureka-sekce.xml
    pub categorytext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// typecheck?
    pub ean: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// typecheck?
    pub isbn: Option<String>,
    // #[dummy(faker = "fake::faker::decimal")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// max 2 decimal places, max 50.00€ if in czk has to be rounded to full number for marketplace
    pub heureka_cpc: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// either number in days or a datetime
    /**
    skladom - 0
    do 3 dní - 1-3
    do týždňa - 4-7
    do 2 týždňov - 8-14
    do mesiaca - 15-30
    viac ako mesiac - 31 a viac
    info v obchode - pokiaľ dodaciu dobu neuvádzate
    **/
    pub delivery_date: Option<DateOrDays>,
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub productno: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub delivery: Vec<Delivery>,
    //TODO: WHERE IS IT IN THE DOCS? WHY IS IT GONE? IT'S IN THE SUPPORT MAIL WHAT
    //Was probably a vec<String>?
    #[cfg_attr(test, dummy(faker = "fake::uuid::UUIDv7"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// max 36 char,  [ _ - 0-9 a-z A-Z ]
    pub itemgroup_id: Option<String>,
    /// ITEM_ID, ref to other shopItem.item_id
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accessory: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Total additional costs, with TAX/DPH
    pub dues: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ///Pair with gift_id
    pub gift: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_warranty: Option<ExtendedWarranty>,
    #[cfg_attr(test, dummy(faker = "Words(0..5)"))]
    ///Max 5 of these
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub special_service: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(untagged)]
pub enum DateOrDays {
    Date(chrono::NaiveDate),
    Days(u16),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
pub struct Param {
    /// I refuse to make this an enum, options here: https://docs.google.com/spreadsheets/d/e/2PACX-1vROYv0vyQXMg7c7Xu5fRTCr1fXlhWaGqRsCtST7-2jy0zQBDcSkvkqO1qawTywbQe8Xd2rPtFiMSjQR/pubhtml?gid=1459300428&single=true
    #[cfg_attr(test, dummy(expr = "\"farba\".into()"))]
    pub param_name: String,
    #[cfg_attr(test, dummy(faker = "Word()"))]
    pub val: String,
}

/// impl Dummy<Name> for &'static str {
///     fn dummy_with_rng<R: Rng + ?Sized>(_: &Name, rng: &mut R) -> &'static str {
///         const NAMES: &[&str] = &["John Doe", "Jane Doe"];
///         NAMES.choose(rng).unwrap()
///     }
/// }

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
pub struct Delivery {
    pub delivery_id: DeliveryCourierId,
    /// Incl. TAX/DPH
    pub delivery_price: rust_decimal::Decimal,
    /// Incl. TAX/DPH of delivery price and COD tax
    pub delivery_price_cod: rust_decimal::Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
pub enum DeliveryCourierId {
    SlovenskaPosta,
    CeskaPosta,
    CeskaPostaDoporucenaZasilka,
    CsadLogistikOstrava,
    #[serde(rename = "DPD")]
    DPD,
    #[serde(rename = "DHL")]
    DHL,
    #[serde(rename = "DSV")]
    DSV,
    #[serde(rename = "FOFR")]
    FOFR,
    ExpresKurier,
    GebruderWeiss,
    Geis,
    #[serde(rename = "GLS")]
    GLS,
    #[serde(rename = "HDS")]
    HDS,
    ExpressOne,
    #[serde(rename = "PPL")]
    PPL,
    Seegmuller,
    #[serde(rename = "TNT")]
    TNT,
    Toptrans,
    #[serde(rename = "UPS")]
    UPS,
    Fedex,
    RabenLogistics,
    ZasilkovnaNaAdresu,
    #[serde(rename = "SDS")]
    SDS,
    #[serde(rename = "SPS")]
    SPS,
    ///123KURIER
    #[serde(rename = "123KURIER")]
    JednaDvaTriKurier,
    PacketaDomov,
    PaletExpress,
    WedoHome,
    RhenusLogistics,
    Messenger,
    #[serde(rename = "SLOVENSKA_POSTA_NAPOSTU_DEPOTAPI")]
    SlovenskaPostaNapostuDEPOTAPI,
    Zasilkovna,
    #[serde(rename = "BALIKOVNA_DEPOTAPI")]
    BalikovnaDEPOTAPI,
    Packeta,
    DpdPickup,
    WedoPoint,
    Balikovo,
    CeskaPostaNapostu,
    PplParcelshop,
    GlsParcelshop,
    Depo,
    Alzapoint,
    DpdBox,
    ZBox,
    WedoBox,
    BalikovnaBox,
    BalikoBox,
    GlsParcellocker,
    Alzabox,
    Online,
    VlastnaPreprava,
    VlastniPreprava,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
pub struct ExtendedWarranty {
    ///in months, above 999 months is lifetime warranty
    pub val: u16,
    pub desc: String,
}

// fn main() {
//     let shop = Faker.fake::<Shop>();
//     let mut xml = String::new();
//     let mut s = quick_xml::se::Serializer::new(&mut xml);
//     s.indent(' ', 4);
//     shop.serialize(s).unwrap();
//     std::fs::write("a.xml", &xml).unwrap();
// }
