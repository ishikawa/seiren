//! Layout engine
use derive_more::Display;

use crate::{
    geometry::{Direction, Path, Point, Size},
    mir::{self, ConnectionPoint, NodeKind},
};
use std::collections::{HashMap, VecDeque};

pub trait LayoutEngine {
    /// Place all nodes on 2D coordination.
    ///
    /// The engine must assign `origin` and `size` of all nodes.
    fn place_nodes(&mut self, doc: &mut mir::Document);

    /// Place all connection points for every node.
    ///
    /// The engine must add all possible connection points to `connection_points` of nodes.
    fn place_connection_points(&mut self, doc: &mut mir::Document);

    /// Draw path between both ends (connection points) of each edge.
    ///
    /// The engine must build a `path` of edges.
    fn draw_edge_path(&mut self, doc: &mut mir::Document);
}

/// Represents routes in a place by graph. Every junction of two edges will be a node of the graph.
/// Neighboring junctions are connected by edges. Each nodes neighbors four other nodes and each
/// edge is NOT directed so shared by two junctions.
#[derive(Debug, Clone)]
pub struct RouteGraph {
    nodes: Vec<RouteNode>,
    // We use adjacency list as our primary data structure to represent graphs because a graph is
    // relatively sparse.
    edges: HashMap<RouteNodeId, Vec<RouteEdge>>,
}

