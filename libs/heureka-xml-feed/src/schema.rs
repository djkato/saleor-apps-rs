use std::{error::Error, fmt::Display};

use pyo3::{
    Python,
    ffi::c_str,
    types::{PyAnyMethods, PyModule},
};

const HEUREKA_FEED_SCHEMA_2: &str = include_str!("../heureka-feed-2.0.xsd");

#[derive(Debug, Clone)]
pub struct XmlSchemaValidationError(String);

impl Display for XmlSchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for XmlSchemaValidationError {}

pub fn validate_xml(xml: &str) -> Result<(), XmlSchemaValidationError> {
    Python::with_gil(|py| {
        pyo3::prepare_freethreaded_python();
        let activators = PyModule::from_code(
            py,
            c_str!(
                r#"
import xmlschema
import sys

def validate(schema, xml):
    try:
        schema = xmlschema.XMLSchema11(schema)
        schema.validate(xml)
        return
    except Exception as e:
        typ = type(e).__name__
        msg = str(e)
        return f"{typ}: {msg}"
    "#
            ),
            c_str!("activators.py"),
            c_str!("activators"),
        )
        .unwrap();

        let validation_result: Option<String> = activators
            .getattr("validate")
            .unwrap()
            .call((HEUREKA_FEED_SCHEMA_2, xml), None)
            .unwrap()
            .extract()
            .unwrap();
        if let Some(e) = validation_result {
            return Err(XmlSchemaValidationError(e));
        }
        Ok(())
    })
}
