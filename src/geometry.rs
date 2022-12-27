/*
use std::fmt;

/// https://developer.mozilla.org/en-US/docs/Web/SVG/Content_type#length
///
/// length used in SVG:
/// ```ignore
/// length ::= number ("em" | "ex" | "px" | "in" | "cm" | "mm" | "pt" | "pc" | "%")?
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Length {
    pub value: f32,
    pub unit: Option<LengthUnit>,
}

impl Length {
    pub fn new(value: f32, unit: Option<LengthUnit>) -> Self {
        Self { value, unit }
    }

    pub fn add(&self, value: f32) -> Self {
        Self {
            value: self.value + value,
            unit: self.unit,
        }
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Self::new(value as f32, None)
    }
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
*/

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Corners and centers in a rectangle.
///
/// ```svgbob
///           minX    midX    maxX
///   (origin) *----------*----------*
///            |                     |
///            |                     |
///            * (center) *          * midY
///            |                     |
///            |                     |
///            *----------*----------* maxY
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn center(&self) -> Point {
        Point::new(self.mid_x(), self.mid_y())
    }

    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    pub fn mid_x(&self) -> f32 {
        self.origin.x + self.size.width / 2.0
    }

    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    pub fn mid_y(&self) -> f32 {
        self.origin.y + self.size.height / 2.0
    }

    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }
}
