//! Graph embedded on a graph.
//!
//! ## Definition
//!
//! - **Grid point** - a point with integer coordinates.
//! - **(m, n) grid** - the set of grid points `(x, y)` with:
//!   - `0 <= x <= m`
//!   - `0 <= y <= n`
//! - Places each vertex of an undirected graph `G = (V, E)` at each grid point.
//! - The number of edges (degree) `d` connecting each vertex is `0 <= d <= 4`.
//! - Only adjacent vertices on the grid can be connected.
//! - **Path** - A sequence of connected vertices in a graph is called a *path*.
//! - **Neighbors** - Nodes connecting by an edge.
//! - **Adjacent nodes** - Nodes adjacent to a node.
//!
//! ```svgbob
//! (0, 0)             (4, 0)
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*...o...*...o
//!     :       :   :   :
//!     o...o...o...*...o
//!             :   :   :
//!             o...o...o
//!             :   :   :
//!             o...*...o
//!             :   :   :
//!             o...*...o
//!             :   :   :
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*...o...*...o
//!     :       :   :   :
//!     o...o...o...*...o
//! (0, 8)      :   :   :
//!             o...o...o
//!                    (4, 9)
//! ```
//!
//! Path:
//!
//! ```svgbob
//! (0, 0)             (0, 4)
//!     o...o...o...o...o
//!     :   :   :   :   :
//!     o...*<--o...*<--o
//!     :       |   :   |
//!     o...o...o---*...o
//!             |   :   |
//!             o...o...o
//!             |   :   |
//!             o...*---o
//!             |   :   |
//!             o---*...o
//!             :   :   |
//!     o...o...o...o...o
//!     :   :   :   :   |
//!     o...*<--o...*---o
//!     :       |   :   :
//!     o...o...o---*...o
//!             :   :   :
//!             o...o...o
//! (0, 9)             (4, 9)
//! ```
use derive_more::Display;
use fixedbitset::FixedBitSet;
use petgraph::graph::{EdgeIndex, IndexType, NodeIndex};
use petgraph::{visit, Directed, EdgeType, Undirected};
use smallvec::SmallVec;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// A point with integer coordinates.
///
/// **(m, n) grid** is the set of grid points `(x, y)` with `0 <= x <= m, 0 <= y <= n`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(fmt = "({}, {})", x, y)]
pub struct GridPoint {
    pub x: u16,
    pub y: u16,
}

