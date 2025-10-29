use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Variable {
    name: String,
    r#type: VariableType,
    // position: usize,
}

impl Variable {
    pub fn from_config(variable: &str) -> Result<Option<Vec<Self>>, VariableError> {
        if variable.trim().is_empty() {
            return Ok(None);
        }
        let mut variables = Vec::new();
        for (_position, v) in variable.split(',').enumerate() {
            let variable = Variable::from_str(v)?;
            // variable.set_position(position);
            variables.push(variable);
        }
        Ok(Some(variables))
    }

    pub fn from_config_map(
        config: indexmap::IndexMap<String, config::Value>,
    ) -> Option<Vec<Variable>> {
        let values = config.get("variables")?.clone().into_string().ok()?;
        Variable::from_config(&values).ok()?
    }

    /// USERNAME will return
    /// ```lua
    /// local USERNAME = "{username}";
    /// ```
    pub fn command_local_lua<T>(&self, value: Option<T>, username: &str) -> String
    where
        T: Display,
    {
        if self.name() == "USERNAME" {
            return format!(r#"local {} = "{}";"#, &self.name, username);
        }
        match &self.r#type {
            VariableType::String(default) => {
                format!(
                    r#"local {} = "{}";"#,
                    &self.name,
                    value.map(|v| v.to_string()).unwrap_or(default.clone())
                )
            }
            VariableType::Int(default) => {
                format!(
                    "local {} = {};",
                    &self.name,
                    value.map(|v| v.to_string()).unwrap_or(default.to_string())
                )
            }
            VariableType::Float(default) => {
                format!(
                    "local {} = {};",
                    &self.name,
                    value.map(|v| v.to_string()).unwrap_or(default.to_string())
                )
            }
        }
    }

    /// Returns a [`Variable`] with the value set to the value from the message.
    /// TODO: Change to nom
    pub fn from_message(&self, msg: Option<&str>) -> Option<Self> {
        if msg.is_none() {
            return None;
        }
        let msg = msg.unwrap();
        for (i, _) in msg
            .to_lowercase()
            .match_indices(&self.name().to_lowercase())
        {
            let mut match_var = msg[i..].trim().strip_prefix(self.name()).unwrap();
            if !match_var.starts_with('=') {
                continue;
            }

            match_var = match_var.strip_prefix('=').unwrap().trim();
            match self.r#type {
                VariableType::String(_) => {
                    if let Some(rem) = match_var.strip_prefix('"') {
                        if let Some(i) = rem.find('"') {
                            let match_string = &rem[..i];
                            return Some(Self {
                                name: self.name.clone(),
                                r#type: VariableType::String(match_string.to_string()),
                                // position: todo!(),
                            });
                        }
                    } else if let Some(rem) = match_var.strip_prefix('\'') {
                        if let Some(i) = rem.find('\'') {
                            let match_string = &rem[..i];
                            return Some(Self {
                                name: self.name.clone(),
                                r#type: VariableType::String(match_string.to_string()),
                                // position: todo!(),
                            });
                        }
                    } else {
                        if let Some(match_str) = match_var.split_whitespace().next() {
                            return Some(Self {
                                name: self.name.clone(),
                                r#type: VariableType::String(match_str.to_string()),
                                // position: todo!(),
                            });
                        };
                    }
                    continue;
                }
                VariableType::Int(_) => {
                    if let Some(match_str) = match_var
                        .split(|c: char| c.is_whitespace() | c.is_alphabetic())
                        .next()
                    {
                        match i64::from_str(match_str) {
                            Ok(match_int) => {
                                return Some(Self {
                                    name: self.name.clone(),
                                    r#type: VariableType::Int(match_int),
                                    // position: todo!(),
                                });
                            }
                            Err(e) => {
                                tracing::error!(
                                    "While parsing message for {}: {e}\n {}",
                                    self.name(),
                                    match_str
                                );
                                continue;
                            }
                        }
                    };
                }
                VariableType::Float(_) => {
                    if let Some(match_str) = match_var
                        .split(|c: char| c.is_whitespace() | c.is_alphabetic())
                        .next()
                    {
                        match f64::from_str(match_str) {
                            Ok(match_float) => {
                                return Some(Self {
                                    name: self.name.clone(),
                                    r#type: VariableType::Float(match_float),
                                    // position: todo!(),
                                });
                            }
                            Err(e) => {
                                tracing::error!(
                                    "While parsing message for {}: {e}\n {}",
                                    self.name(),
                                    match_str
                                );
                                continue;
                            }
                        }
                    };
                }
            }
        }
        None
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variable_type(&self) -> &VariableType {
        &self.r#type
    }

    // pub fn position(&self) -> usize {
    //     self.position
    // }

    // pub fn set_position(&mut self, position: usize) {
    //     self.position = position;
    // }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl FromStr for Variable {
    type Err = VariableError;
    /// "x:float=0,y:float=0"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spliter = s.splitn(2, |c| c == ':' || c == '=');
        let name = match spliter.next() {
            Some(n) => {
                if n.trim().is_empty() {
                    return Err(VariableError::NoName(s.to_string()));
                }
                n.trim().to_string()
            }
            None => return Err(VariableError::NoName(s.to_string())),
        };
        let r#type = VariableType::from_str(s.strip_prefix(&name).unwrap_or_default())?;
        Ok(Self {
            name,
            r#type,
            // position: 0,
        })
    }
}

/// Accepted Variable types for [`Variable`] with there default value.
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd)]
pub enum VariableType {
    String(String),
    Int(i64),
    Float(f64),
}

impl Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableType::String(d) => d.fmt(f),
            VariableType::Int(d) => d.fmt(f),
            VariableType::Float(d) => d.fmt(f),
        }
    }
}

