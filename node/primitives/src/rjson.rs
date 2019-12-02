#![no_std]
#![feature(alloc)]
//#![feature(core_intrinsics)]
extern crate alloc;
use alloc::string::String;
use core::option::Option;
use core::convert::From;
use alloc::vec::Vec;
use alloc::collections::btree_map::BTreeMap;

pub trait Array <T: Value<Self, O, N>, O: Object<T, Self, N>, N: Null<T, Self, O>> where Self: Sized{
    fn push(&mut self, v: T);
    fn new() -> Self;
}

pub trait Object<T: Value<A, Self, N>, A: Array<T, Self, N>, N: Null<T, A, Self>> where Self: Sized{
    fn insert(&mut self, k: String, v: T);
    fn new() -> Self;
}

pub trait Null<T: Value<A, O, Self>, A: Array<T, O, Self>, O: Object<T, A, Self>> where Self: Sized{
    fn new() -> Self;
}

pub trait Value<A: Array<Self, O, N>, O: Object<Self, A, N>, N: Null<Self, A, O>>:
From<String> + From<f64> + From<bool> + From<A> + From<O> + From<N> {
}

fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\t' || c == '\n' || c == '\r'
}
pub fn parse<T: Value<A, O, N>, A: Array<T, O, N>, O: Object<T, A, N>, N: Null<T, A, O>>
(src: &[char], index: &mut usize) -> Option<T> {
    while src.len() > *index && is_space(src[*index]) {
        *index += 1;
    }
    if src.len() <= *index {
        return Option::None;
    }
    if src[*index] == '{' {
        parse_object::<T, A, O, N>(src, index).map(|v| T::from(v))
    } else if src[*index] == '[' {
        parse_array::<T, A, O, N>(src, index).map(|v| T::from(v))
    } else if src[*index] == 't' {
        parse_true(src, index).map(|v| T::from(v))
    } else if src[*index] == 'f' {
        parse_false(src, index).map(|v| T::from(v))
    } else if src[*index] == '"' {
        parse_string(src, index).map(|v| T::from(v))
    } else if src[*index] == 'n' {
        parse_null::<T, A, O, N>(src, index).map(|v| T::from(v))
    } else if src[*index] == '-' || src[*index].is_ascii_digit() {
        parse_number(src, index).map(|v| T::from(v))
    } else {
        Option::None
    }
}

fn parse_object<T: Value<A, O, N>, A: Array<T, O, N>, O: Object<T, A, N>, N: Null<T, A, O>>
(src: &[char], index: &mut usize) -> Option<O> {
    if src.len() <= *index + 1 || src[*index] != '{' {
        return Option::None;
    }
    *index += 1;
    let mut v = O::new();
    while src.len() > *index {
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        if src[*index] == '}' {
            *index += 1;
            return Some(v);
        }
        let k = parse_string(src, index);
        if k.is_none() {
            return Option::None;
        }
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        if src[*index] != ':' {
            return Option::None;
        }
        *index += 1;
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        let c = parse::<T, A, O, N>(src, index);
        if c.is_none() {
            return Option::None;
        }
        v.insert(k.unwrap(), c.unwrap());
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        if src[*index] == ',' {
            *index += 1;
        } else if src[*index] == '}' {
            *index += 1;
            return Some(v);
        } else {
            return Option::None;
        }
    }
    Option::None
}

fn parse_array<T: Value<A, O, N>, A: Array<T, O, N>, O: Object<T, A, N>, N: Null<T, A, O>>
(src: &[char], index: &mut usize) -> Option<A> {
    if src.len() <= *index + 1 || src[*index] != '[' {
        return Option::None;
    }
    *index += 1;
    let mut v = A::new();
    while src.len() > *index {
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        if src[*index] == ']' {
            *index += 1;
            return Some(v);
        }
        let i = parse::<T, A, O, N>(src, index);
        if i.is_none() {
            return Option::None;
        }
        v.push(i.unwrap());
        while src.len() > *index && is_space(src[*index]) {
            *index += 1;
        }
        if src.len() <= *index {
            return Option::None;
        }
        if src[*index] == ',' {
            *index += 1;
        } else if src[*index] == ']' {
            *index += 1;
            return Some(v);
        } else {
            return Option::None;
        }
    }
    Option::None
}

