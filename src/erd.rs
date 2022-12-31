//! ER diagram AST
use crate::color::{NamedColor, RGBColor, WebColor};
use crate::mir;
use derive_more::Display;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Module {
    name: Option<String>,
    entries: Vec<ModuleEntry>,
}

impl Module {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            entries: vec![],
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = &ModuleEntry> {
        self.entries.iter()
    }

    pub fn add_entry(&mut self, entry: ModuleEntry) {
        self.entries.push(entry);
    }

    pub fn add_entity_definition(&mut self, definition: EntityDefinition) {
        self.entries.push(ModuleEntry::EntityDefinition(definition));
    }

    pub fn add_entity_relation(&mut self, relation: EntityRelation) {
        self.entries.push(ModuleEntry::EntityRelation(relation));
    }

    pub fn into_mir(&self) -> mir::Document {
        let light_gray_color = WebColor::RGB(RGBColor::new(73, 73, 73));
        let table_border_color = light_gray_color.clone();
        let table_bg_color = WebColor::RGB(RGBColor::new(33, 33, 33));
        let text_color = WebColor::Named(NamedColor::White);
        let mut doc = mir::Document::new();

        // node path (e.g. ["users", "id"]) -> node ID
        let mut node_paths: HashMap<EntityPath, mir::NodeId> = HashMap::new();

        for entry in self.entries.iter() {
            match entry {
                ModuleEntry::EntityDefinition(definition) => {
                    // table
                    let header_node_id = {
                        let name = mir::TextSpanBuilder::default()
                            .text(definition.name.clone())
                            .color(Some(text_color.clone()))
                            .font_family(Some(mir::FontFamily::Monospace1))
                            .font_weight(Some(mir::FontWeight::Bold))
                            .build()
                            .unwrap();
                        let field = mir::FieldNodeBuilder::default()
                            .title(name)
                            .bg_color(Some(light_gray_color.clone()))
                            .build()
                            .unwrap();

                        doc.create_field(field)
                    };
                    let record = mir::RecordNodeBuilder::default()
                        .rounded(true)
                        .bg_color(Some(table_bg_color.clone()))
                        .border_color(Some(table_border_color.clone()))
                        .build()
                        .unwrap();
                    let field_ids: Vec<_> = definition
                        .fields
                        .iter()
                        .map(|field| {
                            let name = mir::TextSpanBuilder::default()
                                .text(field.name.clone())
                                .color(Some(text_color.clone()))
                                .font_family(Some(mir::FontFamily::Monospace2))
                                .font_weight(Some(mir::FontWeight::Lighter))
                                .build()
                                .unwrap();

                            let column_type = mir::TextSpanBuilder::default()
                                .text(field.field_type.to_string())
                                .color(Some(Module::column_type_color(&field.field_type)))
                                .font_family(Some(mir::FontFamily::Monospace2))
                                .font_weight(Some(mir::FontWeight::Lighter))
                                .font_size(Some(mir::FontSize::Small))
                                .build()
                                .unwrap();

                            let field_node = mir::FieldNodeBuilder::default()
                                .title(name)
                                .subtitle(Some(column_type))
                                .border_color(Some(table_border_color.clone()))
                                .badge(field.field_key.map(|key| key.into_mir()))
                                .build()
                                .unwrap();

                            let node_id = doc.create_field(field_node);

                            node_paths.insert(
                                EntityPath::Field(definition.name.clone(), field.name.clone()),
                                node_id,
                            );
                            node_id
                        })
                        .collect();

                    let record_id = doc.create_record(record);
                    node_paths.insert(EntityPath::Entity(definition.name.clone()), record_id);

                    let record_node = doc.get_node_mut(&record_id).unwrap();

                    record_node.append_child(header_node_id);
                    for field_id in field_ids {
                        record_node.append_child(field_id);
                    }

                    doc.body_mut().append_child(record_id);
                }
                ModuleEntry::EntityRelation(relation) => {
                    let Some(start_node_id) = node_paths.get(relation.start_path()) else { continue };
                    let Some(end_node_id) = node_paths.get(relation.end_path()) else { continue };

                    doc.append_edge(mir::Edge::new(*start_node_id, *end_node_id));
                }
            }
        }

        doc
    }

