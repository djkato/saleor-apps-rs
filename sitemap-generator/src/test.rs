#[cfg(test)]
mod test {
    use crate::sitemap::{Url, UrlSet};

    fn urlset_serialisation_isnt_lossy() {
        let mut url_set = UrlSet::new();
        url_set.url.append(&mut vec![
            Url::new_generic_url("category1coolid".to_string(), "category1".to_string()),
            Url::new_generic_url("category2coolid".to_string(), "category2".to_string()),
            Url::new_product_url(
                "category1coolid".to_string(),
                "category1".to_string(),
                "product1coolid".to_string(),
                "product1".to_string(),
            ),
            Url::new_product_url(
                "category2coolid".to_string(),
                "category2".to_string(),
                "product2coolid".to_string(),
                "product2".to_string(),
            ),
        ]);
        let file_str = url_set.to_file().unwrap();

        let deserialized_url_set: UrlSet = quick_xml::de::from_str(&file_str).unwrap();
        assert_eq!(url_set, deserialized_url_set);
    }
}
