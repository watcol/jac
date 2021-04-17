//! JSON Parser
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::collections::HashMap;
use crate::{Value, Parser};

peg::parser!{grammar json_parser() for str {
    pub rule json() -> Value
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
        / "u" h:hex() {? char::try_from(h).or(Err("hex")) }

    rule hex() -> u32
        = h:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<4,4>) {? h.parse().or(Err("hex")) }

    rule number() -> Value
        = i:signed() d:$(fraction() exponent()) {?
            if d.is_empty() {
                i.parse().map(Value::Int).or(Err("int"))
            } else {
                [i, d].concat().parse().map(Value::Float).or(Err("float"))
            }
        }

    rule exponent() -> &'input str
        = $(("e"/"E") signed() / "")

    rule fraction() -> &'input str
        = $("." ['0'..='9']+ / "")

    rule signed() -> &'input str
        = $(("+" / "-" / "") digits())

    rule digits() -> &'input str
        = $("0" / ['1'..='9']['0'..='9']*)
}}

#[derive(Copy, Clone, Debug)]
pub struct JsonParser;

impl Parser for JsonParser {
    type Err = peg::error::ParseError<peg::str::LineCol>;
    fn parse(s: &str) -> Result<Value, Self::Err> {
        json_parser::json(s)
    }
}
