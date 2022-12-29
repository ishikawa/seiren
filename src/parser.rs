/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = SP, { erd_diagram }, SP ;
erd_diagram = "erd", SP, [ identifier, SP ], "{", stmts, "}" ;
stmts = [ stmt ], { SEP, stmt } ;
stmt = SP, ( entity_definition | relation ), SP ;
entity_definition = identifier, SP, "{", fields, "}" ;
fields = [ field ], { SEP, field } ;
field = SP, identifier, SP, field_type, [ SP, field_key ], SP ;
field_type = "int" | "uuid" | "text" | "timestamp" ;
field_key = "PK" | "FK" ;
relation = entity, { SP, edge, SP, entity } ;
entity = identifier, [ ".", identifier ] ;
edge = "o", "--", "o" ;
identifier = identifier_start, { identifier_continue } ;
identifier_start = "_" | letter ;
identifier_continue = "_" | letter | digit ;
letter = ? a-zA-Z ? ;
digit = ? 0-9 ? ;
whitespace = ? whitespace ? ;
SP = { whitespace } ;
SEP = "\n" | "\r\n" ;
```
*/

use chumsky::prelude::*;
use derive_more::Display;

#[derive(Debug, Display)]
pub enum Stmt {
    Expr(Expr),
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

pub fn parser() -> impl Parser<char, Stmt, Error = Simple<char>> {
    stmt().padded().then_ignore(end())
}

fn stmt() -> impl Parser<char, Stmt, Error = Simple<char>> {
    relation().padded().map(|expr| Stmt::Expr(expr))
}

fn relation() -> impl Parser<char, Expr, Error = Simple<char>> {
    entity()
        .then(just("o--o").padded().ignore_then(entity()).repeated())
        .foldl(|a, b| Expr::Relation(Box::new(a), Box::new(b)))
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
