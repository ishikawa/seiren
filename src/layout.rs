//! Layout engine
use derive_more::Display;

use crate::{
    geometry::{Path, Point, Size},
    mir::{self, NodeKind},
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

    pub fn add_node(&mut self, at: Point) -> RouteNodeId {
        let node_id = RouteNodeId(self.nodes.len());
        let node = RouteNode::new(node_id, at);

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
        for (from, to) in [(a, b), (b, a)] {
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
}

impl RouteNode {
    pub fn new(id: RouteNodeId, location: Point) -> Self {
        Self { id, location }
    }

    pub fn id(&self) -> RouteNodeId {
        self.id
    }

    pub fn location(&self) -> &Point {
        &self.location
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
            for pt in [
                Point::new(record_rect.mid_x(), record_rect.min_y()), // top
                Point::new(record_rect.max_x(), record_rect.mid_y()), // right
                Point::new(record_rect.mid_x(), record_rect.max_y()), // bottom
                Point::new(record_rect.min_x(), record_rect.mid_y()), // left
            ] {
                record_node.append_connection_point(pt);
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
                    for pt in [
                        Point::new(field_rect.mid_x(), field_rect.min_y()), // top
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.mid_x(), field_rect.max_y()), // bottom
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else if field_index == 0 {
                    for pt in [
                        Point::new(field_rect.mid_x(), field_rect.min_y()), // top
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else if field_index == (field_id_vec.len() - 1) {
                    for pt in [
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.mid_x(), field_rect.max_y()), // bottom
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else {
                    for pt in [
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
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
            let mut connection_points: (
                Point, // start point
                Point, // end point
                f32,   // distance
            ) = (Point::default(), Point::default(), f32::MAX);

            for pt1 in start_node.connection_points() {
                for pt2 in end_node.connection_points() {
                    let d = pt1.distance(pt2);
                    if d < connection_points.2 {
                        connection_points = (pt1.clone(), pt2.clone(), d);
                    }
                }
            }

            // Build a path.
            let start_cx = connection_points.0.x;
            let end_cx = connection_points.1.x;
            let start_cy = connection_points.0.y;
            let end_cy = connection_points.1.y;

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
            let mut path = Path::new(connection_points.0);

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

            for c in start_node.connection_points() {
                let junctions = self.edge_junction_nodes_from_connection_point(
                    &doc,
                    start_node,
                    c,
                    &shape_junctions,
                );

                crossing_junctions.extend(junctions);
            }
            for c in end_node.connection_points() {
                let junctions = self.edge_junction_nodes_from_connection_point(
                    &doc,
                    end_node,
                    c,
                    &shape_junctions,
                );

                crossing_junctions.extend(junctions);
            }
        }

        let mut edge_junctions = self.remove_overlapped_junction_nodes(
            &doc,
            shape_junctions.iter().chain(crossing_junctions.iter()),
        );

        // Add start/end connection points.
        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };

            for c in start_node.connection_points() {
                edge_junctions.push(*c);
            }
            for c in end_node.connection_points() {
                edge_junctions.push(*c);
            }
        }

        for j in edge_junctions {
            self.edge_route_graph.add_node(j);
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
        node: &mir::Node,
        connection_point: &Point,
        other_junctions: &[Point],
    ) -> Vec<Point> {
        let margin = Self::SHAPE_JUNCTION_MARGIN;
        let mut junctions = vec![];
        let Some(rect) = node.rect() else { return junctions };
        let center = rect.center();

        let shape_rects = doc
            .body()
            .children()
            .filter_map(|x| doc.get_node(&x))
            .filter_map(|x| x.rect())
            .map(|r| r.inset_by(-margin, -margin))
            .collect::<Vec<_>>();

        if connection_point.x < center.x {
            // Horizontally leftward
            let mut min_x = 0.0f32;
            let line_end = Point::new(f32::MIN, connection_point.y);

            for r in shape_rects {
                if r.max_x() < connection_point.x && r.intersects_line(&connection_point, &line_end)
                {
                    min_x = min_x.max(r.max_x());
                }
            }

            for j in other_junctions {
                if j.x <= connection_point.x && j.x >= min_x {
                    junctions.push(Point::new(j.x, connection_point.y));
                }
            }
        } else if connection_point.x > center.x {
            // Horizontally rightward
            let mut max_x = f32::MAX;
            let line_end = Point::new(f32::MAX, connection_point.y);

            for r in shape_rects {
                if r.min_x() > connection_point.x && r.intersects_line(&connection_point, &line_end)
                {
                    max_x = max_x.min(r.min_x());
                }
            }

            for j in other_junctions {
                if j.x >= connection_point.x && j.x <= max_x {
                    junctions.push(Point::new(j.x, connection_point.y));
                }
            }
        } else if connection_point.y < center.y {
            // Vertically downward
            let mut max_y = f32::MAX;
            let line_end = Point::new(connection_point.x, f32::MAX);

            for r in shape_rects {
                if r.min_y() > connection_point.y && r.intersects_line(&connection_point, &line_end)
                {
                    max_y = max_y.min(r.min_y());
                }
            }

            for j in other_junctions {
                if j.y <= connection_point.y && j.y <= max_y {
                    junctions.push(Point::new(connection_point.x, j.y));
                }
            }
        } else if connection_point.y > center.y {
            // Vertically upward
            let mut min_y = 0.0f32;
            let line_end = Point::new(connection_point.x, f32::MIN);

            for r in shape_rects {
                if r.max_y() < connection_point.y && r.intersects_line(&connection_point, &line_end)
                {
                    min_y = min_y.max(r.max_y());
                }
            }

            for j in other_junctions {
                if j.y >= connection_point.y && j.y >= min_y {
                    junctions.push(Point::new(connection_point.x, j.y));
                }
            }
        }

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
                    // Nodes on the edge of shapes must remain. So minus 1.0 from margin.
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
    fn connect_nearest_neighbor_edge_junctions(&mut self, doc: &mir::Document) {
        let mut edges: Vec<(RouteNodeId, RouteNodeId)> = Vec::new();

        let shape_rects = doc
            .body()
            .children()
            .filter_map(|x| doc.get_node(&x))
            .filter_map(|x| x.rect())
            .collect::<Vec<_>>();

        for n in self.edge_route_graph.nodes() {
            let mut left: Option<&RouteNode> = None;
            let mut right: Option<&RouteNode> = None;
            let mut up: Option<&RouteNode> = None;
            let mut down: Option<&RouteNode> = None;

            for m in self.edge_route_graph.nodes() {
                let p = n.location();
                let q = m.location();

                // the vertical direction
                if q.x == p.x {
                    if q.y < p.y && up.filter(|u| u.location().y > q.y).is_none() {
                        up.replace(m);
                    } else if q.y > p.y && down.filter(|d| d.location().y < q.y).is_none() {
                        down.replace(m);
                    }
                }
                // the horizontal direction
                else if q.y == p.y {
                    if q.x < p.x && left.filter(|l| l.location().x > q.x).is_none() {
                        left.replace(m);
                    } else if q.x > p.x && right.filter(|r| r.location().x < q.x).is_none() {
                        right.replace(m);
                    }
                }
            }

            'OUTER: for dest in [left, right, up, down] {
                let Some(dest) = dest else { continue } ;

                for r in shape_rects.iter() {
                    if let Some((p, q)) = r.intersected_line(n.location(), dest.location()) {
                        // Is a connection point?
                        if p == q && (p == *n.location() || p == *dest.location()) {
                            break;
                        } else {
                            continue 'OUTER;
                        }
                    }
                }

                edges.push((n.id(), dest.id()));
            }
        }

        for (a, b) in edges {
            self.edge_route_graph.add_edge(a, b);
        }
    }
}
