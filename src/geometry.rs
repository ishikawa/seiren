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
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
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
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
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
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            origin: Point::zero(),
            size: Size::zero(),
        }
    }

    #[inline]
    pub fn origin(&self) -> &Point {
        &self.origin
    }

    #[inline]
    pub fn size(&self) -> &Size {
        &self.size
    }

    #[inline]
    pub fn center(&self) -> Point {
        Point::new(self.mid_x(), self.mid_y())
    }

    #[inline]
    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    #[inline]
    pub fn mid_x(&self) -> f32 {
        self.origin.x + self.size.width / 2.0
    }

    #[inline]
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    #[inline]
    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    #[inline]
    pub fn mid_y(&self) -> f32 {
        self.origin.y + self.size.height / 2.0
    }

    #[inline]
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.size.width
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.size.height
    }

    /// Returns a rectangle that is smaller or larger than the source
    /// rectangle, with the same center point.
    ///
    /// A rectangle. The origin value is offset in the x-axis by the distance specified by
    /// the `dx` parameter and in the y-axis by the distance specified by the `dy` parameter,
    /// and its size adjusted by (`2*dx`, `2*dy`), relative to the source rectangle.
    /// If `dx` and `dy` are positive values, then the rectangle’s size is decreased.
    /// If `dx` and `dy` are negative values, the rectangle’s size is increased.
    ///
    /// If the resulting rectangle would have a negative height or width,
    /// a returned rectangle has a zero size.
    ///
    /// - `dx` - The x-coordinate value to use for adjusting the source rectangle.
    ///          To create an inset rectangle, specify a positive value.
    ///          To create a larger, encompassing rectangle, specify a negative value.
    /// - `dy` - The y-coordinate value to use for adjusting the source rectangle.
    ///          To create an inset rectangle, specify a positive value.
    ///          To create a larger, encompassing rectangle, specify a negative value.
    pub fn inset_by(&self, dx: f32, dy: f32) -> Self {
        let origin = Point::new(self.origin.x + dx, self.origin.y + dy);
        let size = Size::new(
            (self.size.width - (dx * 2.0)).max(0.0),
            (self.size.height - (dy * 2.0)).max(0.0),
        );

        Self::new(origin, size)
    }

    /// Returns whether a rectangle contains a specified point.
    ///
    /// `true` if the rectangle is not empty and the point is located within the rectangle;
    /// otherwise, `false`.
    ///
    /// A point is considered inside the rectangle if its coordinates lie inside the rectangle,
    /// or on the minimum X or minimum Y edge if and only if `include_edge` is `true`.
    pub fn contains_point(&self, point: &Point, include_edge: bool) -> bool {
        let min_x = self.min_x();
        let max_x = self.max_x();
        let min_y = self.min_y();
        let max_y = self.max_y();

        if point.x > min_x && point.x < max_x && point.y > min_y && point.y < max_y {
            return true;
        }

        return include_edge
            && (point.x == min_x || point.x == max_x || point.y == min_y || point.y == max_y);
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

        let pt1 = Point::zero();
        let pt2 = Point::new(3.0, 0.0);

        assert_eq!(pt1.distance(&pt2), 3.0);
    }

    #[test]
    fn rect_inset_by() {
        let r = Rect::new(Point::new(10.0, 20.0), Size::new(50.0, 50.0));

        assert_eq!(r.inset_by(0.0, 0.0), r);
        assert_eq!(
            r.inset_by(5.0, -10.0),
            Rect::new(Point::new(15.0, 10.0), Size::new(40.0, 70.0))
        );
        assert_eq!(
            r.inset_by(30.0, 30.0),
            Rect::new(Point::new(40.0, 50.0), Size::zero())
        );

        let r = Rect::new(Point::new(f32::MIN, f32::MIN), Size::new(1.0, 1.0));
        assert_eq!(r.inset_by(f32::MAX, f32::MAX), Rect::zero());
    }

    #[test]
    fn rect_contains_point() {
        let r = Rect::new(Point::new(10.0, 20.0), Size::new(50.0, 50.0));

        assert!(r.contains_point(r.origin(), true));
        assert!(!r.contains_point(r.origin(), false));

        let p = Point::new(r.max_x(), r.max_y());
        assert!(r.contains_point(&p, true));
        assert!(!r.contains_point(&p, false));
    }
}
