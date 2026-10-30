// Derived from Starscript 0.2.5 StandardLib (MIT); see LICENSE.
use super::{Error, Supplier, Value, ValueMap};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
pub struct JavaRandom {
    seed: u64,
}
impl JavaRandom {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ 0x5deece66d) & ((1 << 48) - 1),
        }
    }
    fn bits(&mut self, n: u32) -> u64 {
        self.seed = (self.seed.wrapping_mul(0x5deece66d).wrapping_add(11)) & ((1 << 48) - 1);
        self.seed >> (48 - n)
    }
    pub fn next_double(&mut self) -> f64 {
        ((self.bits(26) << 27) + self.bits(27)) as f64 / ((1u64 << 53) as f64)
    }
}
fn arity(name: &str, args: &[Value], n: usize) -> Result<(), Error> {
    if args.len() == n {
        Ok(())
    } else {
        Err(Error(format!(
            "{name}() requires {n} argument{}, got {}.",
            if n == 1 { "" } else { "s" },
            args.len()
        )))
    }
}
fn number(args: &[Value], index: usize, name: &str, position: &str) -> Result<f64, Error> {
    args[index].number(&format!("{position} to {name}() needs to be a number."))
}
fn string<'a>(
    args: &'a [Value],
    index: usize,
    name: &str,
    position: &str,
) -> Result<&'a str, Error> {
    args[index].string(&format!("{position} to {name}() needs to be a string."))
}
fn java_round(n: f64) -> f64 {
    // Math.round is floor(a + 0.5), but adding 0.5 first would round some values
    // immediately below 0.5 upward. Compare the fractional part instead.
    let floor = n.floor();
    let rounded = if n - floor >= 0.5 { floor + 1.0 } else { floor };
    (rounded as i64) as f64
}
pub(super) fn install(globals: &ValueMap, seed: u64, time: Supplier, date: Supplier) {
    globals
        .set("PI", Value::Number(std::f64::consts::PI))
        .unwrap();
    globals.set_supplier("time", time).unwrap();
    globals.set_supplier("date", date).unwrap();
    let rng = Rc::new(RefCell::new(JavaRandom::new(seed)));
    for name in [
        "round",
        "roundToString",
        "floor",
        "ceil",
        "abs",
        "random",
        "string",
        "toUpper",
        "toLower",
        "contains",
        "replace",
        "pad",
    ] {
        let rng = rng.clone();
        globals
            .set(
                name,
                Value::Function(Rc::new(move |args| {
                    match name {
                        "round" | "roundToString" => {
                            let n = match args.len() {
                                1 => java_round(number(args, 0, "round", "Argument")?),
                                2 => {
                                    let b = number(args, 1, "round", "Second argument")?;
                                    let a = number(args, 0, "round", "First argument")?;
                                    let x = 10f64.powf((b as i32) as f64);
                                    java_round(a * x) / x
                                }
                                n => {
                                    return Err(Error(format!(
                                        "round() requires 1 or 2 arguments, got {n}."
                                    )));
                                }
                            };
                            Ok(if name == "round" {
                                Value::Number(n)
                            } else {
                                Value::String(super::value::double_string(n))
                            })
                        }
                        "floor" | "ceil" | "abs" => {
                            arity(name, args, 1)?;
                            let n = number(args, 0, name, "Argument")?;
                            Ok(Value::Number(match name {
                                "floor" => n.floor(),
                                "ceil" => n.ceil(),
                                _ => n.abs(),
                            }))
                        }
                        "random" => {
                            let (min, max) = match args.len() {
                                0 => (0.0, 1.0),
                                2 => {
                                    let max = number(args, 1, name, "Second argument")?;
                                    (number(args, 0, name, "First argument")?, max)
                                }
                                n => {
                                    return Err(Error(format!(
                                        "random() requires 0 or 2 arguments, got {n}."
                                    )));
                                }
                            };
                            Ok(Value::Number(
                                min + (max - min) * rng.borrow_mut().next_double(),
                            ))
                        }
                        "string" => {
                            arity(name, args, 1)?;
                            Ok(Value::String(args[0].display()?))
                        }
                        "toUpper" | "toLower" => {
                            arity(name, args, 1)?;
                            let s = string(args, 0, name, "Argument")?;
                            Ok(Value::String(if name == "toUpper" {
                                s.to_uppercase()
                            } else {
                                s.to_lowercase()
                            }))
                        }
                        "contains" => {
                            arity("replace", args, 2)?;
                            let search = string(args, 1, name, "Second argument")?;
                            let text = string(args, 0, name, "First argument")?;
                            Ok(Value::Bool(text.contains(search)))
                        }
                        "replace" => {
                            arity(name, args, 3)?;
                            let to = string(args, 2, name, "Third argument")?;
                            let from = string(args, 1, name, "Second argument")?;
                            let text = string(args, 0, name, "First argument")?;
                            let count = if from.is_empty() {
                                text.encode_utf16().count() + 1
                            } else {
                                text.matches(from).count()
                            };
                            if text.len().saturating_add(count.saturating_mul(to.len())) > 1_048_576
                            {
                                return Err(Error("Starscript output limit exceeded.".into()));
                            }
                            // Java inserts between UTF-16 code units. Splitting a supplementary
                            // character cannot be represented by Rust String: reject explicitly.
                            if from.is_empty()
                                && !to.is_empty()
                                && text.chars().any(|c| c.len_utf16() == 2)
                            {
                                return Err(Error(
                                    "Replacement would create unpaired UTF-16 surrogates.".into(),
                                ));
                            }
                            Ok(Value::String(text.replace(from, to)))
                        }
                        "pad" => {
                            arity(name, args, 2)?;
                            let width = number(args, 1, name, "Second argument")? as i32;
                            let text = args[0].display()?;
                            let length = text.encode_utf16().count();
                            if width == i32::MIN || length >= width.unsigned_abs() as usize {
                                return Ok(Value::String(text));
                            }
                            if width.unsigned_abs() > 1_048_576 {
                                return Err(Error("Starscript output limit exceeded.".into()));
                            }
                            let spaces = " ".repeat(width.unsigned_abs() as usize - length);
                            Ok(Value::String(if width >= 0 {
                                format!("{spaces}{text}")
                            } else {
                                format!("{text}{spaces}")
                            }))
                        }
                        _ => unreachable!(),
                    }
                })),
            )
            .unwrap();
    }
}
