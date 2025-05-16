use super::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub struct UrlFaker;

impl Dummy<UrlFaker> for url::Url {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &UrlFaker, rng: &mut R) -> Self {
        let domain: Vec<String> = Words(4..20).fake_with_rng(rng);
        let mut url = "https://".into();
        let path_length: usize = rand::random_range(1..15);
        let subdomain_length: usize = rand::random_range(2..4);
        domain.into_iter().enumerate().for_each(|(i, w)| match i {
            0 => url = format!("{url}{w}"),
            i if (0..subdomain_length).contains(&i) => url = format!("{url}.{w}"),
            i if (subdomain_length..path_length).contains(&i) => url = format!("{url}/{w}"),
            i if (path_length == i) => url = format!("{url}?{i}={w}"),
            _ => url = format!("{url}&{i}={w}"),
        });
        Url::from_str(&url).unwrap()
    }

    fn dummy(_config: &UrlFaker) -> Self {
        let domain: Vec<String> = Words(4..20).fake();
        let mut url = "https://".into();
        let path_length: usize = rand::random_range(1..15);
        let subdomain_length: usize = rand::random_range(2..4);
        domain.into_iter().enumerate().for_each(|(i, w)| match i {
            0 => url = format!("{url}{w}"),
            i if (0..subdomain_length).contains(&i) => url = format!("{url}.{w}"),
            i if (subdomain_length..path_length).contains(&i) => url = format!("{url}/{w}"),
            i if (path_length == i) => url = format!("{url}?{i}={w}"),
            _ => url = format!("{url}&{i}={w}"),
        });
        Url::from_str(&url).unwrap()
    }
}

impl Dummy<UrlFaker> for Vec<url::Url> {
    fn dummy(config: &UrlFaker) -> Self {
        vec![config.fake(); rand::random_range(1..20)]
    }
    fn dummy_with_rng<R: Rng + ?Sized>(config: &UrlFaker, rng: &mut R) -> Self {
        vec![config.fake_with_rng(rng); rand::random_range(1..5)]
    }
}

#[test]
fn xsd_schema_is_ok_and_heureka_example_passes() {
    const HEUREKA_XML: &str = r###"<?xml version="1.0" encoding="utf-8"?>
<SHOP>
  <SHOPITEM>
    <ITEM_ID>AB123</ITEM_ID>
    <PRODUCTNAME>Nokia 5800 XpressMusic</PRODUCTNAME>
    <PRODUCT>Nokia 5800 XpressMusic</PRODUCT>
    <DESCRIPTION>Klasický s plným dotykovým uživatelským rozhraním</DESCRIPTION>
    <URL>http://obchod.cz/mobily/nokia-5800-xpressmusic</URL>
    <IMGURL>http://obchod.cz/mobily/nokia-5800-xpressmusic/obrazek.jpg</IMGURL>
    <IMGURL_ALTERNATIVE>http://obchod.cz/mobily/nokia-5800-xpressmusic/obrazek2.jpg</IMGURL_ALTERNATIVE>
    <PRICE_VAT>6000</PRICE_VAT>
    <HEUREKA_CPC>5.8</HEUREKA_CPC>
    <MANUFACTURER>NOKIA</MANUFACTURER>
    <CATEGORYTEXT>Elektronika | Mobilní telefony</CATEGORYTEXT>
    <EAN>6417182041488</EAN>
    <PRODUCTNO>RM-559394</PRODUCTNO>
    <PARAM>
      <PARAM_NAME>Barva</PARAM_NAME>
      <VAL>černá</VAL>
    </PARAM>
    <DELIVERY_DATE>2</DELIVERY_DATE>
    <DELIVERY>
      <DELIVERY_ID>CESKA_POSTA</DELIVERY_ID>
      <DELIVERY_PRICE>120</DELIVERY_PRICE>
      <DELIVERY_PRICE_COD>120</DELIVERY_PRICE_COD>
    </DELIVERY>
    <DELIVERY>
      <DELIVERY_ID>PPL</DELIVERY_ID>
      <DELIVERY_PRICE>90</DELIVERY_PRICE>
      <DELIVERY_PRICE_COD>120</DELIVERY_PRICE_COD>
    </DELIVERY>
    <ACCESSORY>CD456</ACCESSORY>
    <GIFT>Pouzdro zdarma</GIFT>
    <EXTENDED_WARRANTY>
        <VAL>36</VAL>
        <DESC>Záruka na 36 měsíců</DESC>
    </EXTENDED_WARRANTY>
    <SPECIAL_SERVICE>Aplikace ochranné fólie</SPECIAL_SERVICE>
    <SALES_VOUCHER>
        <CODE>SLEVA20</CODE>
        <DESC>Sleva 20% po zadání kódu do 31.12.2021!</DESC>
    </SALES_VOUCHER>
  </SHOPITEM>
</SHOP>"###;
    validate_xml(HEUREKA_XML).unwrap();
}

