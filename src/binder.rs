//! Function binding system for Mortar.
//!
//! Mortar 函数绑定系统。
//!
//! # Type System
//!
//! Mortar uses strongly-typed wrappers for function arguments:
//! - [`MortarString`] - for string values
//! - [`MortarNumber`] - for numeric values (f64)
//! - [`MortarBoolean`] - for boolean values
//! - [`MortarVoid`] - for void/unit values
//!
//! # Example
//!
//! ```no_run
//! use bevy_mortar_bond::{MortarString, MortarNumber, MortarBoolean};
//!
//! // Clear, type-safe function signature
//! fn create_message(verb: MortarString, obj: MortarString, level: MortarNumber) -> String {
//!     format!("{}{}{}", verb.as_str(), obj.as_str(), "!".repeat(level.as_usize()))
//! }
//! ```

use std::collections::HashMap;

/// String type for Mortar functions.
///
/// Mortar 函数的字符串类型。
#[derive(Debug, Clone, PartialEq)]
pub struct MortarString(pub String);

impl std::fmt::Display for MortarString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Number type for Mortar functions.
///
/// Mortar 函数的数字类型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MortarNumber(pub f64);

impl std::fmt::Display for MortarNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Boolean type for Mortar functions.
///
/// Mortar 函数的布尔类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MortarBoolean(pub bool);

impl std::fmt::Display for MortarBoolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Void type for Mortar functions.
///
/// Mortar 函数的空类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MortarVoid;

impl std::fmt::Display for MortarVoid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

/// Arguments and return values for Mortar functions.
///
/// Mortar 函数的参数和返回值。
#[derive(Debug, Clone)]
pub enum MortarValue {
    String(MortarString),
    Number(MortarNumber),
    Boolean(MortarBoolean),
    Void,
}

