use std::fmt;

/// Relative importance of the three criteria; only the ratios matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weights {
    pub semantic: u16,
    pub readability: u16,
    pub originality: u16,
}

/// Every number the scoring uses. `Default` is the one place they are written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringModel {
    pub weights: Weights,
    pub originality_composed: u8,
    pub originality_coined: u8,
    /// A coined name's originality blends its form value with its morphological quality.
    pub coined_blend_form: u16,
    pub coined_blend_quality: u16,
    pub semantic_base: u64,
    pub semantic_per_match: u64,
    /// Characters a name may have before the length penalty starts.
    pub length_free: usize,
    pub length_penalty: i32,
    pub consonant_penalty: i32,
    pub repetition_penalty: i32,
}

impl Default for ScoringModel {
    fn default() -> Self {
        Self {
            weights: Weights {
                semantic: 45,
                readability: 30,
                originality: 25,
            },
            originality_composed: 70,
            originality_coined: 90,
            coined_blend_form: 55,
            coined_blend_quality: 45,
            semantic_base: 40,
            semantic_per_match: 12,
            length_free: 12,
            length_penalty: 2,
            consonant_penalty: 7,
            repetition_penalty: 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoringError {
    Line(usize, String),
}

impl fmt::Display for ScoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Line(line, why) = self;
        write!(f, "scoring model, line {line}: {why}")
    }
}

impl std::error::Error for ScoringError {}

impl Weights {
    fn sum(&self) -> u32 {
        u32::from(self.semantic) + u32::from(self.readability) + u32::from(self.originality)
    }

    /// Weighted mean of the three scores, rounded to nearest.
    pub fn total(&self, semantic: u8, readability: u8, originality: u8) -> u8 {
        let sum = self.sum().max(1);
        let weighted = u32::from(self.semantic) * u32::from(semantic)
            + u32::from(self.readability) * u32::from(readability)
            + u32::from(self.originality) * u32::from(originality);
        ((weighted + sum / 2) / sum) as u8
    }
}

impl ScoringModel {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.weights.sum() == 0 {
            return Err("score weights must not all be zero");
        }
        if u32::from(self.coined_blend_form) + u32::from(self.coined_blend_quality) == 0 {
            return Err("coined blend weights must not both be zero");
        }
        Ok(())
    }

    /// `key=value` lines over the defaults; `#` comments and blank lines are ignored.
    pub fn from_text(text: &str) -> Result<Self, ScoringError> {
        let mut model = Self::default();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let at = |why: String| ScoringError::Line(index + 1, why);
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| at("expected key=value".into()))?;
            let (key, value) = (key.trim(), value.trim());
            let number: i64 = value
                .parse()
                .map_err(|_| at(format!("{key}: not a number: {value:?}")))?;
            model.set(key, number).map_err(at)?;
        }
        Ok(model)
    }

    fn set(&mut self, key: &str, n: i64) -> Result<(), String> {
        let out_of_range = || format!("{key}: out of range: {n}");
        match key {
            "weight_semantic" => {
                self.weights.semantic = n.try_into().map_err(|_| out_of_range())?
            }
            "weight_readability" => {
                self.weights.readability = n.try_into().map_err(|_| out_of_range())?
            }
            "weight_originality" => {
                self.weights.originality = n.try_into().map_err(|_| out_of_range())?
            }
            "originality_composed" => {
                self.originality_composed = n.try_into().map_err(|_| out_of_range())?
            }
            "originality_coined" => {
                self.originality_coined = n.try_into().map_err(|_| out_of_range())?
            }
            "coined_blend_form" => {
                self.coined_blend_form = n.try_into().map_err(|_| out_of_range())?
            }
            "coined_blend_quality" => {
                self.coined_blend_quality = n.try_into().map_err(|_| out_of_range())?
            }
            "semantic_base" => self.semantic_base = n.try_into().map_err(|_| out_of_range())?,
            "semantic_per_match" => {
                self.semantic_per_match = n.try_into().map_err(|_| out_of_range())?
            }
            "length_free" => self.length_free = n.try_into().map_err(|_| out_of_range())?,
            "length_penalty" => self.length_penalty = n.try_into().map_err(|_| out_of_range())?,
            "consonant_penalty" => {
                self.consonant_penalty = n.try_into().map_err(|_| out_of_range())?
            }
            "repetition_penalty" => {
                self.repetition_penalty = n.try_into().map_err(|_| out_of_range())?
            }
            _ => return Err(format!("unknown key {key}")),
        }
        Ok(())
    }
}
