use std::fmt;

use crate::kv::value;

impl<'v> value::Value<'v> {
    /// Create a value.
    pub fn from_debug(v: &'v impl fmt::Debug) -> Self {
        Self::any(v, |v, visit| visit.debug(v))
    }
}

impl<'v> fmt::Debug for value::Value<'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut visitor = FmtBackend(f);
        self.0.visit(value::Visitor(&mut visitor)).map_err(|_| fmt::Error)
    }
}

impl<'v> fmt::Display for value::Value<'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> value::Visitor<'a> {
    /// Visit a value that can be formatted.
    pub fn debug(self, v: impl fmt::Debug) -> Result<(), value::Error> {
        self.0.debug(&v)
    }
}

pub(in crate::kv::value) trait Backend {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), value::Error>;
}

pub(in crate::kv::value) struct FmtBackend<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> value::Backend for FmtBackend<'a, 'b> {
    fn u64(&mut self, v: u64) -> Result<(), value::Error> {
        self.debug(&v)
    }
}

impl<'a, 'b> Backend for FmtBackend<'a, 'b> {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), value::Error> {
        write!(self.0, "{:?}", v).map_err(|_| value::Error::msg("formatting failed"))?;

        Ok(())
    }
}
