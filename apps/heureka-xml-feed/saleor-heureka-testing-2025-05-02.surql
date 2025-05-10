-- ------------------------------
-- OPTION
-- ------------------------------

-- OPTION IMPORT;

-- ------------------------------
-- TABLE: categorises
-- ------------------------------

DEFINE TABLE IF NOT EXISTS categorises TYPE RELATION IN category OUT product SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS in ON categorises TYPE record<category> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON categorises TYPE record<product> PERMISSIONS FULL;



-- ------------------------------
-- TABLE: category
-- ------------------------------

DEFINE TABLE IF NOT EXISTS category TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: product
-- ------------------------------

DEFINE TABLE IF NOT EXISTS product TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: variant
-- ------------------------------

DEFINE TABLE IF NOT EXISTS variant TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




-- ------------------------------
-- TABLE: varies
-- ------------------------------

DEFINE TABLE IF NOT EXISTS varies TYPE RELATION IN variant OUT product SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS in ON varies TYPE record<variant> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON varies TYPE record<product> PERMISSIONS FULL;



