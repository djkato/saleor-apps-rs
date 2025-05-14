-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: categorises
-- ------------------------------

DEFINE TABLE categorises TYPE RELATION IN category OUT product SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON categorises TYPE record<category> PERMISSIONS FULL;
DEFINE FIELD out ON categorises TYPE record<product> ASSERT IF (SELECT * FROM categorises WHERE out = $value).is_empty() { RETURN true; } ELSE { THROW 'product already has a relation'; } PERMISSIONS FULL;



-- ------------------------------
-- TABLE: category
-- ------------------------------

DEFINE TABLE category TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: parents
-- ------------------------------

DEFINE TABLE parents TYPE RELATION IN category OUT category SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON parents TYPE record<category> ASSERT IF (SELECT * FROM parents WHERE in = $value).is_empty() { RETURN true; } ELSE { THROW 'category already has a relation'; } PERMISSIONS FULL;
DEFINE FIELD out ON parents TYPE record<category> ASSERT IF (SELECT * FROM parents WHERE out = $value).is_empty() { RETURN true; } ELSE { THROW 'category already has a relation'; } PERMISSIONS FULL;



-- ------------------------------
-- TABLE: product
-- ------------------------------

DEFINE TABLE product TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: variant
-- ------------------------------

DEFINE TABLE variant TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: varies
-- ------------------------------

DEFINE TABLE varies TYPE RELATION IN variant OUT product SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON varies TYPE record<variant> ASSERT IF (SELECT * FROM varies WHERE in = $value).is_empty() { RETURN true; } ELSE { THROW 'variant already has a relation'; } PERMISSIONS FULL;
DEFINE FIELD out ON varies TYPE record<product> PERMISSIONS FULL;


-- -----------------------------
-- TABLE: shipping_zone
-- -----------------------------

DEFINE TABLE shipping_zone TYPE NORMAL SCHEMALESS PERMISSIONS NONE;

-- -----------------------------
-- TABLE: issues 
-- -----------------------------

DEFINE TABLE issues TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
