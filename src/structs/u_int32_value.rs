#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UInt32Value {
    value: Option<u32>,
}
impl UInt32Value {
    #[inline]
    pub(crate) fn value(&self) -> u32 {
        self.value.unwrap_or(0)
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use value()")]
    pub(crate) fn get_value(&self) -> u32 {
        self.value()
    }

    #[inline]
    pub(crate) fn value_string(&self) -> String {
        self.value().to_string()
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use value_string()")]
    pub(crate) fn get_value_string(&self) -> String {
        self.value_string()
    }

    #[inline]
    pub(crate) fn set_value(&mut self, value: u32) -> &mut Self {
        self.value = Some(value);
        self
    }

    #[inline]
    pub(crate) fn set_value_string<S: Into<String>>(&mut self, value: S) -> &mut Self {
        // Reader input is UNTRUSTED: a malformed or wrongly-typed attribute
        // must never abort the process. One real case: ECMA-376
        // `dataField/@subtotal` is a string enum ("count"), and typing it
        // as a numeric value made every such pivot workbook panic the whole
        // import. Ignore what will not parse and leave the field at its
        // default, which is how Excel itself treats junk attributes.
        if let Ok(v) = value.into().parse::<u32>() {
            self.set_value(v);
        }
        self
    }

    #[inline]
    pub(crate) fn remove_value(&mut self) -> &mut Self {
        self.value = None;
        self
    }

    #[inline]
    pub(crate) fn has_value(&self) -> bool {
        self.value.is_some()
    }

    #[inline]
    pub(crate) fn hash_string(&self) -> String {
        if self.has_value() {
            return self.value_string();
        }
        String::from("empty!!")
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use hash_string()")]
    pub(crate) fn get_hash_string(&self) -> String {
        self.hash_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reader input is untrusted. Before this, `set_value_string` unwrapped the
    /// parse, so ONE wrongly-typed attribute anywhere in a workbook aborted the
    /// whole process — `dataField/@subtotal="count"` is the case that found it.
    /// All seven numeric value types shared the identical `.unwrap()`.
    #[test]
    fn malformed_input_is_ignored_rather_than_panicking() {
        for junk in ["count", "", "percentOfTotal", "3.5", "-1", "n/a", "1e5"] {
            let mut v = UInt32Value::default();
            v.set_value_string(junk);
            assert!(!v.has_value(), "{junk:?} must not set a value");
        }
    }

    /// Leniency must not cost correctness: well-formed input still parses.
    #[test]
    fn well_formed_input_still_parses() {
        let mut v = UInt32Value::default();
        v.set_value_string("67");
        assert!(v.has_value());
        assert_eq!(v.value(), 67);
    }
}
