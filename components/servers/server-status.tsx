"use client";

import { cn } from "@/lib/utils";
import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Duration,
  GameServerState,
  GameServerStatus,
  Server,
  ServerCommand,
  ServerCommands,
  SystemTime,
} from "@/types";
import { Biohazard, CircleCheck, PlayIcon, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StopIcon } from "@radix-ui/react-icons";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface GameServerStatusBarProps extends React.ComponentProps<"div"> {
  selectedServer?: Server;
  status: GameServerStatus;
}

export default function GameServerStatusBar({
  className = "w-full",
  selectedServer,
  status,
  ...props
}: GameServerStatusBarProps) {
  const [forceUpdate, setForceUpdate] = useState<number>(0);
  const [serverCommands, setServerCommands] = useState<ServerCommand[]>([]);
  const [ranCommands, setRanCommands] = useState<{
    [Property in ServerCommand]: Date;
  }>({
    start: new Date(0),
    stop: new Date(0),
  });

  useEffect(() => {
    const intervalId = setInterval(setForceUpdate, 200, (i) => i + 1);

    // Clean up the interval when the component unmounts or dependencies change
    return () => clearInterval(intervalId);
  }, []);

  // Check for commands
  useEffect(() => {
    if (selectedServer && selectedServer?.commands) {
      setServerCommands(
        Object.keys(selectedServer.commands).map(
          (cmd) => ServerCommand[cmd as ServerCommand]
        )
      );
    }
  }, [selectedServer]);

  const get_state = () => {
    if (status === undefined || status.game == "NoGame") {
      return GameServerState.Down;
    } else if (status.status.durationSinceLastHeartbeat.secs < 45) {
      return GameServerState.Ok;
    } else if (status.status.durationSinceLastHeartbeat.secs < 60) {
      return GameServerState.Warning;
    } else {
      return GameServerState.Down;
    }
  };

  function handleOnClickServerCommandButton(command: ServerCommand) {
    const new_ran = ranCommands;

    new_ran[command] = new Date();
    setRanCommands(new_ran);
    invoke("run_command_on_server", { server: selectedServer, command })
      .then((status) => {
        console.log(status);
      })
      .catch(console.log);
  }

  let seconds_since_last_heartbeat;
  if (status.game !== "NoGame") {
    seconds_since_last_heartbeat = Math.round(
      seconds_since(systemTimeToDate(status.status.lastHeartbeat!))
    );
  }

  return (
    <div
      className={cn(
        "flex flex-col gap-2 justify-center min-h-[70px] w-full",
        className
      )}
    >
      <div className={cn("flex flex-row gap-2 justify-center")} {...props}>
        <ServerStatusIcon state={get_state()}></ServerStatusIcon>
        <span>Server Name: </span>
        {status.game !== "NoGame" ? status.status.name : "Unknown"}
        {seconds_since_last_heartbeat ? (
          <span className="text-muted-foreground">
            {"- "}
            {seconds_since_last_heartbeat < 10 ? (
              <span>0{seconds_since_last_heartbeat} </span>
            ) : (
              seconds_since_last_heartbeat
            )}{" "}
            secs
          </span>
        ) : (
          <></>
        )}
      </div>
      <div className="flex flex-row gap-2 justify-center">
        {selectedServer &&
        selectedServer.commands &&
        serverCommands.length !== 0 ? (
          Object.values(serverCommands).map((serverCommand) => (
            <ServerRunCommandButton
              selectedServerCommands={selectedServer.commands!}
              key={serverCommand}
              ranCommands={ranCommands}
              commandType={serverCommand}
              onClick={() => {
                handleOnClickServerCommandButton(serverCommand);
              }}
            />
          ))
        ) : (
          <></>
        )}
      </div>
    </div>
  );
}
interface ServerStatusIconProps extends React.ComponentProps<"div"> {
  state: GameServerState;
}
function ServerStatusIcon({ state }: ServerStatusIconProps) {
  let icon;
  switch (state) {
    case GameServerState.Ok:
      icon = <CircleCheck className="text-green-500"></CircleCheck>;
      break;
    case GameServerState.Warning:
      icon = <TriangleAlert className="text-yellow-500"></TriangleAlert>;
      break;
    case GameServerState.Down:
      icon = <Biohazard className="text-red-500"></Biohazard>;
      break;
  }
  return <div>{icon}</div>;
}

export function systemTimeToDate(systemTime: SystemTime) {
  return new Date(
    systemTime.secs_since_epoch * 1000 + systemTime.nanos_since_epoch / 1000000
  );
}

export function millisecondsToDuraion(milliseconds: number): Duration {
  return {
    nanos: Math.round((milliseconds % 1000) * 1000000),
    secs: Math.floor(milliseconds / 1000),
  };
}

interface ServerRunCommandButtonProps extends React.ComponentProps<"button"> {
  commandType: ServerCommand;
  selectedServerCommands: ServerCommands;
  ranCommands: {
    [Property in ServerCommand]: Date;
  };
}
function ServerRunCommandButton({
  commandType,
  disabled,
  ranCommands,
  className,
  selectedServerCommands,
  ...props
}: ServerRunCommandButtonProps) {
  let icon;
  switch (commandType) {
    case ServerCommand.start:
      icon = <PlayIcon className="w-6 h-6" />;
      break;
    case ServerCommand.stop:
      icon = <StopIcon className="w-6 h-6" />;
      break;
  }
  const lastRan = seconds_since(ranCommands[commandType]);
  disabled = disabled || lastRan < 10;
  const command_string = selectedServerCommands[commandType];

  let colors;
  switch (commandType) {
    case ServerCommand.stop:
      colors =
        "hover:bg-destructive/50 active:bg-destructive/90 bg-destructive/25";
      break;

    default:
      colors = "hover:bg-green-700/50 active:bg-green-700 bg-green-700/25";
  }

  return (
    <TooltipProvider delayDuration={1000}>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            disabled={disabled}
            variant="outline"
            className={cn(
              "disabled:bg-muted disabled:text-muted-foreground",
              colors,
              className
            )}
            {...props}
          >
            {icon}
          </Button>
        </TooltipTrigger>
        <TooltipContent className="bg-foreground text-background">
          {command_string}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function seconds_since(then: Date, now: Date = new Date()): number {
  return (+now - +then) / 1000;
}