fn parse_true(src: &[char], index: &mut usize) -> Option<bool> {
    let mut test_true = "true".chars();
    while src.len() > *index {
        let c = test_true.next();
        if c.is_none() {
            return Some(true);
        }
        if src[*index] == c.unwrap() {
            *index += 1;
        } else {
            return Option::None;
        }
    }
    Option::None
}
fn parse_false(src: &[char], index: &mut usize) -> Option<bool> {
    let mut test_false = "false".chars();
    while src.len() > *index {
        let c = test_false.next();
        if c.is_none() {
            return Some(false);
        }
        if src[*index] == c.unwrap() {
            *index += 1;
        } else {
            return Option::None;
        }
    }
    Option::None
}

fn parse_null<T: Value<A, O, N>, A: Array<T, O, N>, O: Object<T, A, N>, N: Null<T, A, O>>
(src: &[char], index: &mut usize) -> Option<N> {
    let mut test_null = "null".chars();
    while src.len() > *index {
        let c = test_null.next();
        if c.is_none() {
            return Some(N::new());
        }
        if src[*index] == c.unwrap() {
            *index += 1;
        } else {
            return Option::None;
        }
    }
    Option::None
}

fn parse_string_unicode(src: &[char], index: &mut usize) -> Option<char> {
    if src.len() <= *index + 4 {
        return Option::None;
    }
    let mut v: u32 = 0;
    for i in 1..5 {
        let d = src[*index + i].to_digit(16).unwrap_or(16);
        if d == 16 {
            return Option::None;
        }
        v = v * 16 + d;
    }
    *index += 4; // because there is another `*index += 1` in `parse_string`
    use core::char;
    unsafe { Some(char::from_u32_unchecked(v)) }
}

fn parse_string(src: &[char], index: &mut usize) -> Option<String> {
    if src.len() <= *index + 1 || src[*index] != '"'  {
        return Option::None;
    }
    *index += 1;
    let mut v = String::new();
    let mut escaped = false;
    while src.len() > *index {
        if escaped {
            let c = match src[*index] {
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\n' => '\0',
                '\r' => '\0',
                'u' => parse_string_unicode(src, index).unwrap_or('\u{fffd}'),
                _ => src[*index]
            };
            if c!= '\0' {
                v.push(c);
            }
            escaped = false;
        } else if src[*index] == '\\' {
            escaped = true;
        } else if src[*index] == '"' {
            *index += 1;
            return Some(v);
        } else {
            v.push(src[*index]);
        }
        *index += 1;
    }
    Option::None
}

fn parse_number_integer(src: &[char], index: &mut usize) -> f64 {
    let mut v: f64 = 0 as f64;
    while src.len() > *index && src[*index].is_ascii_digit() {
        v = v * 10.0 + src[*index].to_digit(10).unwrap() as f64;
        *index += 1;
    }
    v
}

fn parse_number_decimal(src: &[char], index: &mut usize) -> f64 {
    let head = *index;
    let v = parse_number_integer(src, index);
    v * unsafe { core::intrinsics::powif64(0.1, (*index - head) as i32) }
}

fn parse_number(src: &[char], index: &mut usize) -> Option<f64> {
    let mut v: f64 = 0 as f64;
    let mut sign = 1;
    if src.len() <= *index {
        return Option::None;
    }
    if src[*index] == '-' {
        sign = -1;
        *index += 1;
        if src.len() <= *index {
            return Option::None;
        }
    }
    if src[*index] != '0' {
        v += parse_number_integer(src, index);
    } else {
        *index += 1;
    }
    if src.len() <= *index {
        return Some(v * sign as f64);
    }
    if src[*index] == '.' {
        *index += 1;
        v += parse_number_decimal(src, index);
        if src.len() <= *index {
            return Some(v * sign as f64);
        }
    }
    if src[*index] == 'e' || src[*index] == 'E' {
        *index += 1;
        if src.len() <= *index {
            return Option::None;
        }
        let mut e_sign = 1;
        if src[*index] == '-' || src[*index] == '+' {
            e_sign = if src[*index] == '-' { -1 } else { 1 };
            *index += 1;
            if src.len() <= *index {
                return Option::None;
            }
        }
        let e = parse_number_integer(src, index);
        v *= unsafe { core::intrinsics::powif64(10.0, e as i32 * e_sign) };
    }
    Some(v * sign as f64)
}