impl GridPoint {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> usize {
        self.x as usize
    }
    pub fn y(&self) -> usize {
        self.y as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
#[display(fmt = "({}, {})", columns, rows)]
/// The number of columns and rows of a `(m, n)` grid.
struct GridShape {
    pub columns: u16, // m
    pub rows: u16,    // n
}

impl GridShape {
    pub fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }

    pub fn columns(&self) -> usize {
        self.columns as usize
    }

    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    /// Return the upper bound of the node indices in a grid.
    pub fn node_bound(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// Return the upper bound of the edge indices in a grid.
    pub fn edge_bound(&self) -> usize {
        if self.columns == 0 || self.rows == 0 {
            0
        } else {
            (self.columns * (self.rows * 2 - 1)) as usize
        }
    }

    /// Computes the grid point of a node at `index`.
    pub fn node_point(&self, index: usize) -> Option<GridPoint> {
        if index >= self.node_bound() {
            return None;
        }

        let x = index % self.columns as usize;
        let y = index / self.columns as usize;

        Some(GridPoint::new(x as u16, y as u16))
    }

    pub fn to_node_index(&self, point: GridPoint) -> Option<usize> {
        let i = point.x() + point.y() * self.columns();

        if i >= self.node_bound() {
            None
        } else {
            Some(i)
        }
    }

    pub fn adjacent_node_indices(&self, index: usize) -> SmallVec<[usize; 4]> {
        let mut v = SmallVec::with_capacity(4);

        let Some(p) = self.node_point(index) else { return v };

        // Left
        if p.x > 0 {
            let Some(q) = self.to_node_index(GridPoint::new(p.x - 1, p.y)) else { return v };
            v.push(q);
        }
        // Right
        if p.x < self.columns {
            let Some(q) = self.to_node_index(GridPoint::new(p.x + 1, p.y)) else { return v };
            v.push(q);
        }
        // Up
        if p.y > 0 {
            let Some(q) = self.to_node_index(GridPoint::new(p.x, p.y - 1)) else { return v };
            v.push(q);
        }
        // Down
        if p.y < self.rows {
            let Some(q) = self.to_node_index(GridPoint::new(p.x, p.y + 1)) else { return v };
            v.push(q);
        }

        v
    }

    /// Computes edge index between node `a` and `b`.
    ///
    /// Returns `None` if:
    /// - `a == b`.
    /// - `a` or `b` is overflow.
    /// - `a` and `b` aren't adjacent nodes.
    ///
    pub fn edge_index_between(&self, a: usize, b: usize) -> Option<usize> {
        let edge_bound = self.edge_bound();

        if a == b || a >= edge_bound || b >= edge_bound {
            return None;
        }

        let u = self.node_point(a.min(b))?;
        let v = self.node_point(b.max(a))?;
        assert!(u.x == v.x || u.y == v.y);

        let edge_idx = if u.y == v.y {
            if v.x - u.x != 1 {
                return None;
            }

            u.y() * 2 * self.columns() + u.x()
        } else {
            if v.y - u.y != 1 {
                return None;
            }

            (u.y() * 2 + 1) * self.columns() + u.x()
        };

        Some(edge_idx)
    }
}

#[derive(Debug)]
pub struct Edge<E, Ix = DefaultIx> {
    weight: E,
    source: NodeIndex<Ix>,
    target: NodeIndex<Ix>,
}

impl<E, Ix> Clone for Edge<E, Ix>
where
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        Edge {
            weight: self.weight.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
        }
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.weight = rhs.weight.clone();
        self.source = rhs.source.clone();
        self.target = rhs.target.clone();
    }
}

impl<E, Ix: IndexType> Edge<E, Ix> {
    /// Return the source node index.
    pub fn source(&self) -> NodeIndex<Ix> {
        self.source
    }

    /// Return the target node index.
    pub fn target(&self) -> NodeIndex<Ix> {
        self.target
    }
}

// The default index type. The max size of the type must be large enough that holds `m x n`.
type DefaultIx = u32;

pub struct GridGraph<N, E, Ty = Directed, Ix = DefaultIx> {
    shape: GridShape,
    // nodes = `m x n` vector
    nodes: Vec<Option<N>>,
    // nodes = `m x (n * 2 - 1)` vector
    edges: Vec<Option<Edge<E, Ix>>>,
    ty: PhantomData<Ty>,
    ix: PhantomData<Ix>,
}

/// A `GridGraph` with directed edges.
///
/// For example, an edge from *1* to *2* is distinct from an edge from *2* to
/// *1*.
pub type DiGridGraph<N, E, Ix = DefaultIx> = GridGraph<N, E, Directed, Ix>;

/// A `GridGraph` with undirected edges.
///
/// For example, an edge between *1* and *2* is equivalent to an edge between
/// *2* and *1*.
pub type UnGridGraph<N, E, Ix = DefaultIx> = GridGraph<N, E, Undirected, Ix>;

impl<N, E, Ty, Ix: IndexType> Clone for GridGraph<N, E, Ty, Ix>
where
    N: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        GridGraph {
            shape: self.shape.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            ty: self.ty,
            ix: self.ix,
        }
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.shape = rhs.shape.clone();
        self.nodes.clone_from(&rhs.nodes);
        self.edges.clone_from(&rhs.edges);
        self.ty = rhs.ty;
    }
}

