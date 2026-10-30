// Derived from Starscript 0.2.5 (MIT); see LICENSE.
use super::Error;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
pub type Function = Rc<dyn Fn(&[Value]) -> Result<Value, Error>>;
pub type Supplier = Rc<dyn Fn() -> Value>;
#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Function(Function),
    Map(ValueMap),
    Object { identity: u64, text: String },
}
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display().unwrap_or_else(|_| "<invalid map>".into()))
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Function(a), Self::Function(b)) => Rc::ptr_eq(a, b),
            (Self::Map(a), Self::Map(b)) => Rc::ptr_eq(&a.0, &b.0),
            (Self::Object { identity: a, .. }, Self::Object { identity: b, .. }) => a == b,
            _ => false,
        }
    }
}
impl Value {
    pub fn truthy(&self) -> bool {
        !matches!(self, Self::Null | Self::Bool(false))
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Bool(_) => "Boolean",
            Self::Number(_) => "Number",
            Self::String(_) => "String",
            Self::Function(_) => "Function",
            Self::Map(_) => "Map",
            Self::Object { .. } => "Object",
        }
    }
    pub fn number(&self, error: &str) -> Result<f64, Error> {
        if let Self::Number(n) = self {
            Ok(*n)
        } else {
            Err(Error(error.into()))
        }
    }
    pub fn string(&self, error: &str) -> Result<&str, Error> {
        if let Self::String(s) = self {
            Ok(s)
        } else {
            Err(Error(error.into()))
        }
    }
    pub fn display(&self) -> Result<String, Error> {
        self.display_depth(0)
    }
    fn display_depth(&self, depth: usize) -> Result<String, Error> {
        if depth > 128 {
            return Err(Error("Recursive map _toString limit exceeded.".into()));
        }
        Ok(match self {
            Self::Null => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => {
                if n % 1.0 == 0.0 {
                    (*n as i32).to_string()
                } else {
                    double_string(*n)
                }
            }
            Self::String(s) => s.clone(),
            Self::Function(_) => "<function>".into(),
            Self::Object { text, .. } => text.clone(),
            Self::Map(m) => {
                if let Some(s) = m.supplier("_toString") {
                    s().display_depth(depth + 1)?
                } else {
                    "<map>".into()
                }
            }
        })
    }
}
/// Java Double.toString presentation (including .0 and exponent thresholds).
/// Rare shortest-decimal tie differences from Java are tracked by differential tests.
pub(super) fn double_string(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n == f64::INFINITY {
        return "Infinity".into();
    }
    if n == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if n == 0.0 {
        return if n.is_sign_negative() { "-0.0" } else { "0.0" }.into();
    }
    if n.abs() >= 1e7 || n.abs() < 1e-3 {
        let s = format!("{n:e}");
        let (mantissa, exponent) = s.split_once('e').unwrap();
        format!(
            "{}{}E{}",
            mantissa,
            if mantissa.contains('.') { "" } else { ".0" },
            exponent.parse::<i32>().unwrap()
        )
    } else {
        let s = n.to_string();
        if s.contains('.') { s } else { format!("{s}.0") }
    }
}
#[derive(Clone, Default)]
pub struct ValueMap(Rc<RefCell<BTreeMap<String, Supplier>>>);
impl ValueMap {
    pub fn supplier(&self, name: &str) -> Option<Supplier> {
        self.0.borrow().get(name).cloned()
    }
    pub fn get_raw(&self, name: &str) -> Value {
        self.supplier(name).map(|f| f()).unwrap_or(Value::Null)
    }
    pub fn get(&self, name: &str) -> Value {
        if let Some((head, tail)) = name.split_once('.') {
            if let Value::Map(m) = self.get_raw(head) {
                m.get(tail)
            } else {
                Value::Null
            }
        } else {
            self.get_raw(name)
        }
    }
    pub fn set_raw(&self, name: &str, value: Supplier) -> Result<(), Error> {
        if matches!(name, "null" | "true" | "false" | "and" | "or") {
            return Err(Error("Variable name cannot be a keyword.".into()));
        }
        self.0.borrow_mut().insert(name.into(), value);
        Ok(())
    }
    pub fn set_supplier(&self, name: &str, value: Supplier) -> Result<(), Error> {
        if let Some((head, tail)) = name.split_once('.') {
            let map = if let Value::Map(m) = self.get_raw(head) {
                m
            } else {
                let m = Self::default();
                self.set(head, Value::Map(m.clone()))?;
                m
            };
            map.set_supplier(tail, value)
        } else {
            self.set_raw(name, value)
        }
    }
    pub fn set(&self, name: &str, value: Value) -> Result<(), Error> {
        self.set_supplier(name, Rc::new(move || value.clone()))
    }
    pub fn keys(&self) -> Vec<String> {
        self.0.borrow().keys().cloned().collect()
    }
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }
    pub fn remove(&self, name: &str) -> Option<Supplier> {
        if let Some((head, tail)) = name.split_once('.') {
            if let Value::Map(m) = self.get_raw(head) {
                m.remove(tail)
            } else {
                self.0.borrow_mut().remove(head)
            }
        } else {
            self.0.borrow_mut().remove(name)
        }
    }
}
