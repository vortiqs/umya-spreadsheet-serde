use std::str::FromStr;

use super::super::super::EnumTrait;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub enum OrientationValues {
    #[default]
    MaxMin,
    MinMax,
}
impl EnumTrait for OrientationValues {
    fn value_string(&self) -> &str {
        match &self {
            Self::MaxMin => "maxMin",
            Self::MinMax => "minMax",
        }
    }
}
impl FromStr for OrientationValues {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "maxMin" => Ok(Self::MaxMin),
            "minMax" => Ok(Self::MinMax),
            _ => Err(()),
        }
    }
}
