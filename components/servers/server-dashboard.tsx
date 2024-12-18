"use client";
import { cn } from "@/lib/utils";

import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Command, GameServerTrigger, RconCommand, Server } from "@/types";
import DashboardTable, { column, CommandTrigger } from "@/components/dashboard-table";

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
  const [commands, setCommands] = useState<CommandTrigger[]>([]);

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
    <div
      className={cn(
        "flex flex-col h-full items-center justify-start p-6 my-auto gap-2",
        className,
      )}
      {...props}
    >
      <div className="font-semibold">Dashboard</div>
      <DashboardTable data={commands} columns={column} />
    </div>
  );
}