impl<N, E, Ty, Ix> GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    /// Create a new `GridGraph` with `m x n` grid.
    pub fn with_grid(columns: u16, rows: u16) -> Self {
        let shape = GridShape::new(columns, rows);
        let node_bound = shape.node_bound();
        let edge_bound = shape.edge_bound();
        let mut nodes = Vec::with_capacity(node_bound);
        let mut edges = Vec::with_capacity(edge_bound);

        nodes.resize_with(node_bound, || None);
        edges.resize_with(edge_bound, || None);

        Self {
            shape,
            nodes,
            edges,
            ty: PhantomData,
            ix: PhantomData,
        }
    }

    /// Return the upper bound of the node indices in a grid.
    pub fn node_bound(&self) -> usize {
        self.shape.node_bound()
    }

    /// Return the upper bound of the edge indices in a grid.
    pub fn edge_bound(&self) -> usize {
        self.shape.edge_bound()
    }

    /// Return the number of nodes (vertices) in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.iter().filter(|x| x.is_some()).count()
    }

    /// Return the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.iter().filter(|x| x.is_some()).count()
    }

    /// Whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    /// Add a node (also called vertex) with associated data `weight` at the first
    /// available cell in the grid.
    ///
    /// Computes in **O(n)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the Graph is full.
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let idx = self.nodes.iter().position(|x| x.is_none());
        assert!(idx.is_some());

        let node_idx = NodeIndex::new(idx.unwrap());
        self.nodes[node_idx.index()] = Some(weight);
        node_idx
    }

    /// Access the weight for node `a`.
    ///
    /// If node `a` doesn't exist in the graph, return `None`.
    /// Also available with indexing syntax: `&graph[a]`.
    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
        self.nodes.get(a.index()).and_then(|x| x.as_ref())
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// If node `a` doesn't exist in the graph, return `None`.
    /// Also available with indexing syntax: `&mut graph[a]`.
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N> {
        self.nodes.get_mut(a.index()).and_then(|n| n.as_mut())
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// - **Panics** if any of the nodes don't exist.
    /// - **Panics** if there is an edge on the same node pair `(u, v)`.
    /// - **Panics** if the node pair `(u, v)` is not a adjacent nodes.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        let edge_idx = self.shape.edge_index_between(a.index(), b.index()).unwrap();
        assert!(self.edges[edge_idx].is_none());

        let edge = Edge {
            weight,
            source: a,
            target: b,
        };
        self.edges[edge_idx] = Some(edge);

        EdgeIndex::new(edge_idx)
    }

    /// Add or update an edge from `a` to `b`.
    /// If the edge already exists, its weight is updated.
    ///
    /// Return the index of the affected edge.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to `a` (and `b`, if the graph edges are undirected).
    ///
    /// **Panics** if any of the nodes doesn't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        if let Some(ix) = self.find_edge(a, b) {
            if let Some(ed) = self.edge_weight_mut(ix) {
                *ed = weight;
                return ix;
            }
        }
        self.add_edge(a, b, weight)
    }

    /// Access the weight for edge `e`.
    ///
    /// If edge `e` doesn't exist in the graph, return `None`.
    /// Also available with indexing syntax: `&graph[e]`.
    pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E> {
        self.edges
            .get(e.index())
            .and_then(|opt| opt.as_ref().map(|ed| &ed.weight))
    }

    /// Access the weight for edge `e`, mutably.
    ///
    /// If edge `e` doesn't exist in the graph, return `None`.
    /// Also available with indexing syntax: `&mut graph[e]`.
    pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E> {
        self.edges
            .get_mut(e.index())
            .and_then(|opt| opt.as_mut().map(|ed| &mut ed.weight))
    }

    /// Access the source and target nodes for `e`.
    ///
    /// If edge `e` doesn't exist in the graph, return `None`.
    pub fn edge_endpoints(&self, e: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.edges
            .get(e.index())
            .and_then(|opt| opt.as_ref().map(|ed| (ed.source(), ed.target())))
    }

    /// Remove `a` from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return `None`.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> Option<N> {
        let node_idx = a.index();

        self.nodes.get(node_idx)?;

        let weight = std::mem::replace(&mut self.nodes[node_idx], None);

        for i in self.shape.adjacent_node_indices(node_idx) {
            let Some(edge_idx) = self.shape.edge_index_between(node_idx, i) else { continue };
            self.edges[edge_idx] = None;
        }

        weight
    }

    /// Remove an edge and return its edge weight, or `None` if it didn't exist.
    pub fn remove_edge(&mut self, e: EdgeIndex<Ix>) -> Option<E> {
        let edge_idx = e.index();

        self.edges.get(edge_idx)?;

        let e = std::mem::replace(&mut self.edges[edge_idx], None);
        e.map(|e| e.weight)
    }

    /// Lookup if there is an edge from `a` to `b`.
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.find_edge(a, b).is_some()
    }

    /// Lookup an edge from `a` to `b`.
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
        let edge_idx = self.shape.edge_index_between(a.index(), b.index())?;
        let edge = self.edges.get(edge_idx)?;

        if edge.is_none() {
            None
        } else if !self.is_directed() || edge.as_ref().unwrap().source() == a {
            Some(EdgeIndex::new(edge_idx))
        } else {
            None
        }
    }

    /// Return an iterator over the node indices of the graph.
    ///
    /// For example, in a rare case where a graph algorithm were not applicable,
    /// the following code will iterate through all nodes to find a
    /// specific index:
    ///
    /// ```
    /// # use seiren::grid::GridGraph;
    /// # let mut g = GridGraph::<&str, i32>::with_grid(1, 1);
    /// # g.add_node("book");
    /// let index = g.node_indices().find(|i| g[*i] == "book").unwrap();
    /// ```
    pub fn node_indices(&self) -> NodeIdentifiers<Ix> {
        let indices = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .filter_map(|(i, _)| Some(i))
            .collect::<Vec<_>>();

        NodeIdentifiers::new(IndexIterator::new(indices))
    }
}

