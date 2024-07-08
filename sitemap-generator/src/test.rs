#[cfg(test)]
mod test {
    use crate::sitemap::{RefType, Url, UrlSet};

    fn urlset_serialisation_isnt_lossy() {
        let mut url_set = UrlSet::new();
        url_set.url.append(&mut vec![
            Url::new(
                "category1coolid".to_string(),
                "category1".to_string(),
                RefType::Category,
            ),
            Url::new(
                "Collection1".to_string(),
                "Collection1coolid".to_string(),
                RefType::Collection,
            ),
            Url::new_with_ref(
                "category1coolid".to_string(),
                "category1".to_string(),
                RefType::Product,
                Some("product1coolid".to_string()),
                Some("product1".to_string()),
                Some(RefType::Category),
            ),
            Url::new_with_ref(
                "category2coolid".to_string(),
                "category2".to_string(),
                RefType::Product,
                Some("product2coolid".to_string()),
                Some("product2".to_string()),
                Some(RefType::Category),
            ),
        ]);
        let file_str = url_set.to_file().unwrap();

        let deserialized_url_set: UrlSet = quick_xml::de::from_str(&file_str).unwrap();
        assert_eq!(url_set, deserialized_url_set);
    }
}
