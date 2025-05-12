use super::Error;
use pyo3::{
    Python,
    ffi::c_str,
    types::{PyAnyMethods, PyModule},
};

const HEUREKA_FEED_SCHEMA_2: &str = include_str!("../heureka-feed-2.0.xsd");

pub fn validate_xml(xml: &str) -> Result<(), Error> {
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
            return Err(Error::XmlValidationError(e));
        }
        Ok(())
    })
}
