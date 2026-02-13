use std::str::FromStr;

use super::super::super::EnumTrait;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub enum LayoutModeValues {
    #[default]
    Edge,
    Factor,
}
impl EnumTrait for LayoutModeValues {
    fn value_string(&self) -> &str {
        match &self {
            Self::Edge => "edge",
            Self::Factor => "factor",
        }
    }
}
impl FromStr for LayoutModeValues {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "edge" => Ok(Self::Edge),
            "factor" => Ok(Self::Factor),
            _ => Err(()),
        }
    }
}
