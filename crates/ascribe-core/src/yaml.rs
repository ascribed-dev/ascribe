//! YAML: the values frontmatter and other project files hold, read and
//! written with `serde-saphyr`.
//!
//! `serde-saphyr` deserializes straight into Rust types and has no value type
//! of its own, so this module keeps one: [`Value`], a YAML document read
//! without a target type. [`from_str`] and [`to_string`] are the one place the
//! parser's and the writer's options are set, so every crate reads YAML the
//! same way: YAML 1.2's scalars, where `yes` and `0755` are strings, as they
//! were with `serde_yaml_ng`.
//!
//! A tag on a node (`!thing value`) is read past: the value is the node's own.

use std::fmt;
use std::hash::{Hash, Hasher};

use indexmap::IndexMap;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};

/// Reads `text` as YAML into `T`.
///
/// # Errors
///
/// When `text` isn't YAML, or doesn't hold a `T`; the error says where.
pub fn from_str<T: de::DeserializeOwned>(text: &str) -> Result<T, Error> {
    let options = serde_saphyr::options! {
        // YAML 1.2's core schema: `yes`, `on`, and `y` are strings.
        strict_booleans: true,
        // `0755` and `1_000` aren't numbers in YAML 1.2.
        leading_zero_numbers_as_strings: true,
        underscored_numbers_as_strings: true,
        // `.inf` and `.nan` are floats, as YAML says.
        non_finite_float_policy: serde_saphyr::NonFiniteFloatPolicy::PassThrough,
        // `<<` is a key like any other, as YAML 1.2 has it.
        merge_keys: serde_saphyr::MergeKeyPolicy::AsOrdinary,
        // The message, not a rendered snippet: callers place it themselves.
        with_snippet: false,
    };
    serde_saphyr::from_str_with_options(text, options).map_err(|error| {
        let at = error.location().and_then(|at| {
            let line = usize::try_from(at.line()).ok()?;
            let column = usize::try_from(at.column()).ok()?;
            let index = byte_index(text, line, column)?;
            Some(Location {
                index,
                line,
                column,
            })
        });
        Error::Read(Box::new(error), at)
    })
}

/// The byte offset of `line` and `column`, both counted from 1, the column in
/// characters.
fn byte_index(text: &str, line: usize, column: usize) -> Option<usize> {
    let start = if line <= 1 {
        0
    } else {
        text.match_indices('\n').nth(line - 2)?.0 + 1
    };
    let rest = text.get(start..)?;
    let within = rest
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(rest.len()))
        .nth(column.saturating_sub(1))?;
    Some(start + within)
}

/// Writes `value` as YAML, ending with a newline.
///
/// # Errors
///
/// When `value` can't be written as YAML.
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    let options = serde_saphyr::ser_options! {
        // Text with line breaks is a literal block; other text stays on its
        // line, however long.
        prefer_block_scalars: true,
        folded_wrap_chars: usize::MAX,
        // YAML 1.2's scalars, as `from_str` reads them, with no `%YAML`
        // line.
        yaml_12: true,
        no_lang_directive: true,
    };
    serde_saphyr::to_string_with_options(&value, options).map_err(|e| Error::Write(e.to_string()))
}

/// Why YAML couldn't be read or written.
#[derive(Debug)]
pub enum Error {
    /// The text isn't YAML, or doesn't hold the type asked for.
    Read(Box<serde_saphyr::Error>, Option<Location>),
    /// The value couldn't be written.
    Write(String),
}

/// Where in the YAML text an error is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    index: usize,
    line: usize,
    column: usize,
}