impl RouteGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &RouteNode> {
        self.nodes.iter()
    }

    pub fn get_node(&self, id: RouteNodeId) -> Option<&RouteNode> {
        self.nodes.get(id.0)
    }

    pub fn add_node(&mut self, location: Point) -> RouteNodeId {
        self._add_node(location, None)
    }

    pub fn add_connection_point(&mut self, connection_point: &ConnectionPoint) -> RouteNodeId {
        self._add_node(
            connection_point.location().clone(),
            Some(connection_point.direction().clone()),
        )
    }

    fn _add_node(&mut self, location: Point, direction: Option<Direction>) -> RouteNodeId {
        let node_id = RouteNodeId(self.nodes.len());
        let node = RouteNode::new(node_id, location, direction);

        self.nodes.push(node);
        node_id
    }

    pub fn edges(
        &self,
        node_id: &RouteNodeId,
    ) -> Option<impl ExactSizeIterator<Item = &RouteEdge>> {
        self.edges.get(node_id).map(|x| x.iter())
    }

    pub fn add_edge(&mut self, a: RouteNodeId, b: RouteNodeId) {
        for (from, to) in [(a, b)] {
            self.edges
                .entry(from)
                .and_modify(|v| {
                    if !v.iter().any(|e| e.dest == to) {
                        v.push(RouteEdge::new(to));
                    }
                })
                .or_insert(vec![RouteEdge::new(to)]);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub struct RouteNodeId(usize);

#[derive(Debug, Clone)]
pub struct RouteNode {
    id: RouteNodeId,
    location: Point,

    /// If the node is a connection point, copy its `direction` to
    /// detect edge connectivity.
    direction: Option<Direction>,
}

impl RouteNode {
    pub fn new(id: RouteNodeId, location: Point, direction: Option<Direction>) -> Self {
        Self {
            id,
            location,
            direction,
        }
    }

    pub fn id(&self) -> RouteNodeId {
        self.id
    }

    pub fn location(&self) -> &Point {
        &self.location
    }

    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    /// Returns `true` if `node.direction` is `None` or `direction`.
    pub fn is_connectable(&self, direction: Direction) -> bool {
        match self.direction {
            None => true,
            Some(d) => d == direction,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteEdge {
    dest: RouteNodeId,
}

impl RouteEdge {
    pub fn new(dest: RouteNodeId) -> Self {
        Self { dest }
    }

    pub fn dest(&self) -> RouteNodeId {
        self.dest
    }
}

#[derive(Debug)]
pub struct SimpleLayoutEngine {
    // for debug
    edge_route_graph: RouteGraph,
}

impl SimpleLayoutEngine {
    pub fn new() -> Self {
        Self {
            edge_route_graph: RouteGraph::new(),
        }
    }
}

impl SimpleLayoutEngine {
    const ORIGIN: Point = Point::new(50.0, 80.0);
    const LINE_HEIGHT: f32 = 35.0;
    const RECORD_WIDTH: f32 = 300.0;
    const RECORD_SPACE: f32 = 80.0;

    // for debug
    pub fn edge_route_graph(&self) -> &RouteGraph {
        &self.edge_route_graph
    }
}

impl LayoutEngine for SimpleLayoutEngine {
    fn place_nodes(&mut self, doc: &mut mir::Document) {
        let x = Self::ORIGIN.x;
        let y = Self::ORIGIN.y;

        // Iterate records
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (record_index, child_id) in child_id_vec.iter().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let NodeKind::Record(_) = record_node.kind() else  { continue };

            let n_records = record_node.children().len() as f32;
            let x = x + (Self::RECORD_WIDTH + Self::RECORD_SPACE) * record_index as f32;

            let record_height = Self::LINE_HEIGHT * n_records;

            record_node.origin = Some(Point::new(x, y));
            record_node.size = Some(Size::new(Self::RECORD_WIDTH.into(), record_height.into()));

            // children
            let base_y = y;
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_id) in field_id_vec.iter().enumerate() {
                let y = base_y + Self::LINE_HEIGHT * field_index as f32;
                let Some(field_node) = doc.get_node_mut(field_node_id) else { continue };
                let NodeKind::Field(_) = field_node.kind() else  { continue };

                field_node.origin = Some(Point::new(x, y));
                field_node.size = Some(Size::new(Self::RECORD_WIDTH, Self::LINE_HEIGHT));
            }
        }
    }

    fn place_connection_points(&mut self, doc: &mut mir::Document) {
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (_, child_id) in child_id_vec.iter().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let Some(record_rect) = record_node.rect() else { continue };

            // In the case of a rectangle, connection points are placed in
            // the center of each of the four edges.
            for (x, y, d) in [
                (record_rect.mid_x(), record_rect.min_y(), Direction::Up),
                (record_rect.max_x(), record_rect.mid_y(), Direction::Right),
                (record_rect.mid_x(), record_rect.max_y(), Direction::Down),
                (record_rect.min_x(), record_rect.mid_y(), Direction::Left),
            ] {
                record_node.append_connection_point(ConnectionPoint::new(Point::new(x, y), d));
            }

            // For each field in a rectangle, connection points are placed
            // the center of:
            // - each of the four edges - if the number of fields is `1`.
            // - top, left and right - for the top field
            // - bottom, left and right - for the bottom field
            // - left and right - for the rest
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_id) in field_id_vec.iter().enumerate() {
                let Some(field_node) = doc.get_node_mut(field_node_id) else { continue };
                let Some(field_rect) = field_node.rect() else { continue };

                if field_id_vec.len() == 1 {
                    for (x, y, d) in [
                        (field_rect.mid_x(), field_rect.min_y(), Direction::Up),
                        (field_rect.max_x(), field_rect.mid_y(), Direction::Right),
                        (field_rect.mid_x(), field_rect.max_y(), Direction::Down),
                        (field_rect.min_x(), field_rect.mid_y(), Direction::Left),
                    ] {
                        field_node
                            .append_connection_point(ConnectionPoint::new(Point::new(x, y), d));
                    }
                } else if field_index == 0 {
                    for (x, y, d) in [
                        (field_rect.mid_x(), field_rect.min_y(), Direction::Up),
                        (field_rect.max_x(), field_rect.mid_y(), Direction::Right),
                        (field_rect.min_x(), field_rect.mid_y(), Direction::Left),
                    ] {
                        field_node
                            .append_connection_point(ConnectionPoint::new(Point::new(x, y), d));
                    }
                } else if field_index == (field_id_vec.len() - 1) {
                    for (x, y, d) in [
                        (field_rect.max_x(), field_rect.mid_y(), Direction::Right),
                        (field_rect.mid_x(), field_rect.max_y(), Direction::Down),
                        (field_rect.min_x(), field_rect.mid_y(), Direction::Left),
                    ] {
                        field_node
                            .append_connection_point(ConnectionPoint::new(Point::new(x, y), d));
                    }
                } else {
                    for (x, y, d) in [
                        (field_rect.max_x(), field_rect.mid_y(), Direction::Right),
                        (field_rect.min_x(), field_rect.mid_y(), Direction::Left),
                    ] {
                        field_node
                            .append_connection_point(ConnectionPoint::new(Point::new(x, y), d));
                    }
                }
            }
        }
    }

    /// ```svgbob
    /// 0 - - - - - - - - - - - - - - - - - - - ->
    /// ! -------+
    /// !        |  ctrl1(x)  middle
    /// !  start o--------*--.
    /// !        |           |
    /// !        |           * ctrl1(y)
    /// !        |           |
    /// !        |           |
    /// !        |           |
    /// !        |  ctrl2(y) *           +-------
    /// !        |           | ctrl2(x)  |
    /// !        |           `--*--------o end
    /// v        |                       |
    /// ```
    fn draw_edge_path(&mut self, doc: &mut mir::Document) {
        let path_radius = 6.0;

        let mut paths: VecDeque<Path> = VecDeque::with_capacity(doc.edges().len());

        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };

            // Give the combination with the maximum distance as the initial value, and choose
            // the combination with the shortest distance between two connection points.
            let mut connection: (
                ConnectionPoint, // start point
                ConnectionPoint, // end point
                f32,             // distance
            ) = (
                ConnectionPoint::new(Point::default(), Direction::Left),
                ConnectionPoint::new(Point::default(), Direction::Left),
                f32::MAX,
            );

            for pt1 in start_node.connection_points() {
                for pt2 in end_node.connection_points() {
                    let d = pt1.location().distance(pt2.location());
                    if d < connection.2 {
                        connection = (pt1.clone(), pt2.clone(), d);
                    }
                }
            }

            // Build a path.
            let start_cx = connection.0.location().x;
            let end_cx = connection.1.location().x;
            let start_cy = connection.0.location().y;
            let end_cy = connection.1.location().y;

            let mid_x = start_cx.min(end_cx) + (start_cx - end_cx).abs() / 2.0;

            let (ctrl1_x, ctrl2_x) = if start_cx < end_cx {
                (mid_x - path_radius, mid_x + path_radius)
            } else {
                (mid_x + path_radius, mid_x - path_radius)
            };
            let (ctrl1_y, ctrl2_y) = if start_cy < end_cy {
                (start_cy + path_radius, end_cy - path_radius)
            } else {
                (start_cy - path_radius, end_cy + path_radius)
            };

            // ```svgbob
            // 0 - - - - - - - - - - - - - - - - - - - ->
            // ! -------+
            // !        |       (A)
            // !  start o--------*--.
            // !        |           |
            // !        |           * (B)
            // !        |           |
            // !        |           |
            // !        |           |
            // !        |       (C) *           +-------
            // !        |           | (D))      |
            // !        |           `--*--------o (E)
            // v        |                       |
            // ```
            let mut path = Path::new(*connection.0.location());

            // (A)
            path.line_to(Point::new(ctrl1_x, start_cy));
            // (B)
            path.quad_to(Point::new(mid_x, start_cy), Point::new(mid_x, ctrl1_y));
            // (C)
            path.line_to(Point::new(mid_x, ctrl2_y));
            // (D)
            path.quad_to(Point::new(mid_x, end_cy), Point::new(ctrl2_x, end_cy));
            // (E)
            path.line_to(Point::new(end_cx, end_cy));

            paths.push_back(path);
        }

        for edge in doc.edges_mut() {
            edge.path = Some(paths.pop_front().unwrap());
        }

        // We don't actually draw the edges here, but only calculate the set of points through which
        // the edges pass.
        //
        // EDGE DRAWING ALGORITHM
        // ======================
        //
        // To draw edges between SHAPE nodes, we must develop an algorithm to solve the so-called
        // "Motion Planning" problem.
        //
        // We try to place JUNCTION nodes on the plane where the edges can pass through without
        // intersecting any obstacles and find the shortest path from the start point to the goal.
        //
        // To place JUNCTION nodes around obstacles, we chose the _expanded obstacles_ approach to
        // simplify the problem. We'll create larger, fatter obstacles of each obstacle that defined
        // by the shadow traced as the "moving point" walks a loop around the object while
        // maintaining contact with it.
        //
        // - `SHAPE node` - Rigid shapes that are obstacles. (e.g. Record)
        // - `JUNCTION node` - Virtual nodes that are placed only for edge drawing. Only virtual
        //                     nodes placed **vertically or horizontally** can be joined.
        //
        // Dijkstra's algorithm or A* can be used as the shortest path algorithm.
        //
        // ALGORITHM
        // ---------
        // Place junction nodes on the place:
        //
        // a. For each shape node, create a new larger, fatter shape.
        //
        // b. Place junction nodes at the four corner points of (a)
        //
        // c. From the start/end connection point, draw a straight line horizontally or vertically
        //    until it collides with another shape node, and place a new junction node at the point
        //    where it intersects the junction node (b) in a crosswise direction.
        //
        // d. Remove junction nodes that overlap any (fatter) shapes. However, nodes on the edge of
        //    the shape must remain.
        //
        // e. Add start/end connection points.

        // Place junction nodes at the four corner points around each shape node.
        let shape_junctions = self.edge_junction_nodes_around_shapes(&doc);

        // From the start/end junction point, draw a straight line horizontally or vertically until
        // it collides with another shape node, and place a new junction node at the point where it
        // intersects the junction node in a crosswise direction.
        let mut crossing_junctions: Vec<Point> = vec![];

        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };

            for pt in start_node.connection_points() {
                let junctions = self.edge_junction_nodes_from_connection_point(
                    &doc,
                    start_node,
                    pt,
                    &shape_junctions,
                );

                crossing_junctions.extend(junctions);
            }
            for pt in end_node.connection_points() {
                let junctions = self.edge_junction_nodes_from_connection_point(
                    &doc,
                    end_node,
                    pt,
                    &shape_junctions,
                );

                crossing_junctions.extend(junctions);
            }
        }

        let edge_junctions = self.remove_overlapped_junction_nodes(
            &doc,
            shape_junctions.iter().chain(crossing_junctions.iter()),
        );

        // --- Move junction points to the graph
        for j in edge_junctions {
            self.edge_route_graph.add_node(j);
        }

        // Add start/end connection points.
        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };

            for pt in start_node.connection_points() {
                self.edge_route_graph.add_connection_point(pt);
            }
            for pt in end_node.connection_points() {
                self.edge_route_graph.add_connection_point(pt);
            }
        }

        self.connect_nearest_neighbor_edge_junctions(doc);
    }
}

