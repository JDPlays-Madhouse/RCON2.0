"use client";
import { cn } from "@/lib/utils";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  type ImperativePanelHandle,
} from "@/components/ui/resizable";
import { useEffect, useRef, useState } from "react";
import LogArea from "./server-log";
import { invoke } from "@tauri-apps/api/core";
import { Api, Command, RconCommand, Server } from "@/types";
import { Button } from "./ui/button";

interface ServerDashBoardProps extends React.ComponentProps<"div"> {
  showLog: boolean;
  selectedServer?: Server;
}

export default function ServerDashboard({
  selectedServer,
  className,
  showLog,
  ...props
}: ServerDashBoardProps) {
  const logRef = useRef<ImperativePanelHandle>(null);
  const [command, setCommand] = useState<Command>();

  useEffect(() => {
    if (logRef && logRef.current && showLog) {
      logRef.current.expand();
    } else if (logRef && logRef.current && showLog === false) {
      logRef.current.collapse();
    }
  }, [showLog]);

  const rconLua: RconCommand = {
    prefix: {
      prefix: "SC",
    },
    lua_command: {
      commandType: "Inline",
      command: "print('hello world')",
    },
  };
  const name: string = "Hello World";
  const commandProps = { name, rconLua };
  useEffect(() => {
    invoke<Command>("create_command", commandProps)
      .then((c) => {
        console.log(c);
        setCommand(c);
      })
      .catch(console.log);
    console.log("after create command");
  }, []); // Needed otherwise ui freezes ...

  function handleOnClick() {
    console.log("HandleOnClick");
    if (command && selectedServer) {
      invoke<string>("send_command_to_server", {
        server: selectedServer,
        command,
      })
        .then(console.log)
        .catch(console.log);
    }
  }

  function handleConnectToIntegration() {
    const api: Api = "Twitch";
    invoke<Api>("connect_to_integration", { api }).then((value) => {
      console.log({ value, type: typeof value });
    });
  }
  function handleListOfIntegrations() {
    invoke<Api>("list_of_integrations").then((value) => {
      console.log({ value, type: typeof value });
    });
  }
  function handleUpdateConfig() {
    invoke("update_config")
  }

  return (
    <div className={cn("", className)} {...props}>
      <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
        <ResizablePanel defaultSize={70}>
          <div className="flex flex-col h-full items-center justify-center p-6 my-auto gap-2">
            <div className="font-semibold">Dashboard</div>
            <Button onClick={handleOnClick} variant="secondary">
              Send:{" "}
              {command?.rcon_lua?.lua_command.commandType == "Inline"
                ? command?.rcon_lua?.lua_command.command
                : command?.rcon_lua?.lua_command.command.command}
            </Button>
            <Button onClick={handleConnectToIntegration} variant="secondary">
              Connect to Twitch
            </Button>
            <Button onClick={handleListOfIntegrations} variant="secondary">
              List of Integrations
            </Button>
            <Button onClick={handleUpdateConfig} variant="secondary">
              Update Config
            </Button>
          </div>
        </ResizablePanel>
        <ResizableHandle withHandle />

        <ResizablePanel
          ref={logRef}
          defaultSize={30}
          collapsible={true}
          className="flex"
        >
          <LogArea />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
