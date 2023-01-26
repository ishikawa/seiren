//! Layout engine
//!
//! Algorithm
//! ---------
//!
//! To illustrate the layout algorithm implemented in this module, consider the following example.
//!
//! ```plaintext
//! posts.created_by o--> users.id
//! comments.post_id o--> posts.id
//! comments.created_by o--> users.id
//! post_tags.post_id o--> posts.id
//! post_tags.tag_id o--> tags.id
//! ```
//!
//! In this example, there are records `users`, `posts`, `comments`, `post_tags`, and `tags`, which
//! are related to each other. Suppose these records are arranged as follows:
//!
//! ```svgbob
//! +---------+    +-------------+
//! | (users) |    | (posts)     |
//! | id      |    | id          |
//! | :       |    | created_by  |
//! +---------+    |             |
//!                +-------------+
//!
//!                +-------------+
//!                | (comments)  |
//!                | post_id     |
//!                | created_by  |
//!                |             |
//!                +-------------+
//!
//! +---------+    +-------------+
//! | (tags)  |    | (post_tags) |
//! | id      |    | post_id     |
//! | :       |    | tag_id      |
//! +---------+    |             |
//!                +-------------+
//! ```
//!
//! Connections between related records (fields) are drawn as follows:
//!
//! ```svgbob
//! +---------+    +-------------+
//! | (users) |    | (posts)     |
//! | id      o<-. | id          o<-.
//! | :       |  |-o created_by  |  |
//! +---------+  | |             |  |
//!              | +-------------+  |
//!              |                  |
//!              | +-------------+  |
//!              | | (comments)  |  |
//!              | | post_id     o--|
//!              `-o created_by  |  |
//!                |             |  |
//!                +-------------+  |
//!                                 |
//! +---------+    +-------------+  |
//! | (tags)  |    | (post_tags) |  |
//! | id      o<-. | post_id     o--'
//! | :       |  `-o tag_id      |
//! +---------+    |             |
//!                +-------------+
//! ```
//!
//! The rules for drawing connections are shown below:
//!
//! - There are connection ports to the left, right, or bottom of the field that can be connected.
//! - Connections can only go horizontally or vertically and can turn at **bends** around the record.
//!   - Therefore, the angle of the bend must always be a right angle,
//! - Connections should choose the shortest path.
//! - Connections incident to different fields SHOULD NOT intersect or take the same path.
//!
//! To calculate the path of a connection, first consider the field and the connection point as
//! vertices aligned on a line of the grid.
//!
//! ```svgbob
//! (0, 0)             (0, 4)
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*...o...*...o
//!     :   :   :   :   :
//!     o.......o...*...o
//!     :   :   :   :   :
//!     ........o...o...o
//!     :   :   :   :   :
//!     ........o...*...o
//!     :   :   :   :   :
//!     ........o...*...o
//!     :   :   :   :   :
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*...o...*...o
//!     :   :   :   :   :
//!     o.......o...*...o
//!     :   :   :   :   :
//!     ........o...o...o
//! (9, 0)             (9, 4)
//! ```
//!
//! Calculate the route according to the above rules.
//!
//! ```svgbob
//! (0, 0)             (0, 4)
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*<--o...*<--o
//!     :   :   |   :   |
//!     o.......o---*...o
//!     :   :   |   :   |
//!     ........o...o...o
//!     :   :   |   :   |
//!     ........o...*---o
//!     :   :   |   :   |
//!     ........o---*...o
//!     :   :   :   :   |
//!     o...o...o...o...o
//!     :   :   :   :   |
//!     o...*<--o...*---o
//!     :   :   |   :   :
//!     o.......o---*...o
//!     :   :   :   :   :
//!     ........o...o...o
//! (9, 0)             (9, 4)
//! ```
use crate::{
    geometry::{Orientation, Point, Rect, Size},
    mir::{self, ShapeKind, TerminalPort, TerminalPortId},
};
use derive_more::Add;
use petgraph::algo;
use petgraph::{
    prelude::{EdgeIndex, NodeIndex, UnGraph},
    visit::EdgeRef,
};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    hash::Hash,
};