impl SimpleLayoutEngine {
    const SHAPE_JUNCTION_MARGIN: f32 = Self::RECORD_SPACE / 2.0;

    // a. For each shape node, create a new larger, fatter shape.
    //
    // b. Place junction nodes at the four corner points of (a)
    fn edge_junction_nodes_around_shapes(&self, doc: &mir::Document) -> Vec<Point> {
        let margin = Self::SHAPE_JUNCTION_MARGIN;
        let mut junctions: Vec<Point> = vec![];

        for child_id in doc.body().children() {
            let Some(record_node) = doc.get_node(&child_id) else { continue };
            let Some(record_rect) = record_node.rect() else { continue };

            let junction_rect = record_rect.inset_by(-margin, -margin);

            junctions.extend([
                junction_rect.origin,
                Point::new(junction_rect.max_x(), junction_rect.min_y()),
                Point::new(junction_rect.max_x(), junction_rect.max_y()),
                Point::new(junction_rect.min_x(), junction_rect.max_y()),
            ]);
        }

        junctions
    }

    // c. From the start/end connection point, draw a straight line horizontally or vertically
    //    until it collides with another shape node, and place a new junction node at the point
    //    where it intersects the junction node (b) in a crosswise direction.
    fn edge_junction_nodes_from_connection_point(
        &self,
        doc: &mir::Document,
        _: &mir::Node,
        connection_point: &ConnectionPoint,
        other_junctions: &[Point],
    ) -> Vec<Point> {
        let margin = Self::SHAPE_JUNCTION_MARGIN;
        let mut junctions = vec![];

        let shape_rects = doc
            .body()
            .children()
            .filter_map(|x| doc.get_node(&x))
            .filter_map(|x| x.rect())
            .map(|r| r.inset_by(-margin, -margin))
            .collect::<Vec<_>>();

        let conn_pt = connection_point.location();

        match connection_point.direction() {
            Direction::Left => {
                let mut min_x = 0.0f32;
                let line_end = Point::new(f32::MIN, conn_pt.y);

                for r in shape_rects {
                    if r.max_x() < conn_pt.x && r.intersects_line(conn_pt, &line_end) {
                        min_x = min_x.max(r.max_x());
                    }
                }

                for j in other_junctions {
                    if j.x <= conn_pt.x && j.x >= min_x {
                        junctions.push(Point::new(j.x, conn_pt.y));
                    }
                }
            }
            Direction::Right => {
                let mut max_x = f32::MAX;
                let line_end = Point::new(f32::MAX, conn_pt.y);

                for r in shape_rects {
                    if r.min_x() > conn_pt.x && r.intersects_line(conn_pt, &line_end) {
                        max_x = max_x.min(r.min_x());
                    }
                }

                for j in other_junctions {
                    if j.x >= conn_pt.x && j.x <= max_x {
                        junctions.push(Point::new(j.x, conn_pt.y));
                    }
                }
            }
            Direction::Up => {
                let mut max_y = f32::MAX;
                let line_end = Point::new(conn_pt.x, f32::MAX);

                for r in shape_rects {
                    if r.min_y() > conn_pt.y && r.intersects_line(conn_pt, &line_end) {
                        max_y = max_y.min(r.min_y());
                    }
                }

                for j in other_junctions {
                    if j.y <= conn_pt.y && j.y <= max_y {
                        junctions.push(Point::new(conn_pt.x, j.y));
                    }
                }
            }
            Direction::Down => {
                let mut min_y = 0.0f32;
                let line_end = Point::new(conn_pt.x, f32::MIN);

                for r in shape_rects {
                    if r.max_y() < conn_pt.y && r.intersects_line(conn_pt, &line_end) {
                        min_y = min_y.max(r.max_y());
                    }
                }

                for j in other_junctions {
                    if j.y >= conn_pt.y && j.y >= min_y {
                        junctions.push(Point::new(conn_pt.x, j.y));
                    }
                }
            }
        };

        junctions
    }

