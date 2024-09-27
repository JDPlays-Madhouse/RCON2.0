use std::path::PathBuf;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub enum CommandType {
    ChannelPoints(String),
    Chat,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RconCommandPrefix {
    Custom(String),
    SC,
    C,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RconLuaType {
    File(PathBuf),
    Inline(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CommandValue<T> {
    pub data: T,
    pub r#type: ValueType,
}

#[allow(dead_code)]
impl<T> CommandValue<T> {
    pub fn new(data: T, r#type: ValueType) -> Self {
        Self { data, r#type }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueType {
    Bool,
    Int,
    String,
    Float,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RconCommand<T> {
    pub prefix: RconCommandPrefix,
    pub lua_type: RconLuaType,
    pub lua: Option<&'static str>,
    pub default_values: Option<CommandValue<T>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Command<T> {
    id: Uuid,
    pub variant: CommandType,
    pub rcon_lua: RconCommand<T>,
}

#[allow(dead_code)]
impl<T> Command<T> {
    pub fn new(variant: CommandType, rcon_lua: RconCommand<T>) -> Self {
        let id = Uuid::now_v7();
        Self {
            id,
            variant,
            rcon_lua,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::integer(8, ValueType::Int)]
    #[case::float(8.1, ValueType::Float)]
    #[case::bool(true, ValueType::Bool)]
    #[case::string("test", ValueType::String)]
    fn command_value_data<T>(#[case] input: T, #[case] input_type: ValueType)
    where
        T: std::fmt::Debug + std::cmp::PartialEq + std::clone::Clone,
    {
        let t = CommandValue::new(input.clone(), input_type);
        assert_eq!(t.data, input);
    }

    #[rstest]
    #[case::integer(8, ValueType::Int)]
    #[case::float(8.1, ValueType::Float)]
    #[case::bool(true, ValueType::Bool)]
    #[case::string("test", ValueType::String)]
    fn command_value_type<T>(#[case] input: T, #[case] input_type: ValueType)
    where
        T: std::fmt::Debug + std::cmp::PartialEq + std::clone::Clone,
    {
        let t = CommandValue::new(input.clone(), input_type);
        assert_eq!(t.r#type, input_type);
    }
}