//
#[derive(Clone)]
pub enum JsonValue {
    Null,
    Number(f64),
    Bool(bool),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
    None
}

pub struct JsonArray(Vec<JsonValue>);
pub struct JsonObject(BTreeMap<String, JsonValue>);

impl Array<JsonValue, JsonObject, JsonValue> for JsonArray {
    fn new() -> Self {
        JsonArray(Vec::new())
    }
    fn push(&mut self, v: JsonValue) {
        self.0.push(v)
    }
}

impl Object<JsonValue, JsonArray, JsonValue> for JsonObject {
    fn new<'b>() -> Self {
        JsonObject(BTreeMap::new())
    }
    fn insert(&mut self, k: String, v: JsonValue) {
        self.0.insert(k, v);
    }
}

impl Null<JsonValue, JsonArray, JsonObject> for JsonValue {
    fn new() -> Self {
        JsonValue::Null
    }
}
impl Value<JsonArray, JsonObject, JsonValue> for JsonValue {}

impl From<f64> for JsonValue {
    fn from(v: f64) -> Self {
        JsonValue::Number(v)
    }
}
impl From<bool> for JsonValue {
    fn from(v: bool) -> Self {
        JsonValue::Bool(v)
    }
}
impl From<String> for JsonValue {
    fn from(v: String) -> Self{
        JsonValue::String(v)
    }
}
impl From<JsonArray> for JsonValue {
    fn from(v: JsonArray) -> Self {
        JsonValue::Array(v.0)
    }
}
impl From<JsonObject> for JsonValue {
    fn from(v: JsonObject) -> Self {
        JsonValue::Object(v.0)
    }
}
/*
impl std::fmt::Debug for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            JsonValue::Null => f.write_str("null"),
            JsonValue::String(ref v) => f.write_fmt(format_args!("\"{}\"", v)),
            JsonValue::Number(ref v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::Bool(ref v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::Array(ref v) => f.write_fmt(format_args!("{:?}", v)),
            JsonValue::Object(ref v) => f.write_fmt(format_args!("{:#?}", v))
        }
    }
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", *self))
    }
}*/

pub fn is_null(o:&JsonValue) -> bool {
    if let JsonValue::Null = o {
        return true;
    }

    return false;
}

pub fn is_bool(o:&JsonValue) -> bool {
    if let JsonValue::Bool(_) = o {
        return true;
    }

    return false;
}

pub fn is_array(o:&JsonValue) -> bool {
    if let JsonValue::Array(_) = o {
        return true;
    }

    return false;
}

pub fn is_object(o:&JsonValue) -> bool {
    if let JsonValue::Object(_) = o {
        return true;
    }

    return false;
}

pub fn is_number(o:&JsonValue) -> bool {
    if let JsonValue::Number(_) = o {
        return true;
    }

    return false;
}

pub fn is_string(o:&JsonValue) -> bool {
    if let JsonValue::String(_) = o {
        return true;
    }

    return false;
}

pub fn is_none(o:&JsonValue) -> bool {
    if let JsonValue::None = o {
        return true;
    }

    return false;
}

//getter
pub fn get_number(o:&JsonValue) -> Option<f64> {
    if let JsonValue::Number(num) = o {
        return Some(*num)
    }

    return None;
}

pub fn get_bool(o:&JsonValue) -> Option<bool> {
    if let JsonValue::Bool(b) = o {
        return Some(*b)
    }

    return None;
}

pub fn get_string<'a>(o:&'a JsonValue) -> Option<&'a str> {
    if let JsonValue::String(string) = o {
        return Some(string)
    }

    return None;
}

