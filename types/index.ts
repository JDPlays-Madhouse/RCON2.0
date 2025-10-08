export enum Page {
  Dashboard,
  ServerSettings,
  TwitchTriggers,
  Settings,
}
export enum ServerCommand {
  start = "start",
  stop = "stop",
}
export type ServerCommands = {
  [Property in ServerCommand]?: string;
};

export type Server = {
  id: string;
  name: string;
  rcon_address: string;
  rcon_port: number;
  password: string;
  game: Game;
  server_name?: string;
  game_address?: string;
  commands?: ServerCommands;
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

export enum CommandPrefixType {
  Custom = "Custom",
  SC = "SC",
  MC = "MC",
  C = "C",
}

export type RconLuaCommand =
  | {
      commandType: "File";
      command: { relative_path: string; command?: string };
    }
  | { commandType: "Inline"; command: string };

export enum LuaCommandType {
  File = "File",
  Inline = "Inline",
}

export type RconCommand = {
  prefix: RconCommandPrefix;
  lua_command: RconLuaCommand;
};

export enum ComparisonOperator {
  Lt = "<",
  Le = "<=",
  Eq = "==",
  Gt = ">",
  Ge = ">=",
  Ne = "!=",
  Any = "Any",
}

export enum TriggerType {
  Chat = "Chat",
  ChatRegex = "Chat Regex",
  ChannelPointRewardRedeemed = "Channel Point Reward Redeemed",
  Subscription = "Subscription",
  GiftSub = "Gift Sub",
}

export type Trigger =
  | { trigger: TriggerType.Chat; data: { pattern: string } }
  | { trigger: TriggerType.ChatRegex; data: { pattern: string } }
  | {
      trigger: TriggerType.ChannelPointRewardRedeemed;
      data: { title: string; id: string };
    }
  | {
      trigger: TriggerType.Subscription;
      data: { tier: string; comaparison_operator: ComparisonOperator };
    }
  | {
      trigger: TriggerType.GiftSub;
      data: {
        tier: string;
        tier_comaparison_operator: ComparisonOperator;
        count: number;
        count_comparison_operator: ComparisonOperator;
      };
    };

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
  | { status: "Unknown" }
  | { status: "NotStarted" };

export type IntegrationError =
  | { error: "Token"; data: TokenError }
  | { error: "NotImplemented"; data: Api }
  | { error: "Unknown" };

/** nanos_since_epoch are nanoseconds since the second.
 *
 * secs_since_epoch are total seconds since epoch.
 *
 * To calculate total time since epoch, sum them.
 */
export type SystemTime = {
  /** nanoseconds since the second. */
  nanos_since_epoch: number;
  /** total seconds since epoch */
  secs_since_epoch: number;
};

export type Duration = {
  nanos: number;
  secs: number;
};

export type GameServerStatus =
  | { game: "NoGame" }
  | {
      game: "Factorio";
      status: {
        name: string;
        gameId: number;
        hostAddress: string;
        lastHeartbeat: SystemTime;
        now: SystemTime;
        durationSinceLastHeartbeat: Duration;
        serverId: string;
      };
    };

export enum GameServerState {
  Ok,
  Warning,
  Down,
}
