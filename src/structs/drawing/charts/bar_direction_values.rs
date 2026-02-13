use std::str::FromStr;

use super::super::super::EnumTrait;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub enum BarDirectionValues {
    #[default]
    Bar,
    Column,
}
impl EnumTrait for BarDirectionValues {
    fn value_string(&self) -> &str {
        match &self {
            Self::Bar => "bar",
            Self::Column => "col",
        }
    }
}
impl FromStr for BarDirectionValues {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "bar" => Ok(Self::Bar),
            "col" => Ok(Self::Column),
            _ => Err(()),
        }
    }
}