    fn column_type_color(column_type: &EntityFieldType) -> WebColor {
        let yellow = WebColor::RGB(RGBColor {
            red: 236,
            green: 199,
            blue: 0,
        });
        let orange = WebColor::RGB(RGBColor {
            red: 214,
            green: 105,
            blue: 5,
        });
        let green = WebColor::RGB(RGBColor {
            red: 6,
            green: 182,
            blue: 151,
        });

        match column_type {
            EntityFieldType::Int => yellow.clone(),
            EntityFieldType::Uuid => yellow.clone(),
            EntityFieldType::Text => orange.clone(),
            EntityFieldType::Timestamp => green.clone(),
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "erd ")?;
        if let Some(name) = &self.name {
            write!(f, "{} ", name)?;
        }
        writeln!(f, "{{")?;
        for entry in self.entries.iter() {
            writeln!(f, "    {}", entry)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Display)]
pub enum ModuleEntry {
    EntityDefinition(EntityDefinition),
    EntityRelation(EntityRelation),
}

#[derive(Debug, Clone, Default)]
pub struct EntityDefinition {
    name: String,
    fields: Vec<EntityField>,
}

impl EntityDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = &EntityField> {
        self.fields.iter()
    }

    pub fn add_field(&mut self, column: EntityField) {
        self.fields.push(column);
    }
}

impl fmt::Display for EntityDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{", self.name)?;
        if self.fields.len() > 0 {
            write!(f, " ")?;

            let mut it = self.fields.iter().peekable();

            while let Some(field) = it.next() {
                write!(f, "{}", field)?;
                if it.peek().is_some() {
                    write!(f, "; ")?;
                }
            }

            write!(f, " ")?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone)]
pub struct EntityField {
    name: String,
    field_type: EntityFieldType,
    field_key: Option<EntityFieldKey>,
}

impl EntityField {
    pub fn new(
        name: String,
        field_type: EntityFieldType,
        field_key: Option<EntityFieldKey>,
    ) -> Self {
        Self {
            name,
            field_type,
            field_key,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_type(&self) -> &EntityFieldType {
        &self.field_type
    }

    pub fn field_key(&self) -> Option<&EntityFieldKey> {
        self.field_key.as_ref()
    }
}

impl fmt::Display for EntityField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.field_type)?;
        let Some(field_key) = self.field_key else { return Ok(()) };
        write!(f, " {}", field_key.to_keyword())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum EntityFieldType {
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "uuid")]
    Uuid,
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "timestamp")]
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum EntityFieldKey {
    #[display(fmt = "Primary Key")]
    PrimaryKey,
    #[display(fmt = "Foregin Key")]
    ForeginKey,
}

impl EntityFieldKey {
    pub fn into_mir(&self) -> mir::Badge {
        mir::BadgeBuilder::default()
            .text(self.badge_text())
            .color(Some(self.badge_text_color()))
            .bg_color(Some(self.badge_bg_color()))
            .build()
            .unwrap()
    }

    pub fn to_keyword(&self) -> String {
        match self {
            EntityFieldKey::PrimaryKey => "PK".into(),
            EntityFieldKey::ForeginKey => "FK".into(),
        }
    }

    fn badge_text(&self) -> String {
        self.to_keyword()
    }

    fn badge_text_color(&self) -> WebColor {
        match self {
            EntityFieldKey::PrimaryKey => WebColor::Named(NamedColor::White),
            EntityFieldKey::ForeginKey => WebColor::RGB(RGBColor::new(17, 112, 251)),
        }
    }

    fn badge_bg_color(&self) -> WebColor {
        match self {
            EntityFieldKey::PrimaryKey => WebColor::RGB(RGBColor::new(55, 55, 55)),
            EntityFieldKey::ForeginKey => WebColor::RGB(RGBColor::new(32, 41, 55)),
        }
    }
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum EntityPath {
    #[display(fmt = "{}", _0)]
    Entity(String),
    #[display(fmt = "{}.{}", _0, _1)]
    Field(String, String),
}

#[derive(Debug, Clone, Display)]
#[display(fmt = "{} o--o {}", start_path, end_path)]
pub struct EntityRelation {
    start_path: EntityPath,
    end_path: EntityPath,
}

impl EntityRelation {
    pub fn new(start_path: EntityPath, end_path: EntityPath) -> Self {
        Self {
            start_path,
            end_path,
        }
    }

    pub fn start_path(&self) -> &EntityPath {
        &self.start_path
    }

    pub fn end_path(&self) -> &EntityPath {
        &self.end_path
    }
}