impl MortarString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MortarNumber {
    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl MortarBoolean {
    pub fn as_bool(&self) -> bool {
        self.0
    }
}

impl MortarValue {
    pub fn as_string(&self) -> Option<&MortarString> {
        match self {
            MortarValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<MortarNumber> {
        match self {
            MortarValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<MortarBoolean> {
        match self {
            MortarValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            MortarValue::String(s) => s.0.clone(),
            MortarValue::Number(n) => n.0.to_string(),
            MortarValue::Boolean(b) => b.0.to_string(),
            MortarValue::Void => String::new(),
        }
    }

    /// Parse a string argument into a MortarValue.
    ///
    /// 将字符串参数解析为 MortarValue。
    pub fn parse(s: &str) -> Self {
        // Try to parse as number first
        if let Ok(n) = s.parse::<f64>() {
            return MortarValue::Number(MortarNumber(n));
        }
        // Try to parse as boolean
        match s {
            "true" => return MortarValue::Boolean(MortarBoolean(true)),
            "false" => return MortarValue::Boolean(MortarBoolean(false)),
            _ => {}
        }
        // Default to string (remove quotes if present)
        let trimmed = s.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            MortarValue::String(MortarString(trimmed[1..trimmed.len() - 1].to_string()))
        } else {
            MortarValue::String(MortarString(s.to_string()))
        }
    }
}

// From implementations for specific types
impl From<String> for MortarString {
    fn from(s: String) -> Self {
        MortarString(s)
    }
}

impl From<&str> for MortarString {
    fn from(s: &str) -> Self {
        MortarString(s.to_string())
    }
}

impl From<f64> for MortarNumber {
    fn from(n: f64) -> Self {
        MortarNumber(n)
    }
}

impl From<i32> for MortarNumber {
    fn from(n: i32) -> Self {
        MortarNumber(n as f64)
    }
}

impl From<usize> for MortarNumber {
    fn from(n: usize) -> Self {
        MortarNumber(n as f64)
    }
}

impl From<bool> for MortarBoolean {
    fn from(b: bool) -> Self {
        MortarBoolean(b)
    }
}

// From implementations for MortarValue
impl From<MortarString> for MortarValue {
    fn from(s: MortarString) -> Self {
        MortarValue::String(s)
    }
}

impl From<String> for MortarValue {
    fn from(s: String) -> Self {
        MortarValue::String(MortarString(s))
    }
}

impl From<&str> for MortarValue {
    fn from(s: &str) -> Self {
        MortarValue::String(MortarString(s.to_string()))
    }
}

impl From<MortarNumber> for MortarValue {
    fn from(n: MortarNumber) -> Self {
        MortarValue::Number(n)
    }
}

impl From<f64> for MortarValue {
    fn from(n: f64) -> Self {
        MortarValue::Number(MortarNumber(n))
    }
}

impl From<i32> for MortarValue {
    fn from(n: i32) -> Self {
        MortarValue::Number(MortarNumber(n as f64))
    }
}

impl From<usize> for MortarValue {
    fn from(n: usize) -> Self {
        MortarValue::Number(MortarNumber(n as f64))
    }
}

impl From<MortarBoolean> for MortarValue {
    fn from(b: MortarBoolean) -> Self {
        MortarValue::Boolean(b)
    }
}

impl From<bool> for MortarValue {
    fn from(b: bool) -> Self {
        MortarValue::Boolean(MortarBoolean(b))
    }
}

impl From<MortarVoid> for MortarValue {
    fn from(_: MortarVoid) -> Self {
        MortarValue::Void
    }
}

impl From<()> for MortarValue {
    fn from(_: ()) -> Self {
        MortarValue::Void
    }
}

/// A function that can be called from Mortar.
///
/// 可以从 Mortar 调用的函数。
pub type MortarFunction = Box<dyn Fn(&[MortarValue]) -> MortarValue + Send + Sync>;

/// A registry for Mortar functions.
///
/// Mortar 函数注册表。
#[derive(Default)]
pub struct MortarFunctionRegistry {
    functions: HashMap<String, MortarFunction>,
}

impl MortarFunctionRegistry {
    /// Creates a new function registry.
    ///
    /// 创建一个新的函数注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a function (internal use by macros).
    pub fn register<F>(&mut self, name: impl Into<String>, func: F)
    where
        F: Fn(&[MortarValue]) -> MortarValue + Send + Sync + 'static,
    {
        self.functions.insert(name.into(), Box::new(func));
    }

    /// Calls a function by name with the given arguments.
    ///
    /// 按名称调用函数，并传递参数。
    pub fn call(&self, name: &str, args: &[MortarValue]) -> Option<MortarValue> {
        self.functions.get(name).map(|f| f(args))
    }
}

// TryFrom implementations for specific types
impl TryFrom<MortarValue> for MortarString {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::String(s) => Ok(s),
            MortarValue::Number(n) => Ok(MortarString(n.0.to_string())),
            MortarValue::Boolean(b) => Ok(MortarString(b.0.to_string())),
            MortarValue::Void => Ok(MortarString(String::new())),
        }
    }
}

impl TryFrom<MortarValue> for MortarNumber {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n),
            MortarValue::String(s) => s.0.parse().map(MortarNumber).map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for MortarBoolean {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

// TryFrom implementations for common types
impl TryFrom<MortarValue> for String {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::String(s) => Ok(s.0),
            MortarValue::Number(n) => Ok(n.0.to_string()),
            MortarValue::Boolean(b) => Ok(b.0.to_string()),
            MortarValue::Void => Ok(String::new()),
        }
    }
}

impl TryFrom<MortarValue> for f64 {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n.0),
            MortarValue::String(s) => s.0.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for i32 {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n.0 as i32),
            MortarValue::String(s) => s.0.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for usize {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n.0 as usize),
            MortarValue::String(s) => s.0.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for bool {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Boolean(b) => Ok(b.0),
            _ => Err(()),
        }
    }
}
