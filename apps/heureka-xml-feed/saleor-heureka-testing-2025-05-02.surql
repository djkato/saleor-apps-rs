-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: categorises
-- ------------------------------

DEFINE TABLE categorises TYPE RELATION IN category OUT product SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON categorises TYPE record<category> PERMISSIONS FULL;
DEFINE FIELD out ON categorises TYPE record<product> PERMISSIONS FULL;



-- ------------------------------
-- TABLE: category
-- ------------------------------

DEFINE TABLE category TYPE NORMAL SCHEMALESS PERMISSIONS NONE;




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

DEFINE FIELD in ON varies TYPE record<variant> PERMISSIONS FULL;
DEFINE FIELD out ON varies TYPE record<product> PERMISSIONS FULL;



