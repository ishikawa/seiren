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

    /// Returns the distance from this `Point` to a specified point.
    pub fn distance(&self, other: &Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
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

/// `Path` is an analogue of SVG `<path>` element without visual properties.
/// It consists of an array of `PathCommand`. See SVG specification for more
/// details about commands.
#[derive(Debug, Clone)]
pub struct Path {
    commands: Vec<PathCommand>,
}

impl Path {
    /// Build a new `Path`.
    ///
    /// - `start_point` - You must supply the start point. A `Path` must contain at least
    ///                   one `MoveTo` command.
    pub fn new(start_point: Point) -> Self {
        Self {
            commands: vec![PathCommand::MoveTo(start_point)],
        }
    }

    pub fn commands(&self) -> impl ExactSizeIterator<Item = &PathCommand> {
        self.commands.iter()
    }

    pub fn move_to(&mut self, point: Point) {
        self.commands.push(PathCommand::MoveTo(point));
    }

    pub fn line_to(&mut self, point: Point) {
        self.commands.push(PathCommand::LineTo(point));
    }

    pub fn quad_to(&mut self, ctrl: Point, to: Point) {
        self.commands.push(PathCommand::QuadTo(ctrl, to));
    }

    pub fn start_point(&self) -> &Point {
        let Some(PathCommand::MoveTo(pt)) = self.commands.get(0) else {
            panic!("A `Path` must contain at least one `MoveTo` command.")
        };

        pt
    }

    pub fn end_point(&self) -> &Point {
        let last_command = self.commands.last().unwrap();

        match last_command {
            PathCommand::MoveTo(pt) => pt,
            PathCommand::LineTo(pt) => pt,
            PathCommand::QuadTo(_, pt) => pt,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    /// Set the beginning of the next contour to the point.
    MoveTo(Point),
    /// Add a line from the last point to the specified point (x, y).
    LineTo(Point),
    /// Add a quadratic bezier from the last point.
    QuadTo(Point, Point),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let pt1 = Point::new(-1.0, -1.0);
        let pt2 = Point::new(1.0, 1.0);

        assert_eq!(pt1.distance(&pt2), 2.8284271247461903);
        assert_eq!(pt1.distance(&pt2), pt2.distance(&pt1));
    }
}
