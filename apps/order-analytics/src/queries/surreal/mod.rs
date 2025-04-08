pub const DEFINITIONS: &str = r#"
USE NS saleor;
REMOVE DATABASE order_analytics;
USE DB order_analytics;
-- Define tables and relationships
DEFINE TABLE product SCHEMALESS;
DEFINE TABLE order SCHEMALESS;
"#;

pub const DUMMY_DATA: &str = r#"
CREATE product:1 SET name='JBL Speaker', category='audio';
CREATE product:2 SET name='3.5mm Cable', category='accessories';
CREATE product:3 SET name='Sony Headphones', category='audio';

CREATE order:1 SET timestamp='2024-03-15T10:00:00Z';
RELATE order:1->bought->product:1;
RELATE order:1->bought->product:2;

CREATE order:2 SET timestamp='2024-03-15T10:00:00Z';
RELATE order:2->bought->product:1;
RELATE order:2->bought->product:2;

CREATE order:3 SET timestamp='2024-03-15T10:00:00Z';
RELATE order:3->bought->product:3;
RELATE order:3->bought->product:2;

CREATE order:4 SET timestamp='2024-03-16T11:30:00Z';
RELATE order:4->bought->product:3;
RELATE order:4->bought->product:1;

-- Manual shown relationship
RELATE order:1->saw->product:3;
RELATE order:2->saw->product:3;
RELATE product:1->related->product:2;
RELATE product:3->related->product:1;
"#;

pub const QUERY_OFTEN_BOUGHT_TOGETHER: &str = r#"
SELECT
    count() AS freq,
    id, name
FROM
    product:1<-bought<-order->bought->product
WHERE
    id != product:1
GROUP BY id
ORDER BY freq DESC;
"#;

pub const QUERY_ALTERNATIVE_PRODUCTS: &str = r#"
SELECT
    count() AS freq,
    id, name, category
FROM
    product:3<-saw<-order->bought->product
WHERE
    id != product:3 AND product:3.category == category
GROUP BY id
ORDER BY freq DESC;
"#;

pub const QUERY_RELATED_PRODUCTS: &str = r#"
SELECT
    id, name
FROM product:1->related->product, product:1<-related<-product
;
"#;

pub const QUERY_TRENDING_PRODUCTS: &str = r#"
SELECT 
    id, name, 
    count(orders) AS total_bought FROM (SELECT id, name, <-bought<-(order WHERE timestamp < time::now() + 1w) AS orders FROM product) 
ORDER BY total_bought DESC
;
"#;

pub const QUERY_BESTSELLER_PRODUCTS: &str = r#"
SELECT 
    id, name, count(orders) AS total_bought FROM (SELECT id, name, <-bought<-order AS orders FROM product) 
ORDER BY total_bought DESC
;
"#;
