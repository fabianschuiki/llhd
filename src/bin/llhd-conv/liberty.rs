// Copyright (c) 2017-2021 Fabian Schuiki

//! Lexer and parser for Liberty files.

use llhd::{int_ty, ir::prelude::*, signal_ty};
use std::collections::HashMap;

/// A lexer for Liberty files.
pub struct Lexer<I> {
    input: I,
    peek: [Option<u8>; 2],
    done: bool,
    offset: usize,
    line: usize,
    column: usize,
}

/// The token emitted by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Literal(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,
}

impl<I: Iterator<Item = std::io::Result<u8>>> Lexer<I> {
    /// Create a new lexer.
    pub fn new(input: I) -> Self {
        let mut lexer = Self {
            input,
            peek: [None, None],
            done: false,
            offset: 0,
            line: 0,
            column: 0,
        };
        lexer.bump();
        lexer.bump();
        lexer.offset = 0;
        lexer
    }

    /// Advance the lexer to the next character.
    fn bump(&mut self) {
        self.offset += 1;
        if self.peek[0] == Some('\n' as u8) {
            self.line += 1;
            self.column = 0;
        }
        self.peek[0] = self.peek[1];
        if !self.done {
            self.peek[1] = self.input.next().map(|b| b.unwrap());
            if self.peek[1].is_none() {
                self.done = true;
            }
        }
    }
}

fn should_skip(c: char) -> bool {
    c.is_whitespace() || c == '\\'
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '+'
}

impl<I: Iterator<Item = std::io::Result<u8>>> Iterator for Lexer<I> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        loop {
            let p0 = self.peek[0].map(|c| c as char);
            let p1 = self.peek[1].map(|c| c as char);
            match (p0, p1) {
                // Skip multi-line comments.
                (Some('/'), Some('*')) => {
                    self.bump();
                    self.bump();
                    while self.peek[0].is_some()
                        && (self.peek[0] != Some('*' as u8) || self.peek[1] != Some('/' as u8))
                    {
                        self.bump();
                    }
                    self.bump();
                    self.bump();
                    continue;
                }
                // Skip single-line comments.
                (Some('/'), Some('/')) => {
                    self.bump();
                    self.bump();
                    while self.peek[0].is_some() && self.peek[0] != Some('\n' as u8) {
                        self.bump();
                    }
                    continue;
                }
                // Skip whitespace and escapes.
                (Some(c), _) if should_skip(c) => {
                    self.bump();
                    continue;
                }
                // Parse symbols.
                (Some('('), _) => {
                    self.bump();
                    return Some(Token::LParen);
                }
                (Some(')'), _) => {
                    self.bump();
                    return Some(Token::RParen);
                }
                (Some('{'), _) => {
                    self.bump();
                    return Some(Token::LBrace);
                }
                (Some('}'), _) => {
                    self.bump();
                    return Some(Token::RBrace);
                }
                (Some(','), _) => {
                    self.bump();
                    return Some(Token::Comma);
                }
                (Some(':'), _) => {
                    self.bump();
                    return Some(Token::Colon);
                }
                (Some(';'), _) => {
                    self.bump();
                    return Some(Token::Semicolon);
                }
                // Identifiers and numbers.
                (Some(c), _) if is_ident(c) => {
                    let mut v = vec![self.peek[0].unwrap()];
                    self.bump();
                    while let Some(c) = self.peek[0] {
                        if !is_ident(c as char) {
                            break;
                        }
                        v.push(c);
                        self.bump();
                    }
                    let v = match String::from_utf8(v) {
                        Ok(v) => v,
                        Err(e) => panic!(
                            "syntax error: line {} column {} (offset {}): invalid UTF-8 string; {}",
                            self.line + 1,
                            self.column + 1,
                            self.offset,
                            e
                        ),
                    };
                    return Some(Token::Ident(v));
                }
                // Literals.
                (Some('"'), _) => {
                    self.bump();
                    let mut v = vec![];
                    while let Some(c) = self.peek[0] {
                        if self.peek[0] == Some('"' as u8) {
                            break;
                        }
                        if self.peek[0] == Some('\\' as u8) {
                            self.bump();
                        }
                        v.push(c);
                        self.bump();
                    }
                    self.bump();
                    let v = match String::from_utf8(v) {
                        Ok(v) => v,
                        Err(e) => panic!(
                            "syntax error: line {} column {} (offset {}): invalid UTF-8 string; {}",
                            self.line + 1,
                            self.column + 1,
                            self.offset,
                            e
                        ),
                    };
                    return Some(Token::Literal(v));
                }
                // End of file.
                (None, _) => return None,
                (Some(c), _) => panic!(
                    "syntax error: line {} column {} (offset {}): unexpected \"{}\"",
                    self.line + 1,
                    self.column + 1,
                    self.offset,
                    c
                ),
            }
        }
    }
}