pub fn get_array<'a>(o:&'a JsonValue) -> Vec<&'a JsonValue> {
    let mut v : Vec<&JsonValue> = Vec::new();

    if let JsonValue::Array(array) = o {
        for item in array {
            v.push(item);
        }

        return v;
    }

    v
}

//parser
fn get_object_keys(o: &JsonValue) -> Vec<&str> {
    let mut v:Vec<&str> = Vec::new();
    if let JsonValue::Object(map) = o {
        for (key, _v) in map {
            v.push(key);
        }
    }

    return v;
}

pub fn get_value_by_key<'a>(o:&'a JsonValue, key:&str) -> Option<&'a JsonValue> {
    if let JsonValue::Object(map) = o {
        for (k, v) in map {
            if k == key {
                return Some(&v);
            }
        }
    }

    None
}

pub fn get_value_by_key_recursively<'a>(o:&'a JsonValue, key:&str) -> Option<&'a JsonValue> {
    if let JsonValue::Object(map) = o {
        for (k, v) in map {
            if k == key {
                return Some(&v);
            }

            let v1 = get_value_by_key_recursively(&v, key);
            if let Some(v1) = v1 {
                return Some(v1);
            }
        }
    } else if let JsonValue::Array(vec) = o {
        for v in vec {
            let v1 = get_value_by_key_recursively(&v, key);
            if let Some(v1) = v1 {
                return Some(v1);
            }
        }
    }

    None
}

pub fn get_value_by_keys<'a>(o:&'a JsonValue, keys:&str) -> Option<&'a JsonValue> {
    let key_array:Vec<&str> = keys.split(".").collect();
    let mut v = get_value_by_key_recursively(&o, key_array[0]).unwrap_or(&JsonValue::Null);

    if is_null(v) {
        return None;
    }

    if key_array.len() == 1 {
        return Some(v);
    }

    for i in 1..key_array.len() {
        let v1 =  get_value_by_key(&v, key_array[i]).unwrap_or(&JsonValue::Null);
        if !is_null(v1) {
            if i == key_array.len() - 1 {
                return Some(v1);
            }

            v = v1;
        }
    }

    None
}

pub fn find_object_by_key_and_value(o: &JsonValue, key:&str, value:&str) -> bool {
    let v = get_value_by_key(&o, key);
    if let Some(v) = v {
        if is_string(v) {
            let str = get_string(v);
            if let Some(str) = str {
                if str == value {
                    return true;
                }
            }
        }
    }

    return false;
}
/*fn get_key<'a>(o:JsonValue) -> Option<&'a str> {
    if Self::is_object(o) {
        for (key, _v) in o as JsonValue::JsonObject {
            return Some(&key);
        }
    }

    None
}*/

mod test {
    use super::*;

    #[test]
    fn test_rjson() {
        let json_str = r#"{"time":{"updated":"Dec 2, 2019 00:36:00 UTC","updatedISO":"2019-12-02T00:36:00+00:00","updateduk":"Dec 2, 2019 at 00:36 GMT"},"disclaimer":"This data was produced from the CoinDesk Bitcoin Price Index (USD). Non-USD currency data converted using hourly conversion rate from openexchangerates.org","bpi":{"USD":{"code":"USD","rate":"7,409.7067","description":"United States Dollar","rate_float":7409.7067}}}"#;

        let data_array: Vec<char> = json_str.chars().collect();
        let mut index:usize = 0;
        let o = parse::<JsonValue, JsonArray, JsonObject, JsonValue>(&*data_array, &mut index).unwrap_or(JsonValue::None);
        assert_eq!(false, is_none(&o));

        let usd = get_value_by_key_recursively(&o, "USD").unwrap();
        assert_eq!(true, is_object(&usd));

        let rate = get_value_by_key(&usd, "rate").unwrap();
        assert_eq!(true, is_string(&rate));

        let rate = get_string(&rate).unwrap();
        let price = rate.replace(",","").parse::<f64>().unwrap_or(0.0) as u32;
        assert_eq!(true, price == 7409);
    }
}