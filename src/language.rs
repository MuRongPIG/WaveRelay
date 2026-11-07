use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-CN")]
    #[value(name = "zh-CN")]
    ZhCn,
}

impl Language {
    pub fn text<'a>(self, en: &'a str, zh: &'a str) -> &'a str {
        match self {
            Self::En => en,
            Self::ZhCn => zh,
        }
    }
}
