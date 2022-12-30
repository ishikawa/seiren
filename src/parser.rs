/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = PAD, erd_module, PAD ;
erd_module = "erd", PAD, [ identifier, PAD ], "{", PAD, module_entries, PAD, "}", PAD ;
module_entries = module_entry, { SP, SEP, PAD, module_entry }
               | EMPTY
module_entry = definition | stmt ;
definition = entity_definition ;
stmt = relation ;
entity_definition = identifier, PAD, "{", entity_fields, "}" ;
entity_fields = PAD, entity_field, { SP, SEP, PAD, entity_field }, PAD
              | EMPTY ;
entity_field = identifier, SP, entity_field_type, [ SP, entity_field_type ] ;
entity_field_type = "int" | "uuid" | "text" | "timestamp" ;
entity_field_key = "PK" | "FK" ;
relation = entity, { PAD, edge, PAD, entity } ;
entity = identifier, [ ".", identifier ] ;
edge = "o", "--", "o" ;
identifier = identifier_start, { identifier_continue } ;
identifier_start = "_" | letter ;
identifier_continue = "_" | letter | digit ;
letter = ? a-zA-Z ? ;
digit = ? 0-9 ? ;
whitespace = ? whitespace ? ;
newline = "\n" | "\r\n" ;
PAD = { whitespace | newline } ;
SP = { whitespace } ;
SEP = newline | ";" ;
EMPTY = ? (empty) ? ;
```
*/

use std::fmt;

use chumsky::prelude::*;
use derive_builder::Builder;
use derive_more::Display;

use crate::erd::{ColumnKey, ColumnType};

#[derive(Debug, Builder)]
pub struct ErdModule {
    pub name: Option<String>,
    #[builder(setter(each(name = "entry")))]
    pub entries: Vec<ModuleEntry>,
}

impl fmt::Display for ErdModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "erd {{")?;
        for entry in self.entries.iter() {
            writeln!(f, "  {}", entry)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Display)]
pub enum ModuleEntry {
    Definition(Definition),
    Stmt(Stmt),
}

#[derive(Debug, Clone, Display)]
pub enum Definition {
    EntityDefinition(EntityDefinition),
}

#[derive(Debug, Clone, Display)]
pub enum Stmt {
    Expr(Expr),
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(default)]
pub struct EntityDefinition {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(each(name = "field")))]
    pub fields: Vec<EntityField>,
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

#[derive(Debug, Clone, Builder)]
pub struct EntityField {
    #[builder(setter(into))]
    pub name: String,
    pub field_type: ColumnType,
    pub field_key: Option<ColumnKey>,
}

impl fmt::Display for EntityField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.field_type)?;
        let Some(field_key) = self.field_key else { return Ok(()) };
        write!(f, " {}", field_key.to_keyword())
    }
}

#[derive(Debug, Clone, Display)]
pub enum Expr {
    #[display(fmt = "({} o--o {})", _0, _1)]
    Relation(Box<Expr>, Box<Expr>),
    #[display(fmt = "{}", _0)]
    Entity(EntityPath),
}

#[derive(Debug, Clone, Display)]
pub enum EntityPath {
    #[display(fmt = "{}", _0)]
    Table(String),
    #[display(fmt = "{}.{}", _0, _1)]
    Column(String, String),
}

pub fn parser() -> impl Parser<char, ErdModule, Error = Simple<char>> {
    erd_module().padded().then_ignore(end())
}

fn spaces() -> impl Parser<char, String, Error = Simple<char>> {
    one_of::<_, _, Simple<char>>(" \t")
        .repeated()
        .collect::<String>()
}

fn separator() -> impl Parser<char, String, Error = Simple<char>> {
    choice((just("\n"), just("\r\n"), just(";"))).map(|x| x.to_string())
}

fn erd_module() -> impl Parser<char, ErdModule, Error = Simple<char>> {
    just("erd")
        .ignore_then(text::ident().padded().or_not())
        .then_ignore(just("{").padded())
        .then(module_entries())
        .then_ignore(just("}").padded())
        .map(|(name, entries)| {
            ErdModuleBuilder::default()
                .name(name)
                .entries(entries)
                .build()
                .unwrap()
        })
}

fn module_entries() -> impl Parser<char, Vec<ModuleEntry>, Error = Simple<char>> {
    module_entry()
        .chain(
            spaces()
                .ignore_then(separator())
                .ignore_then(text::whitespace())
                .ignore_then(module_entry())
                .repeated(),
        )
        .or_not()
        .padded()
        .map(|entries| entries.unwrap_or_else(|| vec![]))
}

fn module_entry() -> impl Parser<char, ModuleEntry, Error = Simple<char>> {
    choice((
        definition().map(|d| ModuleEntry::Definition(d)),
        stmt().map(|stmt| ModuleEntry::Stmt(stmt)),
    ))
}

fn definition() -> impl Parser<char, Definition, Error = Simple<char>> {
    entity_definition().map(|ed| Definition::EntityDefinition(ed))
}

fn stmt() -> impl Parser<char, Stmt, Error = Simple<char>> {
    relation().map(|expr| Stmt::Expr(expr))
}

fn relation() -> impl Parser<char, Expr, Error = Simple<char>> {
    entity()
        .then(just("o--o").padded().ignore_then(entity()).repeated())
        .foldl(|a, b| Expr::Relation(Box::new(a), Box::new(b)))
}

fn entity_definition() -> impl Parser<char, EntityDefinition, Error = Simple<char>> {
    text::ident()
        .then_ignore(just("{").padded())
        .then(entity_fields())
        .then_ignore(just("}"))
        .map(|(name, fields)| {
            EntityDefinitionBuilder::default()
                .name(name)
                .fields(fields)
                .build()
                .unwrap()
        })
}

fn entity_fields() -> impl Parser<char, Vec<EntityField>, Error = Simple<char>> {
    entity_field()
        .chain(
            spaces()
                .ignore_then(separator())
                .ignore_then(text::whitespace())
                .ignore_then(entity_field())
                .repeated(),
        )
        .or_not()
        .padded()
        .map(|fields| fields.unwrap_or_else(|| vec![]))
}

fn entity_field() -> impl Parser<char, EntityField, Error = Simple<char>> {
    text::ident()
        .then_ignore(spaces())
        .then(entity_field_type())
        .then(spaces().ignore_then(entity_field_key()).or_not())
        .map(|((name, field_type), field_key)| {
            EntityFieldBuilder::default()
                .name(name)
                .field_type(field_type)
                .field_key(field_key)
                .build()
                .unwrap()
        })
}

fn entity_field_type() -> impl Parser<char, ColumnType, Error = Simple<char>> {
    // TODO: iterate enum variants
    choice((
        text::keyword("int").to(ColumnType::Int),
        text::keyword("uuid").to(ColumnType::Uuid),
        text::keyword("text").to(ColumnType::Text),
        text::keyword("timestamp").to(ColumnType::Timestamp),
    ))
}

fn entity_field_key() -> impl Parser<char, ColumnKey, Error = Simple<char>> {
    // TODO: iterate enum variants
    choice((
        text::keyword("PK").to(ColumnKey::PrimaryKey),
        text::keyword("FK").to(ColumnKey::ForeginKey),
    ))
}

fn entity() -> impl Parser<char, Expr, Error = Simple<char>> {
    text::ident()
        .then(just(".").ignore_then(text::ident()).or_not())
        .map(|(table, column)| {
            if let Some(column) = column {
                Expr::Entity(EntityPath::Column(table, column))
            } else {
                Expr::Entity(EntityPath::Table(table))
            }
        })
}