#[test]
fn heureka_example_serializes_and_validates() {
    use super::*;
    let stuff = Shop {
        shop_item: vec![ShopItem {
            item_id: "325236407".into(),
            productname: "Adidas Superstar 2 W EUR 36".into(),
            product: Some("Adidas Superstar 2 W EUR 36".into(),),
            description: Some(
                "V rámci kolekcie Originals uvádza adidas športovú obuv The Superstar, ktorá je už od svojho vzniku jedničkou medzi obuvou. Jej poznávacím znamením je mimo iných detailov designové zakončenie špičky. Vďaka kvalitnému materiálu a trendy vzhľadu, podčiarknutého logami Adidas vo vnútri topánky aj na nej, bude hviezdou vášho botníku.".into(),
            ),
            url: Some(
                Url::parse("http://www.obchod-s-obuvou.sk/topanky/adidas-superstar-2-w7/eur-36/")
                    .unwrap(),
            ),
            imgurl: Url::parse("http://www.obchod-s-obuvou.sk/pictures/403078.jpg").unwrap(),
            imgurl_alternative: vec![
                Url::parse("http://www.obchod-s-obuvi.sk/pictures/403080.jpg").unwrap(),
            ],
            price_vat: Decimal::from_u8(240).unwrap(),
            heureka_cpc: Some(Decimal::from_f32(0.24)).unwrap(),
            manufacturer: Some("Adidas".into()),
            categorytext: "Obuv | Dámska obuv".into(),
            ean: Some("5051571703857".into()),
            productno: Some("G43755".to_owned()),
            param: vec![Param {
                param_name: "Farba".into(),
                val: "čierna".into(),
            }],
            delivery_date: Some(DateOrDays::Days(0)),
            delivery: vec![
                Delivery {
                    delivery_id: DeliveryCourierId::SlovenskaPosta,
                    delivery_price_cod: Some(Decimal::from_u8(5).unwrap()),
                    delivery_price: Decimal::from_u8(3).unwrap(),
                },
                Delivery {
                    delivery_id: DeliveryCourierId::PPL,
                    delivery_price_cod: Some(Decimal::from_u8(5).unwrap()),
                    delivery_price: Decimal::from_u8(3).unwrap(),
                },
            ],
            itemgroup_id: Some("EF789".to_owned()),
            accessory: vec!["CD456".into()],
            gift: Some("Púzdro zadarmo".into()),
            extended_warranty: Some(ExtendedWarranty {
                val: 36,
                desc: "Záruka na 36 mesiacov".into(),
            }),
            special_service: vec!["Aplikácia ochrannej fólie".into()],
            vat: None,
            isbn: None,
            dues: None,
            gift_id: None,
            video_url: None,
            item_type: None,
        }],
    };
    let mut xml = String::new();
    let mut s = quick_xml::se::Serializer::new(&mut xml);
    s.indent(' ', 4);
    stuff.serialize(s).unwrap();

    // std::fs::write("output.xml", &xml).unwrap();
    validate_xml(&xml).unwrap();
}

#[test]
fn manymany_shops_validate() {
    let mut i = 0;
    loop {
        i += 1;
        println!("{i}");
        if i > 10 {
            break;
        }
        let mut buf = String::new();
        let mut s = quick_xml::se::Serializer::new(&mut buf);
        s.indent(' ', 4);
        fake::Faker.fake::<Shop>().serialize(s).unwrap();
        // println!("{}", &buf);
        validate_xml(&buf).unwrap();
    }
}
