//! Mid-level IR
//!
//! coordinate system: top-left origin
//!
//! ```svgbob
//! +--------------------------
//! | (0, 0)           (300, 0)
//! |
//! |
//! |
//! | (0, 100)
//! ```
use crate::color::WebColor;
use crate::geometry::{Orientation, Point, Rect, Size};
use derive_builder::Builder;
use derive_more::Display;
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(NodeIndex);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(EdgeIndex);

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{}:{}", _0, _1)]
pub struct TerminalPortId(NodeId, usize);

#[derive(Debug)]
pub struct NodeData {
    /// The origin (absolute in the global coordination)
    pub origin: Option<Point>,
    pub size: Option<Size>,

    /// Points to which edges can be connected.
    terminal_ports: Vec<TerminalPort>,
    kind: ShapeKind,

    /// For container shapes.
    children: Vec<NodeId>,
}

impl NodeData {
    pub fn new(kind: ShapeKind) -> Self {
        Self {
            kind,
            origin: None,
            size: None,
            terminal_ports: vec![],
            children: vec![],
        }
    }

    pub fn kind(&self) -> &ShapeKind {
        &self.kind
    }

    // --- Children

    pub fn children(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.children.iter().copied()
    }

    pub fn append_child(&mut self, node_id: NodeId) {
        self.children.push(node_id);
    }

    // --- Geometry
    pub fn rect(&self) -> Option<Rect> {
        self.origin
            .and_then(|origin| self.size.map(|size| Rect::new(origin, size)))
    }

    // --- Connection points
    pub fn terminal_ports(&self) -> impl ExactSizeIterator<Item = &TerminalPort> {
        self.terminal_ports.iter()
    }

    pub fn add_terminal_port(
        &mut self,
        node_id: NodeId,
        location: Point,
        orientation: Orientation,
    ) -> TerminalPortId {
        let pid = TerminalPortId(node_id, self.terminal_ports.len());

        self.terminal_ports
            .push(TerminalPort::new(pid, location, orientation));
        pid
    }
}

// --- Edge
#[derive(Debug)]
pub struct EdgeData {
    source_id: NodeId,
    target_id: NodeId,
    path_points: Option<Vec<Point>>,
}

impl EdgeData {
    pub fn new(source_id: NodeId, target_id: NodeId, path_points: Option<Vec<Point>>) -> Self {
        Self {
            source_id,
            target_id,
            path_points,
        }
    }

    pub fn source_id(&self) -> NodeId {
        self.source_id
    }

    pub fn target_id(&self) -> NodeId {
        self.target_id
    }

    pub fn path_points(&self) -> Option<&[Point]> {
        self.path_points.as_deref()
    }

    pub fn set_path_points(&mut self, path_points: Option<Vec<Point>>) {
        self.path_points = path_points;
    }
}

#[derive(Debug)]
pub enum ShapeKind {
    Body(BodyShape),
    Record(RecordShape),
    Field(FieldShape),
}

#[derive(Debug, Clone)]
pub struct TerminalPort {
    id: TerminalPortId,
    location: Point,

    /// Angle at which incoming edges can incident to or outgoing edges can exit.
    orientation: Orientation,
}

impl TerminalPort {
    pub fn new(id: TerminalPortId, location: Point, orientation: Orientation) -> Self {
        Self {
            id,
            location,
            orientation,
        }
    }

    pub fn id(&self) -> TerminalPortId {
        self.id
    }

    pub fn location(&self) -> &Point {
        &self.location
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}

type DocumentGraph = UnGraph<NodeData, EdgeData>;

#[derive(Debug)]
pub struct Document {
    graph: DocumentGraph,
    body_id: NodeId,
}

impl Document {
    pub fn new() -> Self {
        let node = NodeData::new(ShapeKind::Body(BodyShape::default()));
        let mut graph = DocumentGraph::new_undirected();
        let body_index = graph.add_node(node);

        Self {
            graph,
            body_id: NodeId(body_index),
        }
    }

    pub fn body(&self) -> &NodeData {
        self.graph.node_weight(self.body_id.0).unwrap()
    }

    pub fn body_mut(&mut self) -> &mut NodeData {
        self.graph.node_weight_mut(self.body_id.0).unwrap()
    }

    // -- Get a node