impl Location {
    /// The byte offset into the text read.
    pub fn index(&self) -> usize {
        self.index
    }
    /// The line, counted from 1.
    pub fn line(&self) -> usize {
        self.line
    }
    /// The column, counted from 1.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl Error {
    /// Where the error is, when the parser knows.
    pub fn location(&self) -> Option<Location> {
        match self {
            Error::Read(_, at) => *at,
            Error::Write(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Read(e, _) => {
                let options = serde_saphyr::render_options! {
                    snippets: serde_saphyr::SnippetMode::Off,
                };
                f.write_str(&e.render_with_options(options))
            }
            Error::Write(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

/// A YAML value read without a type to read it into.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd)]
pub enum Value {
    /// `~`, `null`, or nothing.
    #[default]
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// An integer or a float.
    Number(Number),
    /// Any other scalar.
    String(String),
    /// A list.
    Sequence(Sequence),
    /// Keys and their values, in the order written.
    Mapping(Mapping),
}

/// A YAML list.
pub type Sequence = Vec<Value>;

static NULL: Value = Value::Null;

impl Value {
    /// The value at `index`: a key of a mapping, or a position in a list.
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }
    /// The value at `index`, to change.
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_mut(self)
    }
    /// Whether this is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    /// Whether this is a boolean.
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }
    /// Whether this is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    /// The number, if it's an integer that fits an `i64`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }
    /// Whether this is an integer that fits an `i64`.
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }
    /// The number, if it's an integer that fits a `u64`.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }
    /// Whether this is an integer that fits a `u64`.
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }
    /// The number as a float, if it's a number.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }
    /// Whether this is a float.
    pub fn is_f64(&self) -> bool {
        matches!(self, Value::Number(n) if n.is_f64())
    }
    /// The text, if this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    /// Whether this is a string.
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }
    /// The items, if this is a list.
    pub fn as_sequence(&self) -> Option<&Sequence> {
        match self {
            Value::Sequence(s) => Some(s),
            _ => None,
        }
    }
    /// The items, to change, if this is a list.
    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        match self {
            Value::Sequence(s) => Some(s),
            _ => None,
        }
    }
    /// Whether this is a list.
    pub fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }
    /// The entries, if this is a mapping.
    pub fn as_mapping(&self) -> Option<&Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }
    /// The entries, to change, if this is a mapping.
    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }
    /// Whether this is a mapping.
    pub fn is_mapping(&self) -> bool {
        self.as_mapping().is_some()
    }
}

impl<I: Index> std::ops::Index<I> for Value {
    type Output = Value;
    /// The value at `index`, or null when there's none.
    fn index(&self, index: I) -> &Value {
        index.index_into(self).unwrap_or(&NULL)
    }
}

/// What a [`Value`] or a [`Mapping`] can be indexed by.
pub trait Index {
    #[doc(hidden)]
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value>;
    #[doc(hidden)]
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value>;
    #[doc(hidden)]
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value>;
    #[doc(hidden)]
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value>;
    #[doc(hidden)]
    fn remove_from(&self, m: &mut Mapping) -> Option<Value>;
}

impl Index for usize {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match v {
            Value::Sequence(s) => s.get(*self),
            Value::Mapping(m) => m.map.get(&Value::Number((*self).into())),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match v {
            Value::Sequence(s) => s.get_mut(*self),
            Value::Mapping(m) => m.map.get_mut(&Value::Number((*self).into())),
            _ => None,
        }
    }
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value> {
        m.map.get(&Value::Number((*self).into()))
    }
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value> {
        m.map.get_mut(&Value::Number((*self).into()))
    }
    fn remove_from(&self, m: &mut Mapping) -> Option<Value> {
        m.map.shift_remove(&Value::Number((*self).into()))
    }
}

impl Index for Value {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        v.as_mapping().and_then(|m| m.map.get(self))
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        v.as_mapping_mut().and_then(|m| m.map.get_mut(self))
    }
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value> {
        m.map.get(self)
    }
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value> {
        m.map.get_mut(self)
    }
    fn remove_from(&self, m: &mut Mapping) -> Option<Value> {
        m.map.shift_remove(self)
    }
}

impl Index for str {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        v.as_mapping().and_then(|m| self.key_of(m))
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        v.as_mapping_mut().and_then(|m| self.key_of_mut(m))
    }
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value> {
        m.map.get(&Value::String(self.to_owned()))
    }
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value> {
        m.map.get_mut(&Value::String(self.to_owned()))
    }
    fn remove_from(&self, m: &mut Mapping) -> Option<Value> {
        m.map.shift_remove(&Value::String(self.to_owned()))
    }
}

impl Index for String {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        self.as_str().index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        self.as_str().index_into_mut(v)
    }
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value> {
        self.as_str().key_of(m)
    }
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value> {
        self.as_str().key_of_mut(m)
    }
    fn remove_from(&self, m: &mut Mapping) -> Option<Value> {
        self.as_str().remove_from(m)
    }
}

