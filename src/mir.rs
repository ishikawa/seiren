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
use derive_builder::Builder;
use std::fmt;

/// https://developer.mozilla.org/en-US/docs/Web/SVG/Content_type#length
///
/// length used in SVG:
/// ```
/// length ::= number ("em" | "ex" | "px" | "in" | "cm" | "mm" | "pt" | "pc" | "%")?
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Length {
    pub value: f32,
    pub unit: Option<LengthUnit>,
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: Length,
    pub y: Length,
}

impl Point {
    pub fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    pub fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }
}

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
    pub parent_node_id: Option<NodeId>,
    pub origin: Option<Point>,
    pub size: Option<Size>,
    pub kind: NodeKind,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            kind,
            parent_node_id: None,
            origin: None,
            size: None,
            children: vec![],
        }
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Record(RecordNode),
    Field(FieldNode),
}

#[derive(Debug, Default)]
pub struct Document {
    nodes: Vec<Node>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
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
