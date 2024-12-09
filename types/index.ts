export type Server = {
        id: string;
        name: string;
        address: string;
        port: number;
        password: string;
        game: Game;
};

/// Make sure left side is equal to right.
export enum Game {
        Factorio = "Factorio",
        Satisfactory = "Satisfactory",
}
export type GameString = keyof typeof Game;
export const games: GameString[] = Object.keys(Game) as GameString[];
export type Servers = {
        [Property in GameString]: { label: Game; servers: Server[] };
};
export type Log = {
        uuid: string;
        time: string;
        level: "TRACE" | "DEBUG" | "INFO" | "WARNING" | "ERROR" | "CRITICAL";
        target: string;
        message: string;
};
export type Logs = Log[];
export type LogLevel = Log["level"];
export type LogLevelColors = {
        [Properties in LogLevel]: string;
};

// Commands
export type RconCommandPrefix =
        | {
                prefix: "Custom";
                data: string;
        }
        | { prefix: "SC" }
        | { prefix: "MC" }
        | { prefix: "C" };

export type RconLuaCommand =
        | { commandType: "File"; command: { path: string; command?: string } }
        | { commandType: "Inline"; command: string };

export type RconCommand = {
        prefix: RconCommandPrefix;
        lua_command: RconLuaCommand;
};
export type Command = {
        id: string;
        rcon_lua: RconCommand;
};

export type Api = "Twitch" | "YouTube";