impl<T: Index + ?Sized> Index for &T {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        (**self).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        (**self).index_into_mut(v)
    }
    fn key_of<'m>(&self, m: &'m Mapping) -> Option<&'m Value> {
        (**self).key_of(m)
    }
    fn key_of_mut<'m>(&self, m: &'m mut Mapping) -> Option<&'m mut Value> {
        (**self).key_of_mut(m)
    }
    fn remove_from(&self, m: &mut Mapping) -> Option<Value> {
        (**self).remove_from(m)
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == Some(other.as_str())
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == Some(*other)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<Mapping> for Value {
    fn from(m: Mapping) -> Self {
        Value::Mapping(m)
    }
}

impl From<Sequence> for Value {
    fn from(s: Sequence) -> Self {
        Value::Sequence(s)
    }
}

impl<T: Into<Number>> From<T> for Value {
    fn from(n: T) -> Self {
        Value::Number(n.into())
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<I: IntoIterator<Item = Value>>(items: I) -> Self {
        Value::Sequence(items.into_iter().collect())
    }
}

/// A YAML number: an integer, or a float.
#[derive(Clone, Copy, Debug)]
pub struct Number(N);

#[derive(Clone, Copy, Debug)]
enum N {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}

impl Number {
    /// The number, if it's an integer that fits an `i64`.
    pub fn as_i64(&self) -> Option<i64> {
        match self.0 {
            N::PosInt(n) => i64::try_from(n).ok(),
            N::NegInt(n) => Some(n),
            N::Float(_) => None,
        }
    }
    /// Whether this is an integer that fits an `i64`.
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }
    /// The number, if it's an integer that fits a `u64`.
    pub fn as_u64(&self) -> Option<u64> {
        match self.0 {
            N::PosInt(n) => Some(n),
            _ => None,
        }
    }
    /// Whether this is an integer that fits a `u64`.
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }
    /// The number as a float, if it's a number.
    pub fn as_f64(&self) -> Option<f64> {
        #[allow(clippy::cast_precision_loss)] // As serde_json does: a float near the integer.
        match self.0 {
            N::PosInt(n) => Some(n as f64),
            N::NegInt(n) => Some(n as f64),
            N::Float(n) => Some(n),
        }
    }
    /// Whether this is a float.
    pub fn is_f64(&self) -> bool {
        matches!(self.0, N::Float(_))
    }
    fn key(&self) -> (u8, u64) {
        match self.0 {
            N::PosInt(n) => (0, n),
            N::NegInt(n) => (1, n.cast_unsigned()),
            // Every NaN is the same key, as every other float is its bits.
            N::Float(f) if f.is_nan() => (2, 0),
            N::Float(f) => (2, (f + 0.0).to_bits()),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.0, other.0) {
            (N::Float(a), N::Float(b)) => a.partial_cmp(&b),
            (N::Float(_), _) | (_, N::Float(_)) => self.as_f64()?.partial_cmp(&other.as_f64()?),
            _ => Some(match (self.as_i64(), other.as_i64()) {
                (Some(a), Some(b)) => a.cmp(&b),
                _ => self.as_u64().cmp(&other.as_u64()),
            }),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            N::PosInt(n) => write!(f, "{n}"),
            N::NegInt(n) => write!(f, "{n}"),
            N::Float(n) if n.is_nan() => f.write_str(".nan"),
            N::Float(n) if n.is_infinite() && n < 0.0 => f.write_str("-.inf"),
            N::Float(n) if n.is_infinite() => f.write_str(".inf"),
            // `1.0`, not `1`, so it reads back as a float.
            N::Float(n) => write!(f, "{n:?}"),
        }
    }
}

macro_rules! from_unsigned {
    ($($t:ty)*) => {$(
        impl From<$t> for Number {
            fn from(n: $t) -> Self {
                Number(N::PosInt(n as u64))
            }
        }
    )*};
}

macro_rules! from_signed {
    ($($t:ty)*) => {$(
        impl From<$t> for Number {
            fn from(n: $t) -> Self {
                let n = n as i64;
                Number(if n < 0 { N::NegInt(n) } else { N::PosInt(n.cast_unsigned()) })
            }
        }
    )*};
}

from_unsigned!(u8 u16 u32 u64 usize);
from_signed!(i8 i16 i32 i64 isize);

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number(N::Float(n))
    }
}

impl From<f32> for Number {
    fn from(n: f32) -> Self {
        Number(N::Float(n.into()))
    }
}

/// A YAML mapping: keys and their values, in the order written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mapping {
    map: IndexMap<Value, Value>,
}

