//! Native Starscript 0.2.5 implementation, derived from MeteorDevelopment/starscript.
//! Upstream is MIT licensed; see LICENSE in this directory. No Java runtime required.
//! Language precedence intentionally follows upstream, including `or` binding before `and`.
//!
//! Provenance: official org.meteordev:starscript:0.2.5 source JAR, SHA-256
//! 50ebd9bfc6f93b1cac1494b80224bba9f0fdb8b88e8e0eb990777836464b5b04.
//! The checked-in reference corpus is generated with Java Locale.ROOT and seed 123.
//! Compatibility boundaries: host supplies local time/date and world variables;
//! case conversion uses Rust Unicode rather than Java's locale-sensitive casing;
//! error digits are ASCII. Rare Java shortest-double decimal ties can differ.
//! Lone UTF-16 surrogates are rejected instead of being silently corrupted.
//! Opaque host object equality uses adapter-provided identity, not Java Object.equals.
//! Diagnostics report UTF-16 offsets and the first error; completion/semantic tokens
//! and Java bytecode overflow/optimizer stack corruption are not emulated.
//! Source/output/nesting limits return errors rather than crashing the game.
pub mod meteor;
mod parser;
mod standard;
mod value;

use parser::Expr;
pub use standard::JavaRandom;
pub use value::{Function, Supplier, Value, ValueMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub offset: usize,
    pub message: String,
}
#[derive(Clone, Debug)]
pub struct Script {
    expressions: Vec<Expr>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub index: i8,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub sections: Vec<Section>,
}
impl Output {
    pub fn text(&self) -> String {
        self.sections.iter().map(|s| s.text.as_str()).collect()
    }
}

/// Compile once and run repeatedly against live suppliers. Source is bounded to prevent
/// an imported macro from exhausting the native stack; rejected code never executes.
pub fn compile(source: &str) -> Result<Script, Vec<Diagnostic>> {
    parser::parse(source)
}
/// Double text used by host Vec3 wrappers (not Value.number's integer-clamping display).
pub fn java_double_string(number: f64) -> String {
    value::double_string(number)
}

