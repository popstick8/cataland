use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum Text {
    Literal(String),
    Message { key: String, args: Vec<Text> },
    List { items: Vec<Text> },
}

impl Default for Text {
    fn default() -> Self {
        Self::Literal(String::new())
    }
}

impl From<&str> for Text {
    fn from(key: &str) -> Self {
        Self::Message {
            key: key.into(),
            args: Vec::new(),
        }
    }
}

impl From<&&str> for Text {
    fn from(key: &&str) -> Self {
        Self::from(*key)
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::Message {
            key: "{0}".into(),
            args: vec![Self::Literal(value)],
        }
    }
}

impl From<&String> for Text {
    fn from(value: &String) -> Self {
        Self::from(value.clone())
    }
}

impl From<&Text> for Text {
    fn from(value: &Text) -> Self {
        value.clone()
    }
}

macro_rules! number {
    ($($type:ty),*) => {$(
        impl From<&$type> for Text {
            fn from(value: &$type) -> Self {
                Self::Literal(value.to_string())
            }
        }
    )*};
}

number!(u8, u16, u32, u64, usize, i32, f64);

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(value) => f.write_str(value),
            Self::List { items } => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_str("、")?;
                    }
                    item.fmt(f)?;
                }
                Ok(())
            }
            Self::Message { key, args } => {
                let mut remaining = key.as_str();
                while let Some(start) = remaining.find('{') {
                    f.write_str(&remaining[..start])?;
                    let rest = &remaining[start + 1..];
                    if let Some(end) = rest.find('}')
                        && let Ok(index) = rest[..end].parse::<usize>()
                        && let Some(value) = args.get(index)
                    {
                        value.fmt(f)?;
                        remaining = &rest[end + 1..];
                    } else {
                        f.write_str("{")?;
                        remaining = rest;
                    }
                }
                f.write_str(remaining)
            }
        }
    }
}

impl std::error::Error for Text {}

#[macro_export]
macro_rules! text {
    ($key:literal $(, $value:expr)* $(,)?) => {
        $crate::Text::Message {
            key: $key.into(),
            args: vec![$($crate::Text::from(&$value)),*],
        }
    };
}
