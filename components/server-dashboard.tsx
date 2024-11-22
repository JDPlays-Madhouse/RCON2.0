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
import { Command, CommandType, RconCommand, Server } from "@/types";

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
  useEffect(() => {
    const variant: CommandType = {
      type: "Chat",
    };
    const rconLua: RconCommand = {
      prefix: {
        prefix: "SC",
      },
      lua_command: {
        commandType: "Inline",
        command: "game.print('Hello world', {color= {b=0.5}})",
      },
    };
    invoke<Command>("create_command", { variant, rconLua }).then((c) => {
      setCommand(c);
    });
  });
  function handleOnClick() {
    if (command && selectedServer) {
      invoke<string>("send_command_to_server", {
        server: selectedServer,
        command,
      })
        .then(console.log)
        .catch(console.log);
    }
  }

  return (
    <div className={cn("", className)} {...props}>
      <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
        <ResizablePanel defaultSize={70}>
          <div className="flex h-full items-center justify-center p-6 my-auto">
            <div className="font-semibold">Dashboard</div>
            <div onClick={handleOnClick}> Send Hello world</div>
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