impl Mapping {
    /// An empty mapping.
    pub fn new() -> Self {
        Self::default()
    }
    /// An empty mapping with room for `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: IndexMap::with_capacity(capacity),
        }
    }
    /// Sets `key` to `value`, returning the value it had. A key already
    /// there keeps its place.
    pub fn insert(&mut self, key: Value, value: Value) -> Option<Value> {
        self.map.insert(key, value)
    }
    /// The value of the key `index`.
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.key_of(self)
    }
    /// The value of the key `index`, to change.
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Value> {
        index.key_of_mut(self)
    }
    /// Whether the key `index` is there.
    pub fn contains_key<I: Index>(&self, index: I) -> bool {
        index.key_of(self).is_some()
    }
    /// Removes `index`, keeping the order of the keys left.
    pub fn remove<I: Index>(&mut self, index: I) -> Option<Value> {
        index.remove_from(self)
    }
    /// Keeps only the entries `keep` returns true for, in order.
    pub fn retain(&mut self, keep: impl FnMut(&Value, &mut Value) -> bool) {
        self.map.retain(keep);
    }
    /// How many entries there are.
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    /// The entries, in order.
    pub fn iter(&self) -> indexmap::map::Iter<'_, Value, Value> {
        self.map.iter()
    }
    /// The entries, in order, their values to change.
    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, Value, Value> {
        self.map.iter_mut()
    }
    /// The keys, in order.
    pub fn keys(&self) -> indexmap::map::Keys<'_, Value, Value> {
        self.map.keys()
    }
    /// The values, in order.
    pub fn values(&self) -> indexmap::map::Values<'_, Value, Value> {
        self.map.values()
    }
    /// The values, in order, to change.
    pub fn values_mut(&mut self) -> indexmap::map::ValuesMut<'_, Value, Value> {
        self.map.values_mut()
    }
}

impl Hash for Mapping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.map.len().hash(state);
        for (k, v) in &self.map {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl PartialOrd for Mapping {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.map.iter().partial_cmp(other.map.iter())
    }
}

impl IntoIterator for Mapping {
    type Item = (Value, Value);
    type IntoIter = indexmap::map::IntoIter<Value, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'m> IntoIterator for &'m Mapping {
    type Item = (&'m Value, &'m Value);
    type IntoIter = indexmap::map::Iter<'m, Value, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'m> IntoIterator for &'m mut Mapping {
    type Item = (&'m Value, &'m mut Value);
    type IntoIter = indexmap::map::IterMut<'m, Value, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl FromIterator<(Value, Value)> for Mapping {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(items: I) -> Self {
        Mapping {
            map: items.into_iter().collect(),
        }
    }
}

impl Extend<(Value, Value)> for Mapping {
    fn extend<I: IntoIterator<Item = (Value, Value)>>(&mut self, items: I) {
        self.map.extend(items);
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Null => s.serialize_unit(),
            Value::Bool(b) => s.serialize_bool(*b),
            Value::Number(n) => n.serialize(s),
            Value::String(text) => s.serialize_str(text),
            Value::Sequence(items) => items.serialize(s),
            Value::Mapping(m) => m.serialize(s),
        }
    }
}

impl Serialize for Number {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            N::PosInt(n) => s.serialize_u64(n),
            N::NegInt(n) => s.serialize_i64(n),
            N::Float(n) => s.serialize_f64(n),
        }
    }
}

impl Serialize for Mapping {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ValueVisitor)
    }
}

impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::Mapping(m) => Ok(m),
            Value::Null => Ok(Mapping::new()),
            other => Err(de::Error::invalid_type(unexpected(&other), &"a mapping")),
        }
    }
}