#[derive(Debug, Clone)]
struct IndexIterator {
    indices: Vec<usize>,
    current: usize,
}

impl IndexIterator {
    fn new(indices: Vec<usize>) -> Self {
        Self {
            indices,
            current: 0,
        }
    }
}

impl Iterator for IndexIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // initialize / advance
        let i = self.current;

        self.current += 1;

        if i < self.indices.len() {
            Some(self.indices[i])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeIdentifiers<Ix> {
    iter: IndexIterator,
    ix: PhantomData<Ix>,
}

impl<'a, Ix: IndexType> NodeIdentifiers<Ix> {
    fn new(iter: IndexIterator) -> Self {
        Self {
            iter,
            ix: PhantomData,
        }
    }
}

impl<Ix: IndexType> Iterator for NodeIdentifiers<Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(NodeIndex::new)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<N, E, Ty, Ix> visit::GraphBase for GridGraph<N, E, Ty, Ix>
where
    Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
}

impl<N, E, Ty, Ix> visit::Visitable for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Map = FixedBitSet;

    fn visit_map(&self) -> FixedBitSet {
        FixedBitSet::with_capacity(self.node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

impl<N, E, Ty, Ix> visit::GraphProp for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeType = Ty;
}

impl<'a, N, E: 'a, Ty, Ix> visit::IntoNodeIdentifiers for &'a GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type NodeIdentifiers = NodeIdentifiers<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.node_indices()
    }
}

impl<N, E, Ty, Ix> visit::NodeCount for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn node_count(&self) -> usize {
        self.node_count()
    }
}

impl<N, E, Ty, Ix> visit::NodeIndexable for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    #[inline]
    fn node_bound(&self) -> usize {
        self.node_bound()
    }
    #[inline]
    fn to_index(&self, ix: NodeIndex<Ix>) -> usize {
        ix.index()
    }
    #[inline]
    fn from_index(&self, ix: usize) -> Self::NodeId {
        NodeIndex::new(ix)
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        self.nodes[index.index()].as_ref().unwrap()
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        self.nodes[index.index()].as_mut().unwrap()
    }
}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> Index<EdgeIndex<Ix>> for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = E;
    fn index(&self, index: EdgeIndex<Ix>) -> &E {
        &self.edges[index.index()].as_ref().unwrap().weight
    }
}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> IndexMut<EdgeIndex<Ix>> for GridGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
        &mut self.edges[index.index()].as_mut().unwrap().weight
    }
}

#[cfg(test)]
mod grid_shape_tests {
    use super::*;

