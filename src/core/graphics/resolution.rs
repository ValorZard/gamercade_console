use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Resolution {
    Low,
    Medium,
    High,
}

impl Resolution {
    pub fn width(&self) -> i32 {
        match self {
            Resolution::Low => 320,
            Resolution::Medium => 640,
            Resolution::High => 1280,
        }
    }

    pub fn height(&self) -> i32 {
        match self {
            Resolution::Low => 180,
            Resolution::Medium => 320,
            Resolution::High => 720,
        }
    }

    pub fn total_pixels(&self) -> i32 {
        self.width() * self.height()
    }
}
