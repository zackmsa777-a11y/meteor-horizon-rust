// Derived from Starscript 0.2.5 (MIT); see LICENSE.
use super::{Diagnostic, Script, Value};
#[derive(Clone, Debug)]
pub(super) enum Expr {
    Literal(Value),
    Variable(String),
    Group(Box<Expr>),
    Block(Box<Expr>),
    Section(i8, Box<Expr>),
    Get(Box<Expr>, String),
    Call(Box<Expr>, Vec<Expr>),
    Unary(String, Box<Expr>),
    Binary(Box<Expr>, String, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}
#[derive(Clone, Debug)]
struct Token {
    kind: String,
    text: String,
    offset: usize,
}
struct Lexer {
    chars: Vec<char>,
    i: usize,
    depth: usize,
}
impl Lexer {
    fn peek(&self, n: usize) -> char {
        self.chars.get(self.i + n).copied().unwrap_or('\0')
    }
    fn take(&mut self) -> char {
        let c = self.peek(0);
        self.i += 1;
        c
    }
    fn token(&self, kind: &str, text: String, start: usize) -> Token {
        Token {
            kind: kind.into(),
            text,
            offset: self.chars[..start].iter().map(|c| c.len_utf16()).sum(),
        }
    }
    fn next(&mut self) -> Token {
        if self.depth > 0 {
            while matches!(self.peek(0), ' ' | '\r' | '\n' | '\t') {
                self.i += 1;
            }
        }
        let start = self.i;
        if self.i >= self.chars.len() {
            return self.token("eof", String::new(), start);
        }
        let c = self.take();
        if self.depth == 0 {
            if c == '{' && self.peek(0) != '{' {
                self.depth += 1;
                return self.token("{", "{".into(), start);
            }
            if c == '#' && self.peek(0).is_ascii_digit() {
                while self.peek(0).is_ascii_digit() {
                    self.i += 1;
                }
                return self.token(
                    "section",
                    self.chars[start + 1..self.i].iter().collect(),
                    start,
                );
            }
            while self.i < self.chars.len()
                && !(self.peek(0) == '{' && self.peek(1) != '{')
                && !(self.peek(0) == '#' && self.peek(1).is_ascii_digit())
            {
                let c = self.take();
                if (c == '{' && self.peek(0) == '{') || (c == '#' && self.peek(0) == '#') {
                    self.i += 1;
                }
            }
            return self.token("string", self.chars[start..self.i].iter().collect(), start);
        }
        if c.is_ascii_digit() || (c == '-' && self.peek(0).is_ascii_digit()) {
            while self.peek(0).is_ascii_digit() {
                self.i += 1;
            }
            if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
                self.i += 1;
                while self.peek(0).is_ascii_digit() {
                    self.i += 1;
                }
            }
            return self.token("number", self.chars[start..self.i].iter().collect(), start);
        }
        if c.is_ascii_alphabetic() || c == '_' {
            while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == '_' {
                self.i += 1;
            }
            let text: String = self.chars[start..self.i].iter().collect();
            return self.token(
                if matches!(text.as_str(), "null" | "true" | "false" | "and" | "or") {
                    &text
                } else {
                    "identifier"
                },
                text.clone(),
                start,
            );
        }
        if c == '\'' || c == '"' {
            let mut text = String::new();
            while self.i < self.chars.len() && self.peek(0) != c {
                let mut a = self.take();
                if a == '\\' && self.i < self.chars.len() {
                    a = self.take();
                }
                text.push(a);
            }
            if self.i == self.chars.len() {
                return self.token("error", "Unterminated expression.".into(), start);
            }
            self.i += 1;
            return self.token("string", text, start);
        }
        if c == '#' {
            while self.peek(0).is_ascii_digit() {
                self.i += 1;
            }
            return self.token(
                "section",
                self.chars[start + 1..self.i].iter().collect(),
                start,
            );
        }
        if c == '{' {
            self.depth += 1;
        }
        if c == '}' {
            self.depth -= 1;
        }
        let mut op = c.to_string();
        if matches!(c, '=' | '!' | '<' | '>') && self.peek(0) == '=' {
            op.push(self.take());
        } else if matches!(c, '<' | '>' | '^') && self.peek(0) == c {
            op.push(self.take());
            if c == '>' && self.peek(0) == '>' {
                op.push(self.take());
            }
        }
        if op == "=" || !"=!<>+-*/%^&|~.,?:(){}".contains(c) {
            self.token("error", "Unexpected character.".into(), start)
        } else {
            self.token(&op, op.clone(), start)
        }
    }
}
struct Parser {
    lexer: Lexer,
    current: Token,
    blocks: usize,
    recursion: usize,
}
pub(super) fn parse(source: &str) -> Result<Script, Vec<Diagnostic>> {
    if source.len() > 65_536 {
        return Err(vec![Diagnostic {
            offset: 0,
            message: "Starscript source limit exceeded.".into(),
        }]);
    }
    let mut lexer = Lexer {
        chars: source.chars().collect(),
        i: 0,
        depth: 0,
    };
    let current = lexer.next();
    let mut p = Parser {
        lexer,
        current,
        blocks: 0,
        recursion: 0,
    };
    let mut expressions = vec![];
    while p.current.kind != "eof" {
        match p.statement() {
            Ok(e) => expressions.push(e),
            Err(e) => return Err(vec![e]),
        }
    }
    Ok(Script { expressions })
}
impl Parser {
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic {
            offset: self.current.offset,
            message: message.into(),
        }
    }
    fn advance(&mut self) -> Token {
        let next = self.lexer.next();
        std::mem::replace(&mut self.current, next)
    }
    fn accept(&mut self, kind: &str) -> bool {
        if self.current.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, kind: &str, message: &str) -> Result<(), Diagnostic> {
        if self.accept(kind) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }
    fn statement(&mut self) -> Result<Expr, Diagnostic> {
        if self.current.kind == "section" {
            let t = self.advance();
            let index = t
                .text
                .parse::<u16>()
                .map_err(|_| self.error("Expected section index."))?;
            if index > 255 {
                return Err(self.error("Section index cannot be larger than 255."));
            }
            return Ok(Expr::Section(
                index as u8 as i8,
                Box::new(self.expression(0)?),
            ));
        }
        self.expression(0)
    }
    fn expression(&mut self, min: u8) -> Result<Expr, Diagnostic> {
        self.recursion += 1;
        if self.recursion > 128 {
            return Err(self.error("Starscript nesting limit exceeded."));
        }
        let result = self.expression_inner(min);
        self.recursion -= 1;
        result
    }
    fn expression_inner(&mut self, min: u8) -> Result<Expr, Diagnostic> {
        let mut left = if matches!(self.current.kind.as_str(), "!" | "-" | "~") {
            let op = self.advance().kind;
            Expr::Unary(op, Box::new(self.expression(9)?))
        } else {
            self.primary()?
        };
        let mut chain = 0;
        loop {
            chain += 1;
            if chain > 128 {
                return Err(self.error("Starscript expression chain limit exceeded."));
            }
            if min <= 10 && self.accept("(") {
                let mut args = vec![];
                if self.current.kind != ")" {
                    loop {
                        args.push(self.expression(0)?);
                        if !self.accept(",") {
                            break;
                        }
                        if args.len() >= 127 {
                            return Err(self.error("Starscript argument limit exceeded."));
                        }
                    }
                }
                self.expect(")", "Expected ')' after function arguments.")?;
                left = Expr::Call(Box::new(left), args);
                continue;
            }
            if min <= 10 && self.accept(".") {
                if self.current.kind != "identifier" {
                    return Err(self.error("Expected field name after '.'."));
                }
                let name = self.advance().text;
                left = Expr::Get(Box::new(left), name);
                continue;
            }
            let precedence = match self.current.kind.as_str() {
                "and" => 1,
                "or" => 2,
                "==" | "!=" => 3,
                ">" | ">=" | "<" | "<=" => 4,
                "+" | "-" => 5,
                "*" | "/" | "%" | "^" => 6,
                "&" | "|" | "^^" | "<<" | ">>" | ">>>" => 7,
                _ => 0,
            };
            if precedence == 0 || precedence < min {
                break;
            }
            let op = self.advance().kind;
            let right = self.expression(precedence + 1)?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        if min == 0 && self.accept("?") {
            let t = self.statement()?;
            self.expect(":", "Expected ':' after first part of condition.")?;
            let f = self.statement()?;
            left = Expr::Conditional(Box::new(left), Box::new(t), Box::new(f));
        }
        Ok(left)
    }
    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        match self.current.kind.as_str() {
            "null" => {
                self.advance();
                Ok(Expr::Literal(Value::Null))
            }
            "true" | "false" => {
                let t = self.advance();
                Ok(Expr::Literal(Value::Bool(t.kind == "true")))
            }
            "number" => {
                let t = self.advance();
                Ok(Expr::Literal(Value::Number(
                    t.text.parse().map_err(|_| self.error("Invalid number."))?,
                )))
            }
            "string" => Ok(Expr::Literal(Value::String(self.advance().text))),
            "identifier" => Ok(Expr::Variable(self.advance().text)),
            "(" => {
                self.advance();
                let e = self.statement()?;
                self.expect(")", "Expected ')' after expression.")?;
                Ok(Expr::Group(Box::new(e)))
            }
            "{" => {
                self.advance();
                let outer = self.blocks == 0;
                self.blocks += 1;
                let e = self.statement()?;
                self.expect("}", "Expected '}' after expression.")?;
                self.blocks -= 1;
                Ok(if outer { Expr::Block(Box::new(e)) } else { e })
            }
            _ => Err(self.error("Expected expression.")),
        }
    }
}