    pub fn get_node(&self, node_id: NodeId) -> Option<&NodeData> {
        self.graph.node_weight(node_id.0)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut NodeData> {
        self.graph.node_weight_mut(node_id.0)
    }

    // -- Create a node

    pub fn create_record(&mut self, record: RecordShape) -> NodeId {
        let node = NodeData::new(ShapeKind::Record(record));
        let index = self.graph.add_node(node);

        NodeId(index)
    }

    pub fn create_field(&mut self, field: FieldShape) -> NodeId {
        let node = NodeData::new(ShapeKind::Field(field));
        let index = self.graph.add_node(node);

        NodeId(index)
    }

    // --- Edge
    pub fn edge_endpoints(&self, edge_id: EdgeId) -> Option<(NodeId, NodeId)> {
        self.graph
            .edge_endpoints(edge_id.0)
            .map(|(x, y)| (NodeId(x), NodeId(y)))
    }

    pub fn edge(&self, edge_id: EdgeId) -> Option<&EdgeData> {
        self.graph.edge_weight(edge_id.0)
    }

    pub fn edge_ids(&self) -> impl ExactSizeIterator<Item = EdgeId> {
        self.graph.edge_indices().map(|i| EdgeId(i))
    }

    pub fn edges(&self) -> impl Iterator<Item = &EdgeData> {
        self.graph.edge_weights()
    }

    pub fn edges_mut(&mut self) -> impl Iterator<Item = &mut EdgeData> {
        self.graph.edge_weights_mut()
    }

    pub fn add_edge(&mut self, edge: EdgeData) -> EdgeId {
        let index = self
            .graph
            .add_edge(edge.source_id().0, edge.target_id().0, edge);
        EdgeId(index)
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct BodyShape {}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct RecordShape {
    pub rounded: bool,
    pub bg_color: Option<WebColor>,
    pub border_color: Option<WebColor>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct FieldShape {
    pub title: TextSpan,
    pub subtitle: Option<TextSpan>,
    pub badge: Option<Badge>,
    pub bg_color: Option<WebColor>,
    pub border_color: Option<WebColor>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct TextSpan {
    #[builder(setter(into))]
    pub text: String,
    pub color: Option<WebColor>,
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
    pub font_size: Option<FontSize>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct Badge {
    #[builder(setter(into))]
    pub text: String,
    pub color: Option<WebColor>,
    pub bg_color: Option<WebColor>,
}

impl Badge {
    pub fn into_text_span(&self) -> TextSpan {
        TextSpanBuilder::default()
            .text(self.text.to_string())
            .color(self.color.clone())
            .font_family(Some(FontFamily::SansSerif3))
            .font_size(Some(FontSize::XXSmall))
            .build()
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Display)]
pub enum FontFamily {
    #[display(fmt = "Arial,sans-serif")]
    SansSerif1,
    #[display(fmt = "Verdana,sans-serif")]
    SansSerif2,
    #[display(fmt = "Trebuchet MS,sans-serif")]
    SansSerif3,
    #[display(fmt = "Monaco,Lucida Console,monospace")]
    Monospace1,
    #[display(fmt = "Courier New,monospace")]
    Monospace2,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Display)]
pub enum FontWeight {
    #[display(fmt = "normal")]
    Normal,
    #[display(fmt = "bold")]
    Bold,
    #[display(fmt = "lighter")]
    Lighter,
    #[display(fmt = "bolder")]
    Bolder,
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Display)]
pub enum FontSize {
    /* <absolute-size> values */
    #[display(fmt = "xx-small")]
    XXSmall,
    #[display(fmt = "x-small")]
    XSmall,
    #[display(fmt = "small")]
    Small,
    #[display(fmt = "medium")]
    Medium,
    #[display(fmt = "large")]
    Large,
    #[display(fmt = "x-large")]
    XLarge,
    #[display(fmt = "xx-large")]
    XXLarge,
    #[display(fmt = "xxx-large")]
    XXXLarge,
}

impl Default for FontSize {
    fn default() -> Self {
        Self::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_doc() {
        let mut doc = Document::new();

        let title = TextSpanBuilder::default().text("id").build().unwrap();
        let field = FieldShapeBuilder::default().title(title).build().unwrap();

        let node_index = doc.create_field(field);

        let node = doc.get_node(node_index);
        assert!(node.is_some());

        let node = node.unwrap();
        let ShapeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.title.text, "id");

        // mutate
        {
            let node = doc.get_node_mut(node_index).unwrap();
            let ShapeKind::Field(field) = &mut node.kind else { panic!() };

            field.title.text = "uuid".to_string();
        }

        let node = doc.get_node(node_index).unwrap();
        let ShapeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.title.text, "uuid");
    }
}