    #[test]
    fn node_point() {
        let shape = GridShape::new(3, 2);

        assert_eq!(shape.node_point(0), Some(GridPoint::new(0, 0)));
        assert_eq!(shape.node_point(2), Some(GridPoint::new(2, 0)));
        assert_eq!(shape.node_point(3), Some(GridPoint::new(0, 1)));
        assert_eq!(shape.node_point(5), Some(GridPoint::new(2, 1)));
    }

    #[test]
    fn edge_index_between() {
        let shape = GridShape::new(3, 2);

        // *---*...o...
        // :   :   :
        // o...o...o...
        assert_eq!(shape.edge_index_between(0, 1), Some(0));
        assert_eq!(shape.edge_index_between(1, 0), Some(0));
        // *...o...o...
        // |   :   :
        // *...o...o...
        assert_eq!(shape.edge_index_between(0, 3), Some(3));
        assert_eq!(shape.edge_index_between(3, 0), Some(3));
        // o...o...o...
        // :   :   :
        // o...o---o...
        assert_eq!(shape.edge_index_between(4, 5), Some(7));
        assert_eq!(shape.edge_index_between(5, 4), Some(7));

        // not neighbors
        assert!(shape.edge_index_between(3, 3).is_none());
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn add_node() {
        let mut g = UnGridGraph::<&str, ()>::with_grid(3, 2);

        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let d = g.add_node("D");

        assert_eq!(a, NodeIndex::new(0));
        assert_eq!(b, NodeIndex::new(1));
        assert_eq!(c, NodeIndex::new(2));
        assert_eq!(d, NodeIndex::new(3));
    }

    #[test]
    fn add_edges() {
        let mut g = DiGridGraph::<&str, i32>::with_grid(3, 2);

        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let d = g.add_node("D");

        let e1 = g.add_edge(a, b, 10);
        let e2 = g.add_edge(b, c, 20);
        let e3 = g.add_edge(a, d, 30);

        assert_eq!(e1, EdgeIndex::new(0));
        assert_eq!(e2, EdgeIndex::new(1));
        assert_eq!(e3, EdgeIndex::new(3));

        // get weight
        assert_eq!(g.edge_weight(e1), Some(&10));
        assert_eq!(g.edge_weight(e2), Some(&20));
        assert_eq!(g.edge_weight(e3), Some(&30));

        // find edge
        assert_eq!(g.find_edge(a, b), Some(e1));
        assert_eq!(g.find_edge(b, c), Some(e2));
        assert_eq!(g.find_edge(a, d), Some(e3));
        assert_eq!(g.find_edge(a, c), None);

        // remove node
        assert_eq!(g.remove_node(b), Some("B"));
        assert_eq!(g.find_edge(a, b), None);
        assert_eq!(g.find_edge(b, c), None);
    }

    #[test]
    fn remove_node() {
        let mut g = DiGridGraph::<&str, i32>::with_grid(3, 2);

        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let _ = g.add_node("D");

        assert_eq!(g.remove_node(b), Some("B"));
        assert_eq!(g.find_edge(a, b), None);
        assert_eq!(g.find_edge(b, c), None);
    }

    #[test]
    fn remove_edge() {
        let mut g = DiGridGraph::<&str, i32>::with_grid(3, 2);

        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let d = g.add_node("D");

        let e1 = g.add_edge(a, b, 10);
        let e2 = g.add_edge(b, c, 20);
        let e3 = g.add_edge(a, d, 30);

        assert_eq!(g.remove_edge(e1), Some(10));
        assert_eq!(g.find_edge(a, b), None);
        assert_eq!(g.remove_edge(e2), Some(20));
        assert_eq!(g.find_edge(b, c), None);
        assert_eq!(g.remove_edge(e3), Some(30));
        assert_eq!(g.find_edge(a, d), None);
    }

    #[test]
    #[should_panic]
    fn add_edge_collision() {
        let mut g = DiGridGraph::<&str, i32>::with_grid(3, 2);

        let a = g.add_node("A");
        let b = g.add_node("B");

        g.add_edge(a, b, 10);

        // We CAN'T update an existing edge even if the outgoing edge with an incoming edge.
        g.add_edge(b, a, 10);
    }
}
