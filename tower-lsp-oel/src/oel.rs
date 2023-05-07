use crate::semantic_token::LEGEND_TYPE;
use chumsky::{prelude::*, stream::Stream};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range, SemanticTokenType};

use std::collections::HashSet;
use std::iter::FromIterator;
use tree_sitter::{Node, Point, Tree};

use tree_sitter_traversal::{traverse, traverse_tree, Order};

/// This is the parser and interpreter for the 'Foo' language. See `tutorial.md` in the repository's root to learn
/// about it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub id: usize,
    pub start: Point,
    pub end: Point,
}

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: Point,
    pub end: Point,
    pub token_type: usize,
}
#[derive(Debug)]
pub struct ErrorToken {
    pub start: Point,
    pub end: Point,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Op(String),
    Ctrl(char),
    Ident(String),
    Fn,
    Let,
    Print,
    If,
    Else,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::Bool(x) => write!(f, "{}", x),
            Token::Num(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "{}", s),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Func(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Func(name) => write!(f, "<function: {}>", name),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
}

pub type Spanned<T> = (T, Range);
// A function node in the AST.

pub fn parse(
    src: &str,
) -> (
    Option<HashMap<String, Span>>,
    Vec<ErrorToken>,
    Vec<ImCompleteSemanticToken>,
) {
    let mut parser = tree_sitter::Parser::new();
    let set_language_result = parser.set_language(tree_sitter_oel::language());
    if set_language_result.is_err() {
        return (None, Vec::new(), Vec::new());
    }

    let tree = parser.parse(src, None);

    return if let Some(tree) = tree {
        let preorder: Vec<Node<'_>> = traverse(tree.walk(), Order::Pre).collect::<Vec<_>>();

        let semantic_tokens = preorder
            .iter()
            .filter_map(|token| {
                // println!("token kind: {}", token.kind());
                match token.kind() {
                    "null" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::KEYWORD)
                            .unwrap(),
                    }),
                    "bool" => None,
                    "arguments" => None,
                    "array" => None,
                    "binary_expression" => None,
                    "boolean" => None,
                    "call_expression" => None,
                    "expression" => None,
                    "member_expression" => None,
                    "nested_identifier" => None,
                    "parenthesized_expression" => None,
                    "primary_expression" => None,
                    "primitive" => None,
                    "source_file" => None,
                    "string" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::STRING)
                            .unwrap(),
                    }),
                    "subscript_expression" => None,
                    "ternary_expression" => None,
                    "unary_expression" => None,
                    "!" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "!=" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "\"" => None,
                    "'" => None,
                    "(" => None,
                    ")" => None,
                    "+" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "," => None,
                    "." => None,
                    ":" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "<" => None,
                    "<=" => None,
                    "==" => None,
                    ">" => None,
                    ">=" => None,
                    "?" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "AND" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "OR" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::OPERATOR)
                            .unwrap(),
                    }),
                    "[" => None,
                    "]" => None,
                    "false" => None,
                    "float" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::NUMBER)
                            .unwrap(),
                    }),
                    "identifier" => None,
                    "integer" => Some(ImCompleteSemanticToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        token_type: LEGEND_TYPE
                            .iter()
                            .position(|item| item == &SemanticTokenType::NUMBER)
                            .unwrap(),
                    }),
                    "property_identifier" => None,
                    "string_fragment" => None,
                    "true" => None,
                    "{" => None,
                    "}" => None,
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        let parse_errs = preorder
            .iter()
            .filter_map(|token| {
                // has_error indicates that children have errors,
                // we only want leaf errors to avoid duplicate messages
                if token.is_error() && !token.has_error() {
                    Some(ErrorToken {
                        start: token.start_position(),
                        end: token.end_position(),
                        message: String::from("Parse error"),
                    })
                } else {
                    None
                }
            })
            .collect();

        // println!("{:#?}", ast);
        // if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
        //     if let Some(main) = funcs.get("main") {
        //         assert_eq!(main.args.len(), 0);
        //         match eval_expr(&main.body, &funcs, &mut Vec::new()) {
        //             Ok(val) => println!("Return value: {}", val),
        //             Err(e) => errs.push(Simple::custom(e.span, e.msg)),
        //         }
        //     } else {
        //         panic!("No main function!");
        //     }
        // }

        let ast_mapped: HashMap<_, _> = preorder
            .iter()
            .map(|&node| {
                (
                    node.id().to_string(),
                    Span {
                        // TODO: No unwrap
                        id: node.id(),
                        start: node.start_position(),
                        end: node.end_position(),
                    },
                )
            })
            .collect();

        (Some(ast_mapped), parse_errs, semantic_tokens)
    } else {
        (None, Vec::new(), Vec::new())
    };
}
