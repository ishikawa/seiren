use derive_more::Display;
use smallvec::{smallvec, SmallVec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Display)]
#[display(fmt = "({}, {})", x, y)]
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

    /// Returns the direction of a vertical or horizontal line.
    pub fn orthogonal_direction(&self, to: &Point) -> Orientation {
        if to.x < self.x {
            Orientation::Left
        } else if to.x > self.x {
            Orientation::Right
        } else if to.y < self.y {
            Orientation::Up
        } else {
            Orientation::Down
        }
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
    /// If `dx` and `dy` are positive values, then the rectangle???s size is decreased.
    /// If `dx` and `dy` are negative values, the rectangle???s size is increased.
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
    /// or on the minimum X or minimum Y edge.
    pub fn contains_point(&self, point: &Point) -> bool {
        let min_x = self.min_x();
        let max_x = self.max_x();
        let min_y = self.min_y();
        let max_y = self.max_y();

        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
    }

    /// Returns `true` if a line `a` to `b` intersects the rectangle.
    ///
    /// Implementation details
    /// ----------------------
    ///
    /// The Liang???Barsky algorithm
    /// https://en.wikipedia.org/wiki/Liang???Barsky_algorithm
    ///
    /// > This algorithm is significantly more efficient than Cohen???Sutherland.
    /// > https://en.wikipedia.org/wiki/Cohen???Sutherland_algorithm
    ///
    /// ```svgbob
    ///                ^(x_0 + ??_x, y_0 + ??_y)
    ///               /      y_min
    ///            *-/-------------------*
    ///            |/                    |
    ///            /                     |
    ///           /|                     |
    ///          / |                     |
    ///         *  |                     |
    ///  (x_0, y_0)|                     |
    ///            |                     |
    ///            |                     |
    ///            |                     |
    ///      x_min *---------------------* x_max
    ///                     y_max
    /// ```
    pub fn intersects_line(&self, a: &Point, b: &Point) -> bool {
        self.intersected_line(a, b).is_some()
    }

    pub fn intersected_line(&self, a: &Point, b: &Point) -> Option<(Point, Point)> {
        let (x, y, dx, dy) = if b.x < a.x {
            (b.x, b.y, a.x - b.x, a.y - b.y)
        } else {
            (a.x, a.y, b.x - a.x, b.y - a.y)
        };

        let left = self.min_x();
        let right = self.max_x();
        let top = self.min_y();
        let bottom = self.max_y();

        if x > left && (x + dx) < right && y > top && (y + dy) < bottom {
            // Line is entirely inside the rectangle.
            return None;
        }

        let p1 = -dx;
        let p2 = -p1;
        let p3 = -dy;
        let p4 = -p3;

        let q1 = x - left;
        let q2 = right - x;
        let q3 = y - top;
        let q4 = bottom - y;

        if (p1 == 0.0 && q1 < 0.0)
            || (p2 == 0.0 && q2 < 0.0)
            || (p3 == 0.0 && q3 < 0.0)
            || (p4 == 0.0 && q4 < 0.0)
        {
            // Line is parallel to rectangle.
            return None;
        }

        let mut posarr: SmallVec<[f32; 3]> = smallvec![1.0];
        let mut negarr: SmallVec<[f32; 3]> = smallvec![0.0];

        posarr.push(1.0);
        negarr.push(0.0);

        if p1 != 0.0 {
            let r1 = q1 / p1;
            let r2 = q2 / p2;

            if p1 < 0.0 {
                negarr.push(r1); // for negative p1, add it to negative array
                posarr.push(r2); // and add p2 to positive array
            } else {
                negarr.push(r2);
                posarr.push(r1);
            }
        }

        if p3 != 0.0 {
            let r3 = q3 / p3;
            let r4 = q4 / p4;
            if p3 < 0.0 {
                negarr.push(r3);
                posarr.push(r4);
            } else {
                negarr.push(r4);
                posarr.push(r3);
            }
        }

        let rn1 = negarr.iter().fold(f32::NAN, |m, v| v.max(m));
        let rn2 = posarr.iter().fold(f32::NAN, |m, v| v.min(m));

        if rn1 > rn2 {
            // Line is outside the rectangle.
            return None;
        }

        // computing collision points
        let xn1 = x + p2 * rn1;
        let yn1 = y + p4 * rn1;
        let xn2 = x + p2 * rn2;
        let yn2 = y + p4 * rn2;

        return Some((Point::new(xn1, yn1), Point::new(xn2, yn2)));
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
    fn point_orthogonal_direction() {
        let pt1 = Point::zero();

        assert_eq!(
            pt1.orthogonal_direction(&Point::new(-1.0, 0.0)),
            Orientation::Left
        );
        assert_eq!(
            pt1.orthogonal_direction(&Point::new(1.0, 0.0)),
            Orientation::Right
        );
        assert_eq!(
            pt1.orthogonal_direction(&Point::new(0.0, -1.0)),
            Orientation::Up
        );
        assert_eq!(
            pt1.orthogonal_direction(&Point::new(0.0, 1.0)),
            Orientation::Down
        );
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

        assert!(r.contains_point(r.origin()));

        let p = Point::new(r.max_x(), r.max_y());
        assert!(r.contains_point(&p));
    }

    #[test]
    fn rect_intersects_line() {
        let r = Rect::new(Point::new(15.0, 5.0), Size::new(30.0, 30.0));

        // (A) The line segment is entirely outside the rectangle
        // ------------------------------------------------------
        //
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !
        //     !  *        *----------------*
        //     !   \       |                |
        //     !    \      |                |
        //     !     \     |                |
        //     !      \    |                |
        //     !       v   |                |
        //     !           |                |
        //     !           *----------------*
        //     v
        // ```
        assert!(!r.intersects_line(&Point::new(5.0, 10.0), &Point::new(10.0, 10.0)));

        // ```svgbob
        //   (0, 0)
        //   - - -*- - - - - - - - - - - - - - ->
        //        !
        //      * !       *----------------*
        //      | !       |                |
        //      | !       |                |
        //      | !       |                |
        //      | !       |                |
        //      v !       |                |
        //        !       |                |
        //        !       *----------------*
        //        v
        // ```
        assert!(!r.intersects_line(&Point::new(-1.0, 5.0), &Point::new(-1.0, 10.0)));

        // (B) The line segment is entirely inside the rectangle
        // -----------------------------------------------------
        //
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |  *             |
        //     !       |   \            |
        //     !       |    \           |
        //     !       |     \          |
        //     !       |      v         |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(!r.intersects_line(&Point::new(20.0, 10.0), &Point::new(25.0, 30.0)));

        // (C) One end of the line segment is inside and the other is outside
        // ------------------------------------------------------------------
        //
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !   ^   *----------------*
        //     !    \  |                |
        //     !     \ |                |
        //     !      \|                |
        //     !       \                |
        //     !       |\               |
        //     !       | *              |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(20.0, 30.0), &Point::new(10.0, 5.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !   <---*----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 5.0), &Point::new(5.0, 5.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !   <---*                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 15.0), &Point::new(5.0, 15.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     !      /              (45, 35)
        //     !     /
        //     !    v
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 35.0), &Point::new(5.0, 50.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *---*------------*
        //     !       |   |         (45, 35)
        //     !       |   |
        //     !       v   v
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 35.0), &Point::new(15.0, 50.0)));
        assert!(r.intersects_line(&Point::new(20.0, 35.0), &Point::new(20.0, 50.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     !        \              (45, 35)
        //     !         \
        //     !          v
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 35.0), &Point::new(20.0, 50.0)));

        // (D) One end of the line segment is outside and the other is inside
        // ------------------------------------------------------------------
        //
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !   *   *----------------*
        //     !    \  |                |
        //     !     \ |                |
        //     !      \|                |
        //     !       \                |
        //     !       |\               |
        //     !       | v              |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(10.0, 5.0), &Point::new(20.0, 30.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !   *-->*----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(5.0, 5.0), &Point::new(15.0, 5.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !   *-->*                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(5.0, 15.0), &Point::new(15.0, 15.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     !      ^              (45, 35)
        //     !     /
        //     !    *
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(5.0, 50.0), &Point::new(15.0, 35.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       o---o------------*
        //     !       ^   ^         (45, 35)
        //     !       |   |
        //     !       *   *
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 50.0), &Point::new(15.0, 35.0)));
        assert!(r.intersects_line(&Point::new(20.0, 50.0), &Point::new(20.0, 35.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       o----------------*
        //     !        ^              (45, 35)
        //     !         \
        //     !          *
        //     v
        // ```
        assert!(r.intersects_line(&Point::new(20.0, 50.0), &Point::new(15.0, 35.0)));

        // (E) The line segment starts outside the rectangle, enters it, and leaves it again
        // ---------------------------------------------------------------------------------
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5) /
        //     !       *---o------------*
        //     !       |  /             |
        //     !       | /              |
        //     !       |/               |
        //     !       o                |
        //     !      /|                |
        //     !     / |                |
        //     !    *  |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(5.0, 30.0), &Point::new(20.0, 0.0)));
        assert!(r.intersects_line(&Point::new(20.0, 0.0), &Point::new(5.0, 30.0)));

        // (F) The line segment's both ends on the edge
        // --------------------------------------------
        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       o--------------->o
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 20.0), &Point::new(45.0, 20.0)));
        assert!(r.intersects_line(&Point::new(45.0, 20.0), &Point::new(15.0, 20.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       o----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       v                |
        //     !       o----------------*
        //     v                     (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 5.0), &Point::new(15.0, 35.0)));
        assert!(r.intersects_line(&Point::new(15.0, 35.0), &Point::new(15.0, 5.0)));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       o-------*
        //     !       |\      |
        //     !       | \     |
        //     !       |  \    |
        //     !       |   \   |
        //     !       |    \  |
        //     !       |     \ |
        //     !       |      v|
        //     !       *-------o
        //     v            (45, 35)
        // ```
        assert!(r.intersects_line(&Point::new(15.0, 5.0), &Point::new(45.0, 35.0)));
        assert!(r.intersects_line(&Point::new(45.0, 35.0), &Point::new(15.0, 5.0)));
    }

    #[test]
    fn rect_intersected_line() {
        let r = Rect::new(Point::new(15.0, 5.0), Size::new(30.0, 30.0));

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       o----------------*
        //     !        ^              (45, 35)
        //     !         \
        //     !          *
        //     v
        // ```
        let intersected = r.intersected_line(&Point::new(20.0, 50.0), &Point::new(15.0, 35.0));

        assert!(intersected.is_some());
        assert_eq!(
            intersected.unwrap(),
            (Point::new(15.0, 35.0), Point::new(15.0, 35.0))
        );

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - - - - - - - - - ->
        //     !    (15, 5)
        //     !   *   *----------------*
        //     !    \  |                |
        //     !     \ |                |
        //     !      \|                |
        //     !       \                |
        //     !       |\               |
        //     !       | v              |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        let intersected = r.intersected_line(&Point::new(10.0, 5.0), &Point::new(20.0, 30.0));

        assert!(intersected.is_some());
        assert_eq!(
            intersected.unwrap(),
            (Point::new(15.0, 17.5), Point::new(20.0, 30.0))
        );

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       *----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       o--------------->o
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       *----------------*
        //     v                     (45, 35)
        // ```
        let intersected = r.intersected_line(&Point::new(15.0, 20.0), &Point::new(45.0, 20.0));

        assert!(intersected.is_some());
        assert_eq!(
            intersected.unwrap(),
            (Point::new(15.0, 20.0), Point::new(45.0, 20.0))
        );

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       o----------------*
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       |                |
        //     !       v                |
        //     !       o----------------*
        //     v                     (45, 35)
        // ```
        let intersected = r.intersected_line(&Point::new(15.0, 5.0), &Point::new(15.0, 35.0));

        assert!(intersected.is_some());
        assert_eq!(
            intersected.unwrap(),
            (Point::new(15.0, 5.0), Point::new(15.0, 35.0))
        );

        // ```svgbob
        //   (0, 0)
        //     *- - - - - - -^ - - - - - - - ->
        //     !    (15, 5)
        //     !       o-------*
        //     !       |\      |
        //     !       | \     |
        //     !       |  \    |
        //     !       |   \   |
        //     !       |    \  |
        //     !       |     \ |
        //     !       |      v|
        //     !       *-------o
        //     v            (45, 35)
        // ```
        let intersected = r.intersected_line(&Point::new(15.0, 5.0), &Point::new(45.0, 35.0));

        assert!(intersected.is_some());
        assert_eq!(
            intersected.unwrap(),
            (Point::new(15.0, 5.0), Point::new(45.0, 35.0))
        );
    }
}
