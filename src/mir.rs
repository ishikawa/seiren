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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{}", _0)]
pub struct NodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{}:{}", _0, _1)]
pub struct TerminalPortId(NodeId, usize);

#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    /// The origin (absolute in the global coordination)
    pub origin: Option<Point>,
    pub size: Option<Size>,

    /// Points to which edges can be connected.
    terminal_ports: Vec<TerminalPort>,
    kind: NodeKind,
    children: Vec<NodeId>,
}

impl Node {
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            kind,
            origin: None,
            size: None,
            terminal_ports: vec![],
            children: vec![],
        }
    }

    pub fn kind(&self) -> &NodeKind {
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

    pub fn append_terminal_port(
        &mut self,
        location: Point,
        orientation: Orientation,
    ) -> TerminalPortId {
        let pid = TerminalPortId(self.id, self.terminal_ports.len());

        self.terminal_ports
            .push(TerminalPort::new(pid, location, orientation));
        pid
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Body(BodyNode),
    Record(RecordNode),
    Field(FieldNode),
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

#[derive(Debug)]
pub struct Document {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Document {
    pub fn new() -> Self {
        let node_id = NodeId(0);
        let node = Node::new(node_id, NodeKind::Body(BodyNode::default()));

        Self {
            nodes: vec![node],
            edges: vec![],
        }
    }

    pub fn body(&self) -> &Node {
        self.nodes.get(0).unwrap()
    }

    pub fn body_mut(&mut self) -> &mut Node {
        self.nodes.get_mut(0).unwrap()
    }

    // -- Get a node

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes.get(node_id.0)
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id.0)
    }

    // -- Create a node

    pub fn create_record(&mut self, record: RecordNode) -> NodeId {
        let index = self.nodes.len();
        let node_id = NodeId(index);
        let node = Node::new(node_id, NodeKind::Record(record));

        self.nodes.push(node);
        node_id
    }

    pub fn create_field(&mut self, field: FieldNode) -> NodeId {
        let index = self.nodes.len();
        let node_id = NodeId(index);
        let node = Node::new(node_id, NodeKind::Field(field));

        self.nodes.push(node);
        node_id
    }

    // --- Edge

    pub fn edges(&self) -> impl ExactSizeIterator<Item = &Edge> {
        self.edges.iter()
    }

    pub fn edges_mut(&mut self) -> impl ExactSizeIterator<Item = &mut Edge> {
        self.edges.iter_mut()
    }

    pub fn append_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct BodyNode {}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct RecordNode {
    pub rounded: bool,
    pub bg_color: Option<WebColor>,
    pub border_color: Option<WebColor>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct FieldNode {
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

// --- Edge
#[derive(Debug)]
pub struct Edge {
    pub start_node_id: NodeId,
    pub end_node_id: NodeId,
    pub path_points: Option<Vec<Point>>,
}

impl Edge {
    pub fn new(start_node: NodeId, end_node: NodeId) -> Self {
        Self {
            start_node_id: start_node,
            end_node_id: end_node,
            path_points: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_doc() {
        let mut doc = Document::new();

        let title = TextSpanBuilder::default().text("id").build().unwrap();
        let field = FieldNodeBuilder::default().title(title).build().unwrap();

        let node_id = doc.create_field(field);

        let node = doc.get_node(&node_id);
        assert!(node.is_some());

        let node = node.unwrap();
        let NodeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.title.text, "id");

        // mutate
        {
            let node = doc.get_node_mut(&node_id).unwrap();
            let NodeKind::Field(field) = &mut node.kind else { panic!() };

            field.title.text = "uuid".to_string();
        }

        let node = doc.get_node(&node_id).unwrap();
        let NodeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.title.text, "uuid");
    }
}
