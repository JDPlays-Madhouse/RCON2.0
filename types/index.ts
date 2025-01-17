export enum Page {
  Dashboard,
  ServerSettings,
  TwitchTriggers,
  Settings,
}

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

export type Trigger =
  | { trigger: "Chat"; data: { pattern: string } }
  | { trigger: "ChatRegex"; data: { pattern: string } }
  | {
    trigger: "ChannelPointRewardRedeemed";
    data: { title: string; id: string };
  }
  | { trigger: "Subscription" };

export type GameServerTrigger = {
  server: Server;
  trigger: Trigger;
  enabled: boolean;
};

export type Command = {
  name: string;
  rcon_lua: RconCommand;
  server_triggers: GameServerTrigger[];
};

/// Make sure left side is equal to right.
export enum Api {
  Twitch = "Twitch",
  YouTube = "YouTube",
  Patreon = "Patreon",
  StreamLabs = "StreamLabs",
}
export type ApiString = keyof typeof Api;

export const apis: ApiString[] = Object.keys(Api) as ApiString[];

export type TokenError =
  | "TokenElapsed"
  | "InvalidScopes"
  | "InvalidToken"
  | "UnknownError"
  | "NotAuthorized";

export type ServerStatus =
  | {
    event: "connecting";
    data: { server: Server };
  }
  | { event: "connected"; data: { server: Server } }
  | { event: "checking"; data: { server: Server } }
  | { event: "error"; data: { msg: string; server: Server } }
  | {
    event: "disconnected";
    data: { server?: Server };
  };

export type IntegrationStatusMap = { [Property in Api]: IntegrationStatus };

export type IntegrationStatus =
  | { status: "Connected"; api: { api: Api; expires_at?: number } }
  | { status: "Disconnected"; api: Api }
  | { status: "Connecting"; api: Api }
  | { status: "Error"; api: { api: Api; error: IntegrationError } }
  | { status: "Unknown" };

export type IntegrationError =
  | { error: "Token"; data: TokenError }
  | { error: "NotImplemented"; data: Api }
  | { error: "Unknown" };