#[derive(Default, Clone)]
pub struct Environment {
    pub globals: ValueMap,
}
impl Environment {
    /// Clock suppliers are explicit: the game supplies local time/date, rather than
    /// silently substituting UTC for Java's local timezone.
    pub fn standard(seed: u64, time: Supplier, date: Supplier) -> Self {
        let env = Self::default();
        standard::install(&env.globals, seed, time, date);
        env
    }
    pub fn run(&self, script: &Script) -> Result<Output, Error> {
        let mut runner = Runner {
            env: self,
            sections: vec![],
            index: 0,
            text: String::new(),
            steps: 0,
            depth: 0,
            output_bytes: 0,
        };
        for expr in &script.expressions {
            runner.top(expr)?;
        }
        runner.flush();
        Ok(Output {
            sections: runner.sections,
        })
    }
}
struct Runner<'a> {
    env: &'a Environment,
    sections: Vec<Section>,
    index: i8,
    text: String,
    steps: usize,
    depth: usize,
    output_bytes: usize,
}
impl Runner<'_> {
    fn flush(&mut self) {
        self.sections.push(Section {
            index: self.index,
            text: std::mem::take(&mut self.text),
        });
    }
    fn append(&mut self, value: Value) -> Result<(), Error> {
        let text = value.display()?;
        if self.output_bytes + text.len() > 1_048_576 {
            return Err(Error("Starscript output limit exceeded.".into()));
        }
        self.output_bytes += text.len();
        self.text.push_str(&text);
        Ok(())
    }
    fn top(&mut self, expr: &Expr) -> Result<(), Error> {
        match expr {
            Expr::Section(index, inner) => {
                self.flush();
                self.index = *index;
                self.top(inner)
            }
            Expr::Literal(Value::String(s)) => self.append(Value::String(s.clone())),
            Expr::Block(inner) => {
                // Preserve upstream VariableGetAppend's non-map behavior (empty output).
                if let Expr::Get(object, field) = &**inner {
                    if let Expr::Variable(name) = &**object {
                        if let Value::Map(map) = self.env.globals.get_raw(name) {
                            return self.append(map.get_raw(field));
                        }
                        return Ok(());
                    }
                }
                let v = self.eval(inner)?;
                self.append(v)
            }
            _ => {
                let _ = self.eval(expr)?;
                Ok(())
            }
        }
    }
    fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.depth += 1;
        if self.depth > 128 {
            return Err(Error(
                "Starscript evaluation nesting limit exceeded.".into(),
            ));
        }
        let result = self.eval_inner(expr);
        self.depth -= 1;
        result
    }
    fn eval_inner(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.steps += 1;
        if self.steps > 100_000 {
            return Err(Error("Starscript execution limit exceeded.".into()));
        }
        match expr {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Variable(n) => Ok(self.env.globals.get_raw(n)),
            Expr::Group(e) | Expr::Block(e) => self.eval(e),
            Expr::Section(index, e) => {
                self.flush();
                self.index = *index;
                self.eval(e)
            }
            Expr::Get(e, n) => Ok(match self.eval(e)? {
                Value::Map(m) => m.get_raw(n),
                _ => Value::Null,
            }),
            Expr::Call(e, args) => {
                let callee = self.eval(e)?;
                let values = args
                    .iter()
                    .map(|e| self.eval(e))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee {
                    Value::Function(f) => f(&values),
                    other => Err(Error(format!(
                        "Tried to call a {}, can only call functions.",
                        other.type_name()
                    ))),
                }
            }
            Expr::Conditional(c, t, f) => {
                if self.eval(c)?.truthy() {
                    self.eval(t)
                } else {
                    self.eval(f)
                }
            }
            Expr::Unary(op, e) => {
                let v = self.eval(e)?;
                match op.as_str() {
                    "!" => Ok(Value::Bool(!v.truthy())),
                    "-" => Ok(Value::Number(
                        -v.number("This operation requires a number.")?,
                    )),
                    "~" => Ok(Value::Number(
                        !(v.number("This operation requires a number.")? as i64) as f64,
                    )),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(a, op, b) => {
                let a = self.eval(a)?;
                if op == "and" {
                    return if a.truthy() { self.eval(b) } else { Ok(a) };
                }
                if op == "or" {
                    return if a.truthy() { Ok(a) } else { self.eval(b) };
                }
                let b = self.eval(b)?;
                if op == "==" {
                    return Ok(Value::Bool(a == b));
                }
                if op == "!=" {
                    return Ok(Value::Bool(a != b));
                }
                if op == "+" {
                    return match (&a, &b) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::String(a), _) => {
                            let b = b.display()?;
                            if a.len() + b.len() > 1_048_576 {
                                return Err(Error("Starscript output limit exceeded.".into()));
                            }
                            Ok(Value::String(format!("{a}{b}")))
                        }
                        _ => Err(Error(
                            "Can only add 2 numbers or 1 string and other value.".into(),
                        )),
                    };
                }
                let error = match op.as_str() {
                    "-" => "Can only subtract 2 numbers.",
                    "*" => "Can only multiply 2 numbers.",
                    "/" => "Can only divide 2 numbers.",
                    "%" => "Can only modulo 2 numbers.",
                    "^" => "Can only power 2 numbers.",
                    _ => "This operation requires 2 numbers.",
                };
                let (x, y) = (a.number(error)?, b.number(error)?);
                Ok(match op.as_str() {
                    "-" => Value::Number(x - y),
                    "*" => Value::Number(x * y),
                    "/" => Value::Number(x / y),
                    "%" => Value::Number(x % y),
                    "^" => Value::Number(
                        if y != 0.0 && (y.is_nan() || (x.abs() == 1.0 && y.is_infinite())) {
                            f64::NAN
                        } else {
                            x.powf(y)
                        },
                    ),
                    ">" => Value::Bool(x > y),
                    ">=" => Value::Bool(x >= y),
                    "<" => Value::Bool(x < y),
                    "<=" => Value::Bool(x <= y),
                    "&" => Value::Number(((x as i64) & (y as i64)) as f64),
                    "|" => Value::Number(((x as i64) | (y as i64)) as f64),
                    "^^" => Value::Number(((x as i64) ^ (y as i64)) as f64),
                    "<<" => Value::Number((x as i64).wrapping_shl((y as i64 & 63) as u32) as f64),
                    ">>" => Value::Number(((x as i64) >> ((y as i64 & 63) as u32)) as f64),
                    ">>>" => {
                        Value::Number(((x as i64 as u64) >> ((y as i64 & 63) as u32)) as i64 as f64)
                    }
                    _ => unreachable!(),
                })
            }
        }
    }
}