fn unexpected(value: &Value) -> de::Unexpected<'_> {
    match value {
        Value::Null => de::Unexpected::Unit,
        Value::Bool(b) => de::Unexpected::Bool(*b),
        Value::Number(n) => match n.0 {
            N::PosInt(n) => de::Unexpected::Unsigned(n),
            N::NegInt(n) => de::Unexpected::Signed(n),
            N::Float(n) => de::Unexpected::Float(n),
        },
        Value::String(s) => de::Unexpected::Str(s),
        Value::Sequence(_) => de::Unexpected::Seq,
        Value::Mapping(_) => de::Unexpected::Map,
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any YAML value")
    }
    fn visit_bool<E>(self, b: bool) -> Result<Value, E> {
        Ok(Value::Bool(b))
    }
    fn visit_i64<E>(self, n: i64) -> Result<Value, E> {
        Ok(Value::Number(n.into()))
    }
    fn visit_u64<E>(self, n: u64) -> Result<Value, E> {
        Ok(Value::Number(n.into()))
    }
    fn visit_i128<E: de::Error>(self, n: i128) -> Result<Value, E> {
        // Past 64 bits, the digits.
        match (i64::try_from(n), u64::try_from(n)) {
            (Ok(n), _) => Ok(n.into()),
            (_, Ok(n)) => Ok(n.into()),
            _ => Ok(Value::String(n.to_string())),
        }
    }
    fn visit_u128<E: de::Error>(self, n: u128) -> Result<Value, E> {
        match u64::try_from(n) {
            Ok(n) => Ok(n.into()),
            Err(_) => Ok(Value::String(n.to_string())),
        }
    }
    fn visit_f64<E>(self, n: f64) -> Result<Value, E> {
        Ok(Value::Number(n.into()))
    }
    fn visit_str<E>(self, s: &str) -> Result<Value, E> {
        Ok(Value::String(s.to_owned()))
    }
    fn visit_string<E>(self, s: String) -> Result<Value, E> {
        Ok(Value::String(s))
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        Value::deserialize(d)
    }
    fn visit_newtype_struct<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        Value::deserialize(d)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<Value, A::Error> {
        let mut out = Vec::with_capacity(items.size_hint().unwrap_or(0));
        while let Some(item) = items.next_element()? {
            out.push(item);
        }
        Ok(Value::Sequence(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Value, A::Error> {
        let mut out = Mapping::with_capacity(entries.size_hint().unwrap_or(0));
        while let Some((k, v)) = entries.next_entry::<Value, Value>()? {
            out.insert(k, v);
        }
        Ok(Value::Mapping(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Value {
        from_str(text).unwrap()
    }

    #[test]
    fn scalars_are_yaml_1_2s() {
        let v = read(
            "a: yes\nb: on\nc: 0755\nd: 1_000\ne: true\nf: 2024-01-02\ng: 1.5\nh: -3\ni: ~\nj:\n",
        );
        for (key, text) in [
            ("a", "yes"),
            ("b", "on"),
            ("c", "0755"),
            ("d", "1_000"),
            ("f", "2024-01-02"),
        ] {
            assert_eq!(v.get(key).and_then(Value::as_str), Some(text), "{key}");
        }
        assert_eq!(v["e"], Value::Bool(true));
        assert_eq!(v["g"].as_f64(), Some(1.5));
        assert_eq!(v["h"].as_i64(), Some(-3));
        assert!(v["i"].is_null() && v["j"].is_null());
        assert!(v["missing"].is_null());
    }

    #[test]
    fn a_mapping_keeps_its_order_and_its_keys_of_any_kind() {
        let v = read("z: 1\na: 2\n1: one\n<<: {b: 3}\n");
        let keys: Vec<String> = v
            .as_mapping()
            .unwrap()
            .keys()
            .map(|k| to_string(k).unwrap().trim().to_owned())
            .collect();
        assert_eq!(keys, ["z", "a", "1", "<<"]);
        assert_eq!(v.get(1).and_then(Value::as_str), Some("one"));
    }

    #[test]
    fn a_tag_is_read_past() {
        assert_eq!(read("a: !thing value\n")["a"], "value");
    }

    #[test]
    fn an_error_says_where_in_bytes() {
        // The `é` is two bytes, so the column and the index differ.
        let error = from_str::<Value>("é: x\nkey: [1, 2\n").unwrap_err();
        let at = error.location().unwrap();
        assert_eq!((at.line(), at.column(), at.index()), (2, 6, 11));
        assert!(error.to_string().contains("line 2"), "{error}");
        assert!(from_str::<Value>("a: 1\na: 2\n").is_err());
    }

    #[test]
    fn written_text_reads_back_the_same() {
        let text = "title: \"Quill: the agent *\"\nversions:\n- \"3.5\"\n- 3\nnote: |\n  one\n  two\nempty: []\n";
        let v = read(text);
        assert_eq!(to_string(&v).unwrap(), text);
        assert_eq!(read(&to_string(&v).unwrap()), v);
    }
}