pub trait LayoutEngine {
    /// Place all nodes on 2D coordination.
    ///
    /// The engine must assign `origin` and `size` of all nodes.
    /// Returns computed view box.
    fn place_nodes(&mut self, doc: &mut mir::Document) -> Option<Rect>;

    /// Place all terminal ports for every node.
    ///
    /// The engine must add all possible terminal ports to `terminal_ports` of nodes.
    fn place_terminal_ports(&mut self, doc: &mut mir::Document);

    /// Draw path between both ends (terminal ports) of each edge.
    ///
    /// The engine must build a `path` of edges.
    fn draw_edge_path(&mut self, doc: &mut mir::Document);
}

type _RouteGraph = UnGraph<RouteNodeData, RouteEdgeData>;

/// Represents routes in a place by graph. Every junction of two edges will be a node of the graph.
/// Neighboring junctions are connected by edges. Each nodes neighbors four other nodes and each
/// edge is NOT directed so shared by two junctions.
#[derive(Debug, Clone)]
pub struct RouteGraph {
    graph: _RouteGraph,
    terminal_ports: HashMap<TerminalPortId, RouteNodeId>,
}

impl RouteGraph {
    pub fn new() -> Self {
        let graph = _RouteGraph::new_undirected();

        Self {
            graph,
            terminal_ports: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &RouteNodeData> {
        self.graph.node_weights()
    }

    pub fn node_ids(&self) -> impl ExactSizeIterator<Item = RouteNodeId> {
        self.graph.node_indices().map(|i| RouteNodeId(i))
    }

    pub fn get_node(&self, id: RouteNodeId) -> Option<&RouteNodeData> {
        self.graph.node_weight(id.0)
    }

    pub fn get_terminal_port(&self, id: TerminalPortId) -> Option<RouteNodeId> {
        self.terminal_ports.get(&id).copied()
    }

    pub fn add_node(&mut self, location: Point) -> RouteNodeId {
        self._add_node(location, None)
    }

    pub fn add_terminal_port(&mut self, terminal_port: &TerminalPort) -> RouteNodeId {
        let node_index = self._add_node(
            terminal_port.location().clone(),
            Some(terminal_port.orientation().clone()),
        );

        self.terminal_ports.insert(terminal_port.id(), node_index);
        node_index
    }

    fn _add_node(&mut self, location: Point, orientation: Option<Orientation>) -> RouteNodeId {
        let node_index = if let Some((node_index, node)) = self
            .graph
            .node_indices()
            .flat_map(|i| self.graph.node_weight(i).map(|w| (i, w)))
            .find(|(_, w)| *w.location() == location)
        {
            if node.orientation() != orientation {
                panic!(
                    "[BUG] Placing node at {}, but the old node orientation is different. {:?} != {}",
                    location,
                    node
                        .orientation()
                        .map_or("(none)".into(), |x| x.to_string()),
                    orientation.map_or("(none)".into(), |x| x.to_string())
                );
            }
            node_index
        } else {
            let node = RouteNodeData::new(location, orientation);
            self.graph.add_node(node)
        };

        RouteNodeId(node_index)
    }

    pub fn edge_endpoints(&self, edge_id: RouteEdgeId) -> Option<(RouteNodeId, RouteNodeId)> {
        self.graph
            .edge_endpoints(edge_id.0)
            .map(|(x, y)| (RouteNodeId(x), RouteNodeId(y)))
    }

    pub fn edges(&self) -> impl Iterator<Item = &RouteEdgeData> {
        self.graph.edge_weights()
    }

    pub fn edges_mut(&mut self) -> impl Iterator<Item = &mut RouteEdgeData> {
        self.graph.edge_weights_mut()
    }

    pub fn add_edge(&mut self, a: RouteNodeId, b: RouteNodeId) {
        for (from, to) in [(a, b)] {
            if !self.graph.edges(from.0).any(|e| e.target() == to.0) {
                self.graph
                    .add_edge(from.0, to.0, RouteEdgeData::new(from, to));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RouteNodeId(NodeIndex);

impl fmt::Display for RouteNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RouteEdgeId(EdgeIndex);

impl fmt::Display for RouteEdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.index())
    }
}

#[derive(Debug, Clone)]
pub struct RouteNodeData {
    location: Point,

    /// If the node is a terminal port, copy its `orientation` to
    /// detect edge connectivity.
    orientation: Option<Orientation>,
}

impl RouteNodeData {
    pub fn new(location: Point, orientation: Option<Orientation>) -> Self {
        Self {
            location,
            orientation,
        }
    }

    pub fn location(&self) -> &Point {
        &self.location
    }

    pub fn orientation(&self) -> Option<Orientation> {
        self.orientation
    }

    /// Returns `true` if `node.orientation` is `None` or equal to a specified `orientation`.
    pub fn is_connectable(&self, orientation: Orientation) -> bool {
        match self.orientation {
            None => true,
            Some(d) => d == orientation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteEdgeData {
    source_id: RouteNodeId,
    target_id: RouteNodeId,
}

impl RouteEdgeData {
    pub fn new(source_id: RouteNodeId, target_id: RouteNodeId) -> Self {
        Self {
            source_id,
            target_id,
        }
    }

    pub fn source_id(&self) -> RouteNodeId {
        self.source_id
    }

    pub fn target_id(&self) -> RouteNodeId {
        self.target_id
    }
}

// Used for computing shortest path
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Add)]
struct RouteCost(u32);

impl RouteCost {
    pub const MAX: Self = Self(u32::MAX);
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
    const ORIGIN: Point = Point::new(50.0, 50.0);
    const LINE_HEIGHT: f32 = 35.0;
    const RECORD_WIDTH: f32 = 300.0;
    const RECORD_SPACE: f32 = 80.0;

    // The number of columns in fixed grid.
    const GRID_N_COLUMNS: usize = 2;

    // for debug
    pub fn edge_route_graph(&self) -> &RouteGraph {
        &self.edge_route_graph
    }
}

impl LayoutEngine for SimpleLayoutEngine {
    fn place_nodes(&mut self, doc: &mut mir::Document) -> Option<Rect> {
        // Grid
        let n_columns = Self::GRID_N_COLUMNS;
        let blank_cell_indices: [usize; 1] = [3];

        // Iterate records
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        let mut base_y = Self::ORIGIN.y;
        let mut max_height = f32::MIN;
        let mut grid_cell_index = 0;

        for (record_index, child_id) in child_id_vec.iter().copied().enumerate() {
            if blank_cell_indices.contains(&grid_cell_index) {
                grid_cell_index += 1;
            }

            // Calculate grid cell rectangle
            if grid_cell_index > 0 && (grid_cell_index % n_columns == 0) {
                // Move to next row.
                base_y += max_height + Self::RECORD_SPACE;
                max_height = f32::MIN;
            }

            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let ShapeKind::Record(_) = record_node.kind() else  { continue };

            let n_fields = record_node.children().len() as f32;
            let x = Self::ORIGIN.x
                + (Self::RECORD_WIDTH + Self::RECORD_SPACE) * (grid_cell_index % n_columns) as f32;

            let record_height = Self::LINE_HEIGHT * n_fields;
            max_height = record_height.max(max_height);

            record_node.origin = Some(Point::new(x, base_y));
            record_node.size = Some(Size::new(Self::RECORD_WIDTH.into(), record_height.into()));

            // children
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_index) in field_id_vec.iter().copied().enumerate() {
                let y = base_y + Self::LINE_HEIGHT * field_index as f32;
                let Some(field_node) = doc.get_node_mut(field_node_index) else { continue };
                let ShapeKind::Field(_) = field_node.kind() else  { continue };

                field_node.origin = Some(Point::new(x, y));
                field_node.size = Some(Size::new(Self::RECORD_WIDTH, Self::LINE_HEIGHT));
            }

            grid_cell_index += 1;
        }

        // Compute view box
        let min_width = (Self::ORIGIN.x * 2.0) // x-margin
            + ((n_columns as f32) * Self::RECORD_WIDTH) // shape width
            + (((n_columns - 1) as f32) * Self::RECORD_SPACE); // spaces
        let min_height = base_y + max_height + Self::ORIGIN.y;

        Some(Rect::new(Point::zero(), Size::new(min_width, min_height)))
    }

    fn place_terminal_ports(&mut self, doc: &mut mir::Document) {
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (_, child_id) in child_id_vec.iter().copied().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let Some(record_rect) = record_node.rect() else { continue };

            // In the case of a rectangle, terminal ports are placed in
            // the center of each of the four edges.
            for (x, y, d) in [
                (record_rect.mid_x(), record_rect.min_y(), Orientation::Up),
                (record_rect.max_x(), record_rect.mid_y(), Orientation::Right),
                (record_rect.mid_x(), record_rect.max_y(), Orientation::Down),
                (record_rect.min_x(), record_rect.mid_y(), Orientation::Left),
            ] {
                record_node.add_terminal_port(child_id, Point::new(x, y), d);
            }

            // For each field in a rectangle, terminal ports are placed
            // the center of:
            // - each of the four edges - if the number of fields is `1`.
            // - top, left and right - for the top field
            // - bottom, left and right - for the bottom field
            // - left and right - for the rest
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_index) in field_id_vec.iter().copied().enumerate() {
                let Some(field_node) = doc.get_node_mut(field_node_index) else { continue };
                let Some(field_rect) = field_node.rect() else { continue };

                if field_id_vec.len() == 1 {
                    for (x, y, d) in [
                        (field_rect.mid_x(), field_rect.min_y(), Orientation::Up),
                        (field_rect.max_x(), field_rect.mid_y(), Orientation::Right),
                        (field_rect.mid_x(), field_rect.max_y(), Orientation::Down),
                        (field_rect.min_x(), field_rect.mid_y(), Orientation::Left),
                    ] {
                        field_node.add_terminal_port(field_node_index, Point::new(x, y), d);
                    }
                } else if field_index == 0 {
                    for (x, y, d) in [
                        (field_rect.mid_x(), field_rect.min_y(), Orientation::Up),
                        (field_rect.max_x(), field_rect.mid_y(), Orientation::Right),
                        (field_rect.min_x(), field_rect.mid_y(), Orientation::Left),
                    ] {
                        field_node.add_terminal_port(field_node_index, Point::new(x, y), d);
                    }
                } else if field_index == (field_id_vec.len() - 1) {
                    for (x, y, d) in [
                        (field_rect.max_x(), field_rect.mid_y(), Orientation::Right),
                        (field_rect.mid_x(), field_rect.max_y(), Orientation::Down),
                        (field_rect.min_x(), field_rect.mid_y(), Orientation::Left),
                    ] {
                        field_node.add_terminal_port(field_node_index, Point::new(x, y), d);
                    }
                } else {
                    for (x, y, d) in [
                        (field_rect.max_x(), field_rect.mid_y(), Orientation::Right),
                        (field_rect.min_x(), field_rect.mid_y(), Orientation::Left),
                    ] {
                        field_node.add_terminal_port(field_node_index, Point::new(x, y), d);
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
        // c. From the start/end terminal port, draw a straight line horizontally or vertically
        //    until it collides with another shape node, and place a new junction node at the point
        //    where it intersects the junction node (b) in a crosswise direction.
        //
        // d. Remove junction nodes that overlap any (fatter) shapes. However, nodes on the edge of
        //    the shape must remain.
        //
        // e. Add start/end terminal ports.

        // Place junction nodes at the four corner points around each shape node.
        let shape_junctions = self.edge_junction_nodes_around_shapes(&doc);

        // From the start/end junction point, draw a straight line horizontally or vertically until
        // it collides with another shape node, and place a new junction node at the point where it
        // intersects the junction node in a crosswise direction.
        let mut crossing_junctions: Vec<Point> = vec![];

        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(edge.source_id()) else { continue };
            let Some(end_node) = doc.get_node(edge.target_id()) else { continue };

            for pt in start_node.terminal_ports() {
                let junctions = self.edge_junction_nodes_from_terminal_port(
                    &doc,
                    start_node,
                    pt,
                    &shape_junctions,
                );

                crossing_junctions.extend(junctions);
            }
            for pt in end_node.terminal_ports() {
                let junctions = self.edge_junction_nodes_from_terminal_port(
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

        // Add start/end terminal ports.
        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(edge.source_id()) else { continue };
            let Some(end_node) = doc.get_node(edge.target_id()) else { continue };

            for pt in start_node.terminal_ports() {
                self.edge_route_graph.add_terminal_port(pt);
            }
            for pt in end_node.terminal_ports() {
                self.edge_route_graph.add_terminal_port(pt);
            }
        }

        self.connect_nearest_neighbor_edge_junctions(doc);

        // Finding shortest edge paths
        let edge_ids = doc.edge_ids();
        let mut paths: VecDeque<Vec<Point>> = VecDeque::with_capacity(edge_ids.len());

        for edge_id in edge_ids {
            if let Some(path) = self.find_shortest_edges_path(doc, edge_id) {
                paths.push_back(path);
            }
        }

        for edge in doc.edges_mut() {
            edge.set_path_points(Some(paths.pop_front().unwrap()));
        }
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
            let Some(record_node) = doc.get_node(child_id) else { continue };
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

    // c. From the start/end terminal port, draw a straight line horizontally or vertically
    //    until it collides with another shape node, and place a new junction node at the point
    //    where it intersects the junction node (b) in a crosswise direction.
    fn edge_junction_nodes_from_terminal_port(
        &self,
        doc: &mir::Document,
        _: &mir::NodeData,
        terminal_port: &TerminalPort,
        other_junctions: &[Point],
    ) -> Vec<Point> {
        let margin = Self::SHAPE_JUNCTION_MARGIN;
        let mut junctions = vec![];

        let shape_rects = doc
            .body()
            .children()
            .filter_map(|child_id| doc.get_node(child_id))
            .filter_map(|node| node.rect())
            .map(|r| r.inset_by(-margin, -margin))
            .collect::<Vec<_>>();

        let conn_pt = terminal_port.location();

        match terminal_port.orientation() {
            Orientation::Left => {
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
            Orientation::Right => {
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
            Orientation::Up => {
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
            Orientation::Down => {
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
            .filter_map(|node_index| doc.get_node(node_index))
            .filter_map(|node| node.rect())
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
    fn connect_nearest_neighbor_edge_junctions(&mut self, doc: &mir::Document) {
        let mut edges: Vec<(RouteNodeId, RouteNodeId)> = Vec::new();

        // Collision detection
        let shape_rects = doc
            .body()
            .children()
            .filter_map(|node_index| doc.get_node(node_index).map(|node| (node_index, node)))
            .filter_map(|(node_index, node)| {
                node.rect().map(|r| {
                    (
                        node_index,
                        // Nodes on the edge of shapes must remain. So minus 1.0.
                        r.inset_by(1.0, 1.0),
                    )
                })
            })
            .collect::<Vec<_>>();

        for a in self.edge_route_graph.node_ids() {
            let mut left: Option<(RouteNodeId, &RouteNodeData)> = None;
            let mut right: Option<(RouteNodeId, &RouteNodeData)> = None;
            let mut up: Option<(RouteNodeId, &RouteNodeData)> = None;
            let mut down: Option<(RouteNodeId, &RouteNodeData)> = None;

            for b in self.edge_route_graph.node_ids() {
                let n = self.edge_route_graph.get_node(a).unwrap();
                let m = self.edge_route_graph.get_node(b).unwrap();
                let p = n.location();
                let q = m.location();
                let no_collision = || !shape_rects.iter().any(|(_, r)| r.intersects_line(p, q));

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
                    if n.is_connectable(Orientation::Up) && m.is_connectable(Orientation::Down) {
                        // Is nearest neighbor?
                        if up.is_none() || up.unwrap().1.location().y < q.y && no_collision() {
                            up.replace((b, m));
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
                    if n.is_connectable(Orientation::Down) && m.is_connectable(Orientation::Up) {
                        // Is nearest neighbor?
                        if down.is_none() || down.unwrap().1.location().y > q.y && no_collision() {
                            down.replace((b, m));
                        }
                    }
                } else if q.y == p.y && q.x < p.x {
                    // horizontally leftward
                    //
                    // ```svgbob
                    // o <-- *
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Orientation::Left) && m.is_connectable(Orientation::Right) {
                        // Is nearest neighbor?
                        if left.is_none() || left.unwrap().1.location().x < q.x && no_collision() {
                            left.replace((b, m));
                        }
                    }
                } else if q.y == p.y && q.x > p.x {
                    // horizontally rightward
                    //
                    // ```svgbob
                    // * --> o
                    // ```

                    // Is connectable direction?
                    if n.is_connectable(Orientation::Right) && m.is_connectable(Orientation::Left) {
                        // Is nearest neighbor?
                        if right.is_none() || right.unwrap().1.location().x > q.x && no_collision()
                        {
                            right.replace((b, m));
                        }
                    }
                }
            }

            for dest in [left, right, up, down] {
                let Some(dest) = dest else { continue } ;
                edges.push((a, dest.0));
            }
        }

        for (a, b) in edges {
            self.edge_route_graph.add_edge(a, b);
        }
    }

    /// Find the shortest path between both ends of a specified `edge`.
    ///
    /// Returns locations of each nodes (start, intermediate and end) on the shortest path.
    fn find_shortest_edges_path(
        &self,
        doc: &mir::Document,
        edge_id: mir::EdgeId,
    ) -> Option<Vec<Point>> {
        let Some((source_id, target_id)) = doc.edge_endpoints(edge_id) else { return None };

        // Run Dijkstra's algorithm for each terminal ports of the start/end node. It's
        // inefficient but more generic solution than using heuristics about the distance between
        // nodes.
        let Some(start_node) = doc.get_node(source_id) else { return None };
        let Some(end_node) = doc.get_node(target_id) else { return None };

        let mut cost = RouteCost::MAX;
        let mut path: Option<Vec<RouteNodeId>> = None;

        for src in start_node.terminal_ports() {
            for dst in end_node.terminal_ports() {
                let Some(src_node) = self.edge_route_graph.get_terminal_port(src.id()) else { continue };
                let Some(dst_node) = self.edge_route_graph.get_terminal_port(dst.id()) else { continue };

                let (c, p) = self.compute_shortest_path(src_node, dst_node);
                if c < cost {
                    path.replace(p);
                    cost = c;
                }
            }
        }

        path.map(|path| {
            path.iter()
                .copied()
                .map(|id| self.edge_route_graph().get_node(id).unwrap().location())
                .copied()
                .collect()
        })
    }

    /// Run Dijkstra's algorithm to compute the shortest path between `start_node` and `end_node`.
    fn compute_shortest_path(
        &self,
        start_node: RouteNodeId,
        end_node: RouteNodeId,
    ) -> (RouteCost, Vec<RouteNodeId>) {
        let graph = &self.edge_route_graph().graph;

        let (cost, path) = algo::astar(
            graph,
            start_node.0,
            |finish| finish == end_node.0,
            |edge| {
                let node = graph.node_weight(edge.source()).unwrap();
                let to_node = graph.node_weight(edge.target()).unwrap();

                let distance = node.location().distance(to_node.location());
                RouteCost(distance as u32)
            },
            |_| RouteCost(0),
        )
        .unwrap_or_else(|| {
            panic!(
                "can't compute shortest path: {} -> {}",
                start_node, end_node
            )
        });

        (cost, path.iter().map(|i| RouteNodeId(*i)).collect())
    }
}
