/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = erd_module ;
erd_module = PAD, "erd", PAD, [ identifier, PAD ], "{", PAD, module_entries, PAD, "}", PAD ;
module_entries = module_entry, { SEP, PAD, module_entry }
               | EMPTY ;
module_entry = entity_definition | relation ;
entity_definition = identifier, PAD, "{", entity_fields, "}" ;
entity_fields = PAD, entity_field, { SEP, PAD, entity_field }, PAD
              | EMPTY ;
entity_field = identifier, entity_field_type, [ entity_field_type ] ;
entity_field_type = "int" | "uuid" | "text" | "timestamp" ;
entity_field_key = "PK" | "FK" ;
relation = entity, PAD, edge, PAD, entity ;
entity = identifier, [ ".", identifier ] ;
edge = "o", "--", "o" ;
identifier = identifier_start, { identifier_continue }
           | quoted_identifier ;
identifier_start = "_" | letter ;
identifier_continue = "_" | letter | digit ;
quoted_identifier = "`", { ? any character or escaped character ? }, "`" ;
letter = ? a-zA-Z ? ;
digit = ? 0-9 ? ;
whitespace = ? whitespace ? ;
newline = "\n" | "\r\n" ;
PAD = { whitespace | newline } ;
SEP = newline | ";" ;
EMPTY = ? (empty) ? ;
```
*/
use crate::erd::{EntityDefinition, EntityField, EntityRelation};
use crate::erd::{EntityFieldKey, EntityFieldType, EntityPath, Module, ModuleEntry};
use chumsky::prelude::*;
use chumsky::Stream;
use derive_more::Display;

pub type Span = std::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Display)]
pub enum Token {
    // Operator
    #[display(fmt = "o--o")]
    Edge,
    // Identifier
    #[display(fmt = "{}", _0)]
    Ident(String),
    // Keywords
    #[display(fmt = "erd")]
    Erd,
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "uuid")]
    Uuid,
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "timestamp")]
    Timestamp,
    #[display(fmt = "PK")]
    PK,
    #[display(fmt = "FK")]
    FK,
    // Control characters (delimiters, semicolons, etc.)
    #[display(fmt = "'{}'", _0)]
    Ctrl(char),
    #[display(fmt = "'\\n'")]
    Newline,
}

pub fn parse(src: &str) -> (Option<Module>, Vec<Simple<char>>, Vec<Simple<Token>>) {
    let (tokens, errs) = tokenizer().parse_recovery(src);

    if let Some(tokens) = tokens {
        let len = src.chars().count();
        let eoi = len..len + 1;

        let (ast, parse_errs) =
            erd_module_parser().parse_recovery(Stream::from_iter(eoi, tokens.into_iter()));

        return (ast, errs, parse_errs);
    }

    (None, errs, vec![])
}

fn tokenizer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let edge = just("o--o").to(Token::Edge);
    let ctrl = one_of("{};.").map(|c| Token::Ctrl(c));
    let newline = choice((
        just("\n").to(Token::Newline),
        just("\r\n").to(Token::Newline),
    ));
    let keyword = choice((
        text::keyword("erd").to(Token::Erd),
        text::keyword("int").to(Token::Int),
        text::keyword("uuid").to(Token::Uuid),
        text::keyword("text").to(Token::Text),
        text::keyword("timestamp").to(Token::Timestamp),
        text::keyword("PK").to(Token::PK),
        text::keyword("FK").to(Token::FK),
    ));

    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('`'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    );

    let ident = text::ident().map(|ident| Token::Ident(ident));

    // `...`
    let quoted_ident = just("`")
        .ignore_then(filter(|c| *c != '\\' && *c != '`').or(escape).repeated())
        .then_ignore(just('`'))
        .collect::<String>()
        .map(Token::Ident);

    // A single token can be one of the above
    let token = edge
        .or(keyword)
        .or(ident)
        .or(quoted_ident)
        .or(ctrl)
        .or(newline)
        // TODO: Choose other recovery mode for better error generation.
        // https://docs.rs/chumsky/latest/chumsky/recovery/fn.skip_then_retry_until.html
        .recover_with(skip_then_retry_until([]));

    let spaces = one_of::<_, _, Simple<char>>(" \t")
        .repeated()
        .collect::<String>();
    let comment = just("//")
        .ignore_then(filter(|c| *c != '\n').repeated())
        .padded_by(spaces.clone())
        .collect::<String>();

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded_by(spaces)
        .repeated()
}

