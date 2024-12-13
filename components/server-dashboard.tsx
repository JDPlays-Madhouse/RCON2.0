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
import { Command, GameServerTrigger, RconCommand, Server } from "@/types";
import DashboardTable, { column, CommandTrigger } from "./dashboard-table";

interface ServerDashBoardProps extends React.ComponentProps<"div"> {
  showLog: boolean;
  selectedServer?: Server;
}
type TriggerCommand = [GameServerTrigger, Command];
export default function ServerDashboard({
  selectedServer,
  className,
  showLog,
  ...props
}: ServerDashBoardProps) {
  const logRef = useRef<ImperativePanelHandle>(null);
  const [commands, setCommands] = useState<CommandTrigger[]>([]);

  useEffect(() => {
    if (logRef && logRef.current && showLog) {
      logRef.current.expand();
    } else if (logRef && logRef.current && showLog === false) {
      logRef.current.collapse();
    }
  }, [showLog]);

  useEffect(() => {
    if (selectedServer) {
      invoke<TriggerCommand[]>("server_trigger_commands", {
        server: selectedServer,
      }).then((commandtriggers) => {
        console.log({ commandtriggers });
        const ret_vec: CommandTrigger[] = [];
        for (const [trig, comm] of commandtriggers) {
          ret_vec.push({ serverTrigger: trig, command: comm });
        }
        setCommands(ret_vec);
      });
    }
  }, [selectedServer]);

  return (
    <div className={cn("", className)} {...props}>
      <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
        <ResizablePanel defaultSize={70}>
          <div className="flex flex-col h-full items-center justify-start p-6 my-auto gap-2">
            <div className="font-semibold">Dashboard</div>
            <DashboardTable data={commands} columns={column} />
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
