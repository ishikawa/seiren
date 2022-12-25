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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug)]
pub enum Node {
    Record(RecordNode),
    Field(FieldNode),
}

pub struct NodeId(usize);

#[derive(Debug, Default)]
pub struct Document {
    nodes: Vec<Node>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes.get(node_id.0)
    }

    pub fn add_record(&mut self, field: RecordNode) -> NodeId {
        let index = self.nodes.len();

        self.nodes.push(Node::Record(field));
        NodeId(index)
    }

    pub fn add_field(&mut self, field: FieldNode) -> NodeId {
        let index = self.nodes.len();

        self.nodes.push(Node::Field(field));
        NodeId(index)
    }
}

#[derive(Debug, Clone, Builder)]
pub struct RecordNode {
    #[builder(default)]
    pub origin: Point,
    #[builder(default)]
    pub size: Size,
    #[builder(default)]
    pub rounded: bool,
    #[builder(default)]
    pub border_color: WebColor,
    #[builder(default)]
    pub bg_color: WebColor,
    #[builder(setter(strip_option), default)]
    pub header: Option<RecordHeader>,
    #[builder(setter(each(name = "field")), default)]
    pub fields: Vec<FieldNode>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct RecordHeader {
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

        let node_id = doc.add_field(field);

        let node = doc.get_node(&node_id);

        assert!(node.is_some());

        let Node::Field(field) = node.unwrap() else { panic!() };

        assert_eq!(field.name, "id");
    }
}
