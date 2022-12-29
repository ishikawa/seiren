/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = PAD, { erd_diagram }, PAD ;
erd_diagram = "erd", PAD, [ identifier, PAD ], "{", stmts, "}" ;
stmts = PAD, stmt, { SP, SEP, stmt }, PAD
      | EMPTY ;
stmt = ( entity_definition | relation ) ;
entity_definition = identifier, SP, "{", fields, "}" ;
fields = [ field ], { SEP, field } ;
fields = PAD, field, { SP, SEP, field }, PAD
       | EMPTY ;
field = identifier, SP, field_type, [ SP, field_key ] ;
field_type = "int" | "uuid" | "text" | "timestamp" ;
field_key = "PK" | "FK" ;
relation = entity, { PAD, edge, PAD, entity } ;
entity = identifier, [ ".", identifier ] ;
edge = "o", "--", "o" ;
identifier = identifier_start, { identifier_continue } ;
identifier_start = "_" | letter ;
identifier_continue = "_" | letter | digit ;
letter = ? a-zA-Z ? ;
digit = ? 0-9 ? ;
whitespace = ? whitespace ? ;
newline = ? newline ? ;
PAD = { whitespace | newline } ;
SP = { whitespace } ;
SEP = newline ;
EMPTY = () ;
```
*/

use chumsky::prelude::*;
use derive_builder::Builder;
use derive_more::Display;

#[derive(Debug, Display)]
pub enum Stmt {
    Expr(Expr),
    EntityDefinition(EntityDefinition),
}

#[derive(Debug, Clone, Default, Builder, Display)]
#[display(fmt = "{} {{}}", name)]
#[builder(default)]
pub struct EntityDefinition {
    #[builder(setter(into))]
    pub name: String,
}

#[derive(Debug, Display)]
pub enum Expr {
    #[display(fmt = "({} o--o {})", _0, _1)]
    Relation(Box<Expr>, Box<Expr>),
    #[display(fmt = "{}", _0)]
    Entity(EntityPath),
}

#[derive(Debug, Display)]
pub enum EntityPath {
    #[display(fmt = "{}", _0)]
    Table(String),
    #[display(fmt = "{}.{}", _0, _1)]
    Column(String, String),
}

pub fn parser() -> impl Parser<char, Vec<Stmt>, Error = Simple<char>> {
    stmts().padded().then_ignore(end())
}

fn spaces() -> impl Parser<char, String, Error = Simple<char>> {
    one_of::<_, _, Simple<char>>(" \t")
        .repeated()
        .collect::<String>()
}

fn stmts() -> impl Parser<char, Vec<Stmt>, Error = Simple<char>> {
    stmt()
        .chain(
            spaces()
                .ignore_then(text::newline())
                .ignore_then(stmt())
                .repeated(),
        )
        .or_not()
        .padded()
        .map(|stmts| stmts.unwrap_or_else(|| vec![]))
}

fn stmt() -> impl Parser<char, Stmt, Error = Simple<char>> {
    entity_definition().or(relation_stmt())
}

fn relation_stmt() -> impl Parser<char, Stmt, Error = Simple<char>> {
    relation().map(|expr| Stmt::Expr(expr))
}

fn relation() -> impl Parser<char, Expr, Error = Simple<char>> {
    entity()
        .then(just("o--o").padded().ignore_then(entity()).repeated())
        .foldl(|a, b| Expr::Relation(Box::new(a), Box::new(b)))
}

fn entity_definition() -> impl Parser<char, Stmt, Error = Simple<char>> {
    text::ident()
        .then(just("{").padded())
        .then(just("}"))
        .map(|((name, _), _)| {
            let definition = EntityDefinitionBuilder::default()
                .name(name)
                .build()
                .unwrap();
            Stmt::EntityDefinition(definition)
        })
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