fn erd_module_parser() -> impl Parser<Token, Module, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let separator = choice((just(Token::Newline).to(()), just(Token::Ctrl(';')).to(())));

    let pad = separator.clone().repeated();

    // We want the compiler to check for exclusivity. However, due to the limitations of Rust and the nature of combinator typing, this could not be achieved without introducing code complexity and third-party libraries.
    //
    // - To iterate through the variants of enum, I can use the `strum` crate.
    // - However, since the types do not match, we cannot construct a combinator with looping through variats.
    let entity_field_type = choice((
        just(Token::Int).to(EntityFieldType::Int),
        just(Token::Uuid).to(EntityFieldType::Uuid),
        just(Token::Text).to(EntityFieldType::Text),
        just(Token::Timestamp).to(EntityFieldType::Timestamp),
    ));

    let entity_field_key = choice((
        just(Token::PK).to(EntityFieldKey::PrimaryKey),
        just(Token::FK).to(EntityFieldKey::ForeginKey),
    ));

    let entity = ident
        .then(just(Token::Ctrl('.')).ignore_then(ident.or_not()))
        .map(|(table, field)| {
            if let Some(field) = field {
                EntityPath::Field(table, field)
            } else {
                EntityPath::Entity(table)
            }
        });

    let entity_field = ident
        .then(entity_field_type)
        .then(entity_field_key.or_not())
        .map(|((name, field_type), field_key)| EntityField::new(name, field_type, field_key));

    let entity_fields = entity_field
        .clone()
        .chain(
            separator
                .clone()
                .ignore_then(pad.clone())
                .ignore_then(entity_field.clone())
                .repeated(),
        )
        .or_not()
        .padded_by(pad.clone())
        .map(|fields| fields.unwrap_or_else(|| vec![]));

    let entity_definition = ident
        .then_ignore(pad.clone())
        .then_ignore(just(Token::Ctrl('{')))
        .then(entity_fields)
        .then_ignore(just(Token::Ctrl('}')))
        .map(|(name, fields)| {
            let mut definition = EntityDefinition::new(name);

            for f in fields {
                definition.add_field(f);
            }

            definition
        });

    let relation = entity
        .clone()
        .then(
            just(Token::Edge)
                .padded_by(pad.clone())
                .ignore_then(entity.clone()),
        )
        .map(|(a, b)| EntityRelation::new(a, b));

    let module_entry = choice((
        entity_definition.map(|d| ModuleEntry::EntityDefinition(d)),
        relation.map(|r| ModuleEntry::EntityRelation(r)),
    ));

    let module_entries = module_entry
        .clone()
        .chain(
            separator
                .clone()
                .ignore_then(pad.clone())
                .ignore_then(module_entry.clone())
                .repeated(),
        )
        .or_not()
        .map(|entries| entries.unwrap_or_else(|| vec![]));

    just(Token::Erd)
        .padded_by(pad.clone())
        .ignore_then(ident.padded_by(pad.clone()).or_not())
        .then_ignore(just(Token::Ctrl('{')))
        .then(module_entries.padded_by(pad.clone()))
        .then_ignore(just(Token::Ctrl('}')))
        .padded_by(pad.clone())
        .map(|(name, entries)| {
            let mut module = Module::new(name);

            for entry in entries {
                module.add_entry(entry);
            }

            module
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use difference::assert_diff;

    // This macro expands two strings `src` and `expected` into a code that
    // compare an AST generated from `src` and `expected` string.
    macro_rules! assert_ast {
        ( $src:expr, $expected:expr ) => {
            let (ast, errs, parse_errs) = parse($src);

            assert!(errs.is_empty());
            assert!(parse_errs.is_empty());
            assert!(ast.is_some());
            assert_diff!(&ast.unwrap().to_string(), $expected, "\n", 0);
        };
    }

    #[test]
    fn test_script() {
        assert_ast!(
            "erd G {
users {
    id     int PK
    `uuid` uuid
    `text` text; about_html text
}
users.id o--o posts.created_by
}",
            "erd G {
    users { id int PK; uuid uuid; text text; about_html text }
    users.id o--o posts.created_by
}"
        );
    }
}
