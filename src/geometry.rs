/// https://developer.mozilla.org/en-US/docs/Web/SVG/Content_type#length
///
/// length used in SVG:
/// ```
/// length ::= number ("em" | "ex" | "px" | "in" | "cm" | "mm" | "pt" | "pc" | "%")?
/// ```
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Length {
    pub value: f32,
    pub unit: Option<LengthUnit>,
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}", self.value)?;
        let Some(unit) = self.unit else { return Ok(()) };
        write!(f, "{}", unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthUnit {
    Pixel,
    Percentage,
}

impl fmt::Display for LengthUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LengthUnit::Pixel => write!(f, "px"),
            LengthUnit::Percentage => write!(f, "%"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: Length,
    pub y: Length,
}

impl Point {
    pub fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    pub fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }
}
