use std::{path::PathBuf, str::FromStr};

use rcon2_lib::command::{LuaFile, RconCommand, Variable};

use rstest::rstest;

#[rstest]
fn file_command_print() {
    let luafile_path = PathBuf::from_str("./tests/fixtures/decon.lua")
        .unwrap()
        .canonicalize()
        .unwrap();
    let luafile = LuaFile::new(luafile_path);
    let variables = Variable::from_config("player_name=JD-Plays").unwrap();
    let lua_command = rcon2_lib::command::RconCommandLua::File(luafile);
    let mut rcon_command = RconCommand {
        prefix: rcon2_lib::command::Prefix::SC,
        lua_command,
        variables,
    };
    let expected = "/silent-command local player_name = \"JD-Plays\";\nlocal radius = 25;\r\nlocal player = game.get_player(player_name);\r\n\r\nif player then;\r\n  local force = player.force;\r\n  for k,v in pairs(player.surface.find_entities_filtered{position=player.position, radius=radius}) do;\r\n    v.order_deconstruction(force);\r\n  end;\r\nend;";
    assert_eq!(rcon_command.command(None, "test").as_str(), expected);
}
