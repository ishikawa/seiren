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

#[derive(Debug, Clone, Builder)]
pub struct Record {
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
    pub fields: Vec<RecordField>,
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
pub struct RecordField {
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
    fn build_record() {
        let record = RecordBuilder::default().build().unwrap();

        assert_eq!(record.origin, Point::default());
        assert_eq!(record.size, Size::default());

        // 2
        let header = RecordHeaderBuilder::default()
            .title("users")
            .build()
            .unwrap();
        let field1 = RecordFieldBuilder::default()
            .name("id")
            .r#type("int")
            .build()
            .unwrap();
        let field2 = RecordFieldBuilder::default()
            .name("uuid")
            .r#type("uuid")
            .build()
            .unwrap();

        let record = RecordBuilder::default()
            .header(header)
            .field(field1)
            .field(field2)
            .build()
            .unwrap();

        assert_eq!(record.header.unwrap().title, "users");
        assert_eq!(record.fields.len(), 2);
    }
}