    fn remove_overlapped_junction_nodes<'a>(
        &self,
        doc: &mir::Document,
        junctions: impl IntoIterator<Item = &'a Point>,
    ) -> Vec<Point> {
        let mut edge_junctions: Vec<Point> = vec![];

        // Remove junction nodes that overlap any (fatter) shapes. However, nodes on the edge of the
        // shape must remain.
        let shape_rects = doc
            .body()
            .children()
            .filter_map(|x| doc.get_node(&x))
            .filter_map(|x| x.rect())
            .map(|r| {
                r.inset_by(
                    // Nodes on the edge of fatter shapes must remain. So minus 1.0 from margin.
                    -(Self::SHAPE_JUNCTION_MARGIN - 1.0),
                    -(Self::SHAPE_JUNCTION_MARGIN - 1.0),
                )
            })
            .collect::<Vec<_>>();

        'OUTER: for j in junctions {
            for r in &shape_rects {
                if r.contains_point(j) {
                    continue 'OUTER;
                }
            }

            edge_junctions.push(*j);
        }

        edge_junctions
    }

    /// Connects the nearest nodes in the vertical and horizontal directions.
    fn connect_nearest_neighbor_edge_junctions(&mut self, _: &mir::Document) {
        let mut edges: Vec<(RouteNodeId, RouteNodeId)> = Vec::new();

        for n in self.edge_route_graph.nodes() {
            let mut left: Option<&RouteNode> = None;
            let mut right: Option<&RouteNode> = None;
            let mut up: Option<&RouteNode> = None;
            let mut down: Option<&RouteNode> = None;

            for m in self.edge_route_graph.nodes() {
                let p = n.location();
                let q = m.location();

                if q.x == p.x && q.y < p.y {
                    // vertically upward
                    //
                    // ```svgbob
                    //   o
                    //   ^
                    //   |
                    //   *
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Direction::Up) && m.is_connectable(Direction::Down) {
                        // Is nearest neighbor?
                        if up.is_none() || up.unwrap().location().y < q.y {
                            up.replace(m);
                        }
                    }
                } else if q.x == p.x && q.y > p.y {
                    // vertically downward
                    //
                    // ```svgbob
                    //   *
                    //   |
                    //   v
                    //   o
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Direction::Down) && m.is_connectable(Direction::Up) {
                        // Is nearest neighbor?
                        if down.is_none() || down.unwrap().location().y > q.y {
                            down.replace(m);
                        }
                    }
                } else if q.y == p.y && q.x < p.x {
                    // horizontally leftward
                    //
                    // ```svgbob
                    // o <-- *
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Direction::Left) && m.is_connectable(Direction::Right) {
                        // Is nearest neighbor?
                        if left.is_none() || left.unwrap().location().x < q.x {
                            left.replace(m);
                        }
                    }
                } else if q.y == p.y && q.x > p.x {
                    // horizontally rightward
                    //
                    // ```svgbob
                    // * --> o
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Direction::Right) && m.is_connectable(Direction::Left) {
                        // Is nearest neighbor?
                        if right.is_none() || right.unwrap().location().x > q.x {
                            right.replace(m);
                        }
                    }
                }
            }

            for dest in [left, right, up, down] {
                let Some(dest) = dest else { continue } ;
                edges.push((n.id(), dest.id()));
            }
        }

        for (a, b) in edges {
            self.edge_route_graph.add_edge(a, b);
        }
    }
}
