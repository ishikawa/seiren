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
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// A point with integer coordinates.
///
/// **(m, n) grid** is the set of grid points `(x, y)` with `0 <= x <= m, 0 <= y <= n`.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(fmt = "({}, {})", _0, _1)]
pub struct GridPoint(u16, u16);

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

    /// Return the upper bound of the node indices in a grid.
    pub fn node_bound(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// Return the upper bound of the edge indices in a grid.
    pub fn edge_bound(&self) -> usize {
        if self.rows == 0 {
            0
        } else {
            (self.columns * (self.rows * 2 - 1)) as usize
        }
    }
}

// The default index type. The max size of the type must be large enough that holds `m x n`.
type DefaultIx = u32;

pub struct GridGraph<N, E, Ty = Directed, Ix = DefaultIx> {
    shape: GridShape,
    // nodes = `m x n` vector
    nodes: Vec<Option<N>>,
    // nodes = `m x (n * 2 - 1)` vector
    edges: Vec<Option<E>>,
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
        self.edges[index.index()].as_ref().unwrap()
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
        self.edges[index.index()].as_mut().unwrap()
    }
}
