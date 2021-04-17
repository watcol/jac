//! JSON5 Parser
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::collections::HashMap;
use crate::{Value, Parser};

peg::parser!{grammar json5_parser() for str {
    pub rule json5() -> Value
        = elem()

    rule _ = " " / "\n" / "\r" / "\t" / ""

    rule elem() -> Value
        = _ v:value() _ { v }

    rule value() -> Value
        = object()
        / array()
        / string()
        / number()
        / bool()
        / null()

    rule object() -> Value
        = "{" _ m:(member() ** ",") _ "}" { Value::Map(HashMap::from_iter(m)) }

    rule member() -> (String, Value)
        = _ s:string_() _ ":" e:elem() { (s, e) }

    rule array() -> Value
        = "[" _ e:(elem() ** ",") _ "]" { Value::Array(e) }

    rule bool() -> Value
        = "true" { Value::Boolean(true) }
        / "false" { Value::Boolean(false) }

    rule null() -> Value
        = "null" { Value::Null }

    rule string() -> Value
        = s:string_() { Value::String(s) }

    rule string_() -> String
        = "\"" s:character()* "\"" { String::from_iter(s) }

    rule character() -> char
        = c:$([^ '"' | '\\']) { c.chars().next().unwrap() }
        / "\\" e:escape() { e }

    rule escape() -> char
        = "\"" { '"' }
        / "\\" { '\\' }
        / "/"  { '/' }
        / "b"  { '\x08' }
        / "f"  { '\x0c' }
        / "n"  { '\n' }
        / "r"  { '\r' }
        / "t"  { '\t' }
        / "u" h:$(hex()*<4>) {? char::try_from(u32::from_str_radix(h, 16).or(Err("hexchar"))?).or(Err("escape")) }

    rule number() -> Value
        = "-" v:unsigned() {
            match v {
                Value::Int(i) => Value::Int(-i),
                Value::Float(f) => Value::Float(-f),
                _ => unreachable!(),
            }
        }
        / ("+"/"") v:unsigned() { v }

    rule unsigned() -> Value
        = "Infinity" { Value::Float(f64::INFINITY) }
        / "NaN" { Value::Float(f64::NAN) }
        / "0x" h:$(hex()+) {? Ok(Value::Int(i128::from_str_radix(h, 16).or(Err("hexint"))?)) }
        / d:$("." ['0'..='9']+ exponent()) {? Ok(Value::Float(d.parse().or(Err("float"))?)) }
        / i:digits() d:$(fraction() exponent()) {?
            Ok(if d.is_empty() {
                Value::Int(i.parse().or(Err("int"))?)
            } else {
                Value::Float([i, d].concat().parse().or(Err("float"))?)
            })
        }

    rule exponent() -> &'input str
        = $(("e"/"E") ("+"/"-"/"") digits() / "")

    rule fraction() -> &'input str
        = $("." ['0'..='9']+ / "")

    rule hex() -> &'input str
        = $(['0'..='9' | 'a'..='f' | 'A'..='F'])

    rule digits() -> &'input str
        = $("0" / ['1'..='9']['0'..='9']*)
}}

#[derive(Copy, Clone, Debug)]
pub struct Json5Parser;

impl Parser for Json5Parser {
    type Err = peg::error::ParseError<peg::str::LineCol>;
    fn parse(s: &str) -> Result<Value, Self::Err> {
        json5_parser::json5(s)
    }
}