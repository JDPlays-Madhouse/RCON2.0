"use client";

import { cn } from "@/lib/utils";
import { GameServerStatus, Server, ServerStatus } from "@/types";
import { Pause, Play, Square, StopCircle } from "lucide-react";
import React, { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Channel, invoke } from "@tauri-apps/api/core";
import { StopIcon } from "@radix-ui/react-icons";
import GameServerStatusBar, {
  millisecondsToDuraion,
  systemTimeToDate,
} from "@/components/servers/server-status";

export const NoGame: GameServerStatus = { game: "NoGame" };
interface ServerControlProps extends React.ComponentProps<"div"> {
  selectedServer?: Server;
}

export default function ServerControl({
  selectedServer,
  className,
  ...props
}: ServerControlProps) {
  const [gameId, setGameId] = useState<number>(-1);
  const [channel, setChannel] = useState<Channel<ServerStatus>>();
  const [autoConnect, setAutoConnect] = useState<boolean>(false);
  const [status, setStatus] = useState<ServerStatus>({
    event: "disconnected",
    data: { server: selectedServer },
  });
  const [gameStatus, setGameStatus] = useState<GameServerStatus>(NoGame);
  const [manuallyStopped, setManuallyStopped] = useState<boolean>(false);

  const [forceUpdate, setForceUpdate] = useState<number>(0);
  const connected = () => status.event === "connected";
  const connecting = () => status.event === "connecting";
  const checking = () => status.event === "checking";
  const hasGameServerDetails = () =>
    selectedServer?.server_name !== undefined ||
    selectedServer?.game_address !== undefined;
  // console.log({ status, connected: connected() });
  // BUG: #10 Improve autostart logic.
  useEffect(() => {
    invoke<boolean>("get_config_bool", {
      key: "servers.autostart",
    }).then((connect) => {
      setAutoConnect(connect);
    });
  }, []);

  useEffect(() => {
    console.log({ gameStatus, autoConnect, manuallyStopped });
    if (gameId === -1) {
      return;
    } else if (
      gameStatus.game !== "NoGame" &&
      autoConnect &&
      !manuallyStopped
    ) {
      console.log("Reconnecting server");
      handleServerDisconnect();
      handleServerConnect();
      setGameId(gameStatus.status.gameId);
    }
  }, [gameId, manuallyStopped, autoConnect]);

  useEffect(() => {
    console.log({
      connected: connected(),
      autoConnect,
      connecting: connecting(),
      manuallyStopped,
    });
    if (selectedServer && channel) {
      if (!connected() && autoConnect && !connecting() && !manuallyStopped) {
        handleServerConnect();
      } else {
        handleServerCheck();
      }
    }
  }, [selectedServer, autoConnect, channel, manuallyStopped]);

  useEffect(() => {
    const channel = new Channel<ServerStatus>();
    channel.onmessage = (message) => {
      // console.log("New Message: ", message);
      if (message.data.server?.id === selectedServer?.id) {
        setStatus(message);
      }
    };
    setChannel(channel);
  }, []); // Needs empty array..

  function handleOnClick() {
    if (["disconnected", "error"].includes(status.event)) {
      handleServerConnect();
      setManuallyStopped(false);
      // handleGameServerStatusUpdate();
      handleSetGameServerStatus();
    } else if (status.event == "connected") {
      setManuallyStopped(true);
      handleServerDisconnect();
      setGameId(-1);
    } else {
      handleServerCheck();
    }
  }

  function handleServerConnect() {
    if (channel && selectedServer && !connecting()) {
      setStatus({ event: "connecting", data: { server: selectedServer } });
      invoke<ServerStatus>("connect_to_server", {
        server: selectedServer,
        channel,
      })
        .then((status) => {
          console.log(
            `Connected to server with channel: ${status.data.server?.name}`
          );
          setStatus(status);
        })
        .catch((e: ServerStatus) => {
          console.log(`Error: ${e.event === "error" ? e.data.msg : "unknown"}`);
          setStatus(e);
        });
    }
  }

  function handleServerCheck() {
    if (selectedServer) {
      invoke<ServerStatus>("check_connection", { server: selectedServer })
        .then((s) => {
          setStatus(s);
        })
        .catch(console.error);
    }
  }
  function handleServerDisconnect() {
    invoke<ServerStatus>("disconnect_connection", {
      server: selectedServer,
    })
      .then((s) => {
        setStatus(s);
      })
      .catch(console.error);
  }

  // function handleGameServerStatusUpdate() {
  //   if (selectedServer) {
  //     invoke<GameServerStatus>("latest_game_server_status", {
  //       server: selectedServer,
  //     })
  //       .then((s) => {
  //         console.log(s);
  //       })
  //       .catch(console.error);
  //   }
  // }
  useEffect(() => {
    const intervalId = setInterval(updateSecsSince, 200);

    // Clean up the interval when the component unmounts or dependencies change
    return () => clearInterval(intervalId);
  }, []);
  function updateSecsSince() {
    setForceUpdate((i) => i + 1);
    if (gameStatus.game === "NoGame") {
      return;
    } else {
      setGameStatus((gameStatus) => {
        if (gameStatus.game === "NoGame") {
          return gameStatus;
        } else {
          const now = new Date();
          gameStatus.status.durationSinceLastHeartbeat = millisecondsToDuraion(
            +now - +systemTimeToDate(gameStatus.status.lastHeartbeat)
          );
        }
        return gameStatus;
      });
    }
  }
  function updateSecsSinceWithStatus(
    status: GameServerStatus
  ): GameServerStatus {
    if (status.game === "NoGame") {
      return NoGame;
    } else {
      const now = new Date();
      status.status.durationSinceLastHeartbeat = millisecondsToDuraion(
        +now - +systemTimeToDate(status.status.lastHeartbeat)
      );
      return status;
    }
  }
  useEffect(() => {
    const intervalId = setInterval(handleSetGameServerStatus, 1000);

    // Clean up the interval when the component unmounts or dependencies change
    return () => clearInterval(intervalId);
  }, [selectedServer]);

  function handleSetGameServerStatus() {
    invoke<GameServerStatus>("latest_game_server_status", {
      server: selectedServer,
    })
      .then((new_status) => {
        if (
          new_status.game === "NoGame" ||
          new_status.status.serverId !== selectedServer?.id
        ) {
          setGameStatus(NoGame);
          return;
        } else {
          if (gameId === -1 || gameId !== new_status.status.gameId) {
            setGameId(new_status.status.gameId);
          }
          new_status = updateSecsSinceWithStatus(new_status);
          setGameStatus(new_status);
        }
      })
      .catch((e) => {
        console.log(e);
        updateSecsSince();
      });
  }
  const handleMessage = () => {
    switch (status.event) {
      case "connected":
        return `Connected to ${status.data.server.name}`;
      case "connecting":
        return `Connecting to ${status.data.server.name}...`;
      case "checking":
        return `Checking ${status.data.server.name}...`;
      case "error":
        return `Rcon Connection Error: Server Unreachable`;
      case "disconnected":
        if (status.data.server) {
          return `Disconnected from ${status.data.server.name}`;
        } else {
          return "Disconnected";
        }
    }
  };

  if (!selectedServer) {
    return <></>;
  }

  return (
    <div
      className={cn("flex flex-row gap-4 m-0 w-full justify-center", className)}
      {...props}
    >
      <div className=" flex-1 flex flex-row align-center items-center gap-2 justify-center">
        <Button
          variant={connected() || connecting() ? "default" : "destructive"}
          size="icon"
          onClick={handleOnClick}
          disabled={["connecting", "checking"].includes(status.event)}
        >
          {connected() || connecting() ? (
            <StopIcon className="w-6 h-6" />
          ) : (
            <Play />
          )}
        </Button>
        <span onClick={handleServerCheck}>{handleMessage()}</span>
      </div>
      {hasGameServerDetails() ? (
        <GameServerStatusBar
          selectedServer={selectedServer}
          className="flex-2"
          status={gameStatus}
        />
      ) : (
        <></>
      )}
    </div>
  );
}
