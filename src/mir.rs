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
use crate::geometry::{Point, Size};
use derive_builder::Builder;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub origin: Option<Point>,
    pub size: Option<Size>,
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
            children: vec![],
        }
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn children(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.children.iter().copied()
    }

    pub fn append_child(&mut self, node_id: NodeId) {
        self.children.push(node_id);
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Body(BodyNode),
    Record(RecordNode),
    Field(FieldNode),
}

#[derive(Debug)]
pub struct Document {
    nodes: Vec<Node>,
}

impl Document {
    pub fn new() -> Self {
        let node_id = NodeId(0);
        let node = Node::new(node_id, NodeKind::Body(BodyNode::default()));

        Self { nodes: vec![node] }
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
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct BodyNode {}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct RecordNode {
    pub rounded: bool,
    pub border_color: WebColor,
    pub bg_color: WebColor,
    #[builder(setter(strip_option))]
    pub header: Option<RecordNodeHeader>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct RecordNodeHeader {
    #[builder(setter(into))]
    pub title: String,
    pub text_color: WebColor,
    pub bg_color: WebColor,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct FieldNode {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub r#type: String,
    pub text_color: WebColor,
    pub type_color: WebColor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_doc() {
        let mut doc = Document::new();

        let field = FieldNodeBuilder::default().name("id").build().unwrap();

        let node_id = doc.create_field(field);

        let node = doc.get_node(&node_id);
        assert!(node.is_some());

        let node = node.unwrap();
        let NodeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.name, "id");

        // mutate
        {
            let node = doc.get_node_mut(&node_id).unwrap();
            let NodeKind::Field(field) = &mut node.kind else { panic!() };

            field.name = "uuid".to_string();
        }

        let node = doc.get_node(&node_id).unwrap();
        let NodeKind::Field(field) = &node.kind else { panic!() };

        assert_eq!(field.name, "uuid");
    }
}
