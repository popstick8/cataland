use crate::Text;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Language {
    #[serde(rename = "zh-CN")]
    Chinese,
    #[serde(rename = "en")]
    English,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub master: f64,
    pub music: f64,
    pub effects: f64,
    pub ambience: f64,
    pub scale: f64,
    pub animation: f64,
    pub language: Language,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            master: 0.7,
            music: 0.35,
            effects: 0.7,
            ambience: 0.3,
            scale: 1.0,
            animation: 1.0,
            language: Language::Chinese,
        }
    }
}

impl Preferences {
    pub fn validate(&self) -> Result<(), Text> {
        if ![
            self.master,
            self.music,
            self.effects,
            self.ambience,
            self.animation,
        ]
        .into_iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
        {
            return Err("音量与动画强度需要在 0–100% 之间".into());
        }
        if !self.scale.is_finite() || !(0.75..=1.5).contains(&self.scale) {
            return Err("界面缩放需要在 75–150% 之间".into());
        }
        Ok(())
    }
}