impl PartialEq for VariableType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for VariableType {}

impl Default for VariableType {
    fn default() -> Self {
        Self::String(String::default())
    }
}

impl FromStr for VariableType {
    type Err = VariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splitter = s.trim_start_matches(":").splitn(2, "=");
        let t = splitter.next().unwrap();
        let default = splitter.next();
        match t.to_lowercase().as_str() {
            "string" | "str" | "" => {
                return Ok(VariableType::String(
                    default.unwrap_or_default().to_string(),
                ))
            }
            "int" => {
                let s = if let Some(val) = default {
                    i64::from_str(val).map_err(|e| VariableError::ParseIntError(e))?
                } else {
                    0
                };
                Ok(VariableType::Int(s))
            }
            "float" | "num" | "number" => {
                let s = if let Some(val) = default {
                    f64::from_str(val).map_err(|e| VariableError::ParseFloatError(e))?
                } else {
                    0.0
                };
                Ok(VariableType::Float(s))
            }
            _ => return Err(VariableError::InvalidType(t.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic, PartialEq)]
pub enum VariableError {
    #[error("Invalid variable type: {0}")]
    InvalidType(String),
    #[error("{0}")]
    ParseIntError(#[source] ParseIntError),
    #[error("{0}")]
    ParseFloatError(#[source] ParseFloatError),
    #[error("No name provided: {0}")]
    NoName(String),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use Variable as V;
    use VariableType as VT;

    #[rstest]
    #[case("x", "x", VT::String(String::from("")))]
    #[case("x:String", "x", VT::String(String::from("")))]
    #[case("x=hello-world", "x", VT::String(String::from("hello-world")))]
    #[case("x:String=stuff", "x", VT::String(String::from("stuff")))]
    #[case("x:int", "x", VT::Int(0))]
    #[case("x:iNT", "x", VT::Int(0))]
    #[case("x:int=5", "x", VT::Int(5))]
    #[case("x:float", "x", VT::Float(0.0))]
    #[case("x:float=5.1", "x", VT::Float(5.1))]
    #[case("x:float=5", "x", VT::Float(5.0))]
    #[case("x:num=5", "x", VT::Float(5.0))]
    fn variable_from_str(
        #[case] variable: Variable,
        #[case] name: &str,
        #[case] r#type: VariableType,
    ) {
        let expected = Variable {
            name: name.into(),
            r#type,
        };
        assert_eq!(variable, expected);
    }

    #[rstest]
    #[case("", VariableError::NoName("".to_string()) )]
    #[case(":Int", VariableError::NoName(":Int".to_string()) )]
    #[case("x:Int=notanum", VariableError::ParseIntError(i64::from_str("notanum").unwrap_err()) )]
    #[case("x:Float=notanum", VariableError::ParseFloatError(f64::from_str("notanum").unwrap_err()) )]
    #[case("x:dwhalj=5", VariableError::InvalidType("dwhalj".to_string()))]
    fn variable_from_str_errors(#[case] input_str: &str, #[case] expected: VariableError) {
        assert_eq!(Variable::from_str(input_str), Err(expected));
    }

    #[rstest]
    #[case("", None)]
    #[case("x:int=5", Some(vec![Variable::from_str("x:int=5").unwrap()]))]
    #[case("x:int=5,y", Some(vec![V{ name: "x".into(), r#type: VT::Int(5) }, V{ name: "y".into(), r#type: VT::String("".into()) } ]))]
    #[case("x:int=5,y:float,USERNAME", Some(vec![V{ name: "x".into(), r#type: VT::Int(5) }, V{ name: "y".into(), r#type: VT::Float(0.0) },V{ name: "USERNAME".into(), r#type: VT::String("".into()) }]))]
    fn from_config(#[case] input_str: &str, #[case] expected: Option<Vec<Variable>>) {
        assert_eq!(Variable::from_config(input_str), Ok(expected));
    }

    #[rstest]
    #[case(V{ name: "x".into(), r#type: VT::Int(5)}, None, "local x = 5;")]
    #[case(V{ name: "x".into(), r#type: VT::String("".into())}, None, r#"local x = "";"#)]
    #[case(V{ name: "playerName".into(), r#type: VT::String("JD-Plays".into())}, None, r#"local playerName = "JD-Plays";"#)]
    fn command_local_lua_string(
        #[case] variable: Variable,
        #[case] value: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_eq!(variable.command_local_lua(value, "test").as_str(), expected);
    }

    #[rstest]
    #[case(V{ name: "x".into(), r#type: VT::Int(5)}, "", None)]
    #[case(V{ name: "x".into(), r#type: VT::Int(5)}, "HI you awesome", None)]
    #[case(V{ name: "x".into(), r#type: VT::Int(5)}, "HI you awesomex=4", Some(V{ name: "x".into(), r#type: VT::Int(4)}))]
    #[case(V{ name: "x".into(), r#type: VT::Float(5.0)}, "HI you awesomex=4", Some(V{ name: "x".into(), r#type: VT::Float(4.0)}))]
    #[case(V{ name: "x".into(), r#type: VT::String("".to_string())}, r#"HI you awesomex="hello world""#, Some(V{ name: "x".into(), r#type: VT::String("hello world".to_string())}))]
    #[case(V{ name: "x".into(), r#type: VT::String("".to_string())}, r#"HI you awesomex=hello world"#, Some(V{ name: "x".into(), r#type: VT::String("hello".to_string())}))]
    fn variable_from_msg(
        #[case] variable: Variable,
        #[case] msg: &str,
        #[case] expected: Option<Variable>,
    ) {
        assert_eq!(variable.from_message(Some(msg)), expected);
    }
}
