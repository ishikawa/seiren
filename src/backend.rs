//! Backends translate MIR into graphics format.
use std::io::Write;

use crate::{error::BackendError, mir};

pub trait Backend {
    fn generate(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError>;
}

#[derive(Debug)]
pub struct SVGBackend {}

impl SVGBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for SVGBackend {
    fn generate(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError> {
        writer.write_all("".as_bytes())?;
        Ok(())
    }
}