/// A visitor for a Liberty file.
pub trait Visitor {
    /// Called for `name : value;` fields.
    fn visit_scalar(&mut self, _name: String, _value: String) {}
    /// Called for `name(values);` fields.
    fn visit_array(&mut self, _name: String, _values: Vec<String>) {}
    /// Called for `name(values) { ... }` fields.
    fn visit_group_begin(&mut self, _name: String, _values: Vec<String>) {}
    /// Called for end of groups.
    fn visit_group_end(&mut self) {}
}

/// Parse a entire Liberty file.
pub fn parse(p: &mut impl Iterator<Item = Token>, with: &mut impl Visitor) {
    loop {
        let name = match p.next() {
            Some(Token::Ident(ident)) => ident,
            Some(Token::RBrace) | None => return,
            _ => panic!("syntax error: expected field name"),
        };
        match p.next() {
            Some(Token::Colon) => {
                let value = match p.next() {
                    Some(Token::Ident(ident)) => ident,
                    Some(Token::Literal(literal)) => literal,
                    _ => panic!("syntax error: expected field value after \":\""),
                };
                match p.next() {
                    Some(Token::Semicolon) => (),
                    _ => panic!("syntax error: expected \";\" after field value"),
                }
                with.visit_scalar(name, value);
            }
            Some(Token::LParen) => {
                let mut values = vec![];
                loop {
                    match p.next() {
                        Some(Token::Ident(ident)) => values.push(ident),
                        Some(Token::Literal(literal)) => values.push(literal),
                        Some(Token::Comma) => (),
                        Some(Token::RParen) => break,
                        _ => panic!("syntax error: expected value or \")\""),
                    }
                }

                match p.next() {
                    Some(Token::Semicolon) => with.visit_array(name, values),
                    Some(Token::LBrace) => {
                        with.visit_group_begin(name, values);
                        parse(p, with);
                        with.visit_group_end();
                    }
                    _ => panic!("syntax error: expected \";\" or \"{\""),
                }
            }
            _ => panic!("syntax error: expected \":\" or \"(\""),
        }
    }
}

/// The root visitor.
pub struct RootVisitor<'a> {
    module: &'a mut Module,
    stack: Vec<Context>,
    cell_name: Option<String>,
    cell_inputs: Vec<String>,
    cell_outputs: Vec<(String, String)>,
    pin_name: Option<String>,
    pin_function: Option<String>,
    pin_direction: Option<String>,
}

enum Context {
    None,
    Cell,
    Pin,
}

impl Visitor for RootVisitor<'_> {
    fn visit_scalar(&mut self, name: String, value: String) {
        match self.stack.last() {
            Some(Context::Pin) if name == "function" => self.pin_function = Some(value),
            Some(Context::Pin) if name == "direction" => self.pin_direction = Some(value),
            _ => (),
        }
    }

    fn visit_group_begin(&mut self, name: String, mut values: Vec<String>) {
        let context = match (name.as_str(), values.pop()) {
            ("cell", Some(value)) => {
                self.cell_name = Some(value);
                Context::Cell
            }
            ("pin", Some(value)) => {
                self.pin_name = Some(value);
                self.pin_function = None;
                self.pin_direction = None;
                Context::Pin
            }
            _ => Context::None,
        };
        self.stack.push(context);
    }

    fn visit_group_end(&mut self) {
        match self.stack.pop().expect("unbalanced LIB file") {
            Context::Cell => self.emit_cell(),
            Context::Pin => {
                let dir = self.pin_direction.take();
                let name = self.pin_name.take();
                let func = self.pin_function.take();
                match (dir.as_ref().map(AsRef::as_ref), name, func) {
                    (Some("input"), Some(name), _) => self.cell_inputs.push(name),
                    (Some("output"), Some(name), Some(func)) => {
                        self.cell_outputs.push((name, func))
                    }
                    _ => (),
                }
            }
            Context::None => (),
        }
    }
}

