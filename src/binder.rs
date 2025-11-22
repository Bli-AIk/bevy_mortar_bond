//! Function binding system for Mortar.
//!
//! Mortar 函数绑定系统。

use std::collections::HashMap;

/// Arguments and return values for Mortar functions.
///
/// Mortar 函数的参数和返回值。
#[derive(Debug, Clone)]
pub enum MortarValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Void,
}

impl MortarValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            MortarValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            MortarValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MortarValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            MortarValue::String(s) => s.clone(),
            MortarValue::Number(n) => n.to_string(),
            MortarValue::Boolean(b) => b.to_string(),
            MortarValue::Void => String::new(),
        }
    }

    /// Parse a string argument into a MortarValue.
    ///
    /// 将字符串参数解析为 MortarValue。
    pub fn parse(s: &str) -> Self {
        // Try to parse as number first
        if let Ok(n) = s.parse::<f64>() {
            return MortarValue::Number(n);
        }
        // Try to parse as boolean
        match s {
            "true" => return MortarValue::Boolean(true),
            "false" => return MortarValue::Boolean(false),
            _ => {}
        }
        // Default to string (remove quotes if present)
        let trimmed = s.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            MortarValue::String(trimmed[1..trimmed.len() - 1].to_string())
        } else {
            MortarValue::String(s.to_string())
        }
    }
}

impl From<String> for MortarValue {
    fn from(s: String) -> Self {
        MortarValue::String(s)
    }
}

impl From<&str> for MortarValue {
    fn from(s: &str) -> Self {
        MortarValue::String(s.to_string())
    }
}

impl From<f64> for MortarValue {
    fn from(n: f64) -> Self {
        MortarValue::Number(n)
    }
}

impl From<bool> for MortarValue {
    fn from(b: bool) -> Self {
        MortarValue::Boolean(b)
    }
}

impl From<i32> for MortarValue {
    fn from(n: i32) -> Self {
        MortarValue::Number(n as f64)
    }
}

impl From<usize> for MortarValue {
    fn from(n: usize) -> Self {
        MortarValue::Number(n as f64)
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

// TryFrom implementations for common types
impl TryFrom<MortarValue> for String {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::String(s) => Ok(s),
            MortarValue::Number(n) => Ok(n.to_string()),
            MortarValue::Boolean(b) => Ok(b.to_string()),
            MortarValue::Void => Ok(String::new()),
        }
    }
}

impl TryFrom<MortarValue> for f64 {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n),
            MortarValue::String(s) => s.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for i32 {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n as i32),
            MortarValue::String(s) => s.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for usize {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Number(n) => Ok(n as usize),
            MortarValue::String(s) => s.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<MortarValue> for bool {
    type Error = ();

    fn try_from(value: MortarValue) -> Result<Self, Self::Error> {
        match value {
            MortarValue::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}