impl<'a> RootVisitor<'a> {
    /// Create a new visitor.
    pub fn new(module: &'a mut Module) -> Self {
        Self {
            module,
            stack: Default::default(),
            cell_name: Default::default(),
            cell_inputs: Default::default(),
            cell_outputs: Default::default(),
            pin_name: Default::default(),
            pin_function: Default::default(),
            pin_direction: Default::default(),
        }
    }

    fn emit_cell(&mut self) {
        let cell_name = match self.cell_name.take() {
            Some(name) => UnitName::Global(name),
            None => return,
        };
        let mut sig = Signature::new();
        let mut input_map = HashMap::new();
        for name in self.cell_inputs.drain(..) {
            let arg = sig.add_input(signal_ty(int_ty(1)));
            input_map.insert(name, arg);
        }
        let mut output_map = HashMap::new();
        let mut funcs = vec![];
        for (name, func) in self.cell_outputs.drain(..) {
            let arg = sig.add_output(signal_ty(int_ty(1)));
            let func = match FunctionParser::new().parse(&func) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        "{}: invalid function `{}` on pin `{}`; {}",
                        cell_name, func, name, e
                    );
                    return;
                }
            };
            funcs.push((arg, func));
            output_map.insert(name, arg);
        }
        let mut ent = UnitData::new(UnitKind::Entity, cell_name, sig);
        let mut builder = UnitBuilder::new_anonymous(&mut ent);
        for (name, &arg) in input_map.iter().chain(output_map.iter()) {
            let arg = builder.arg_value(arg);
            builder.set_name(arg, name.clone());
        }
        for (arg, func) in funcs {
            let arg = builder.arg_value(arg);
            let value = match self.emit_term(&mut builder, &input_map, func) {
                Ok(v) => v,
                Err(e) => {
                    let unit = builder.finish();
                    eprintln!(
                        "{}: invalid function on `{}`; {}",
                        unit.name(),
                        unit.get_name(arg)
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{}", arg)),
                        e
                    );
                    return;
                }
            };
            builder.ins().con(arg, value);
        }
        self.module.add_unit(ent);
    }

    fn emit_term(
        &mut self,
        builder: &mut UnitBuilder,
        map: &HashMap<String, Arg>,
        func: FunctionTerm,
    ) -> Result<Value, String> {
        Ok(match func {
            FunctionTerm::Or(lhs, rhs) => {
                let x = self.emit_term(builder, map, *lhs)?;
                let y = self.emit_term(builder, map, *rhs)?;
                builder.ins().or(x, y)
            }
            FunctionTerm::And(lhs, rhs) => {
                let x = self.emit_term(builder, map, *lhs)?;
                let y = self.emit_term(builder, map, *rhs)?;
                builder.ins().and(x, y)
            }
            FunctionTerm::Not(term) => {
                let x = self.emit_term(builder, map, *term)?;
                builder.ins().not(x)
            }
            FunctionTerm::Atom(name) => {
                let arg = map.get(&name).cloned().ok_or_else(|| {
                    format!("term references argument `{}` which is not a pin", name)
                })?;
                builder.arg_value(arg)
            }
        })
    }
}

#[derive(Debug)]
pub enum FunctionTerm {
    Or(Box<FunctionTerm>, Box<FunctionTerm>),
    And(Box<FunctionTerm>, Box<FunctionTerm>),
    Not(Box<FunctionTerm>),
    Atom(String),
}

#[allow(unused_parens)]
mod grammar {
    include!("liberty_parser.rs");
}

use grammar::FunctionParser;
