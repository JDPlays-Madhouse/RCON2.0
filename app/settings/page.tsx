"use client";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerDashboard from "@/components/server-dashboard";
import ServerSwitcher from "@/components/server-switcher";
import { defaultServers } from "@/lib/utils";
import { Server, Servers } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import React, { useEffect } from "react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export default function Settings() {
  const [selectedServer, setSelectedServer] = React.useState<Server>();
  const servers: Servers = defaultServers();
  const [showLog, setShowLog] = React.useState(true);
  useEffect(() => {
    invoke<Server[]>("list_game_servers").then((list_of_servers: Server[]) => {
      list_of_servers.map((server) =>
        servers[server.game].servers.push(server),
      );
    });
  });
  const handleOnClick = () => {
    invoke("restart").then();
  };
  return (
    <div className="flex flex-col h-dvh bg-background">
      {/*<MainContextMenu
        selectedServer={selectedServer}
        setSelectedServer={setSelectedServer}
        servers={servers}
      >*/}
      <header className="px-10 py-5 flex flex-row justify-between w-full max-w-[2560px] mx-auto border border-t-none border-x-none flex-initial">
        <ServerSwitcher
          className=""
          selectedServer={selectedServer}
          setSelectedServer={setSelectedServer}
        />

        <MainNav server={selectedServer} />
      </header>
      <main className="h-full w-full flex">
        <div className="m-auto">
          {" "}
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="destructive"
                  className="text-7xl h-max pt-6 px-9 rounded-full"
                  onClick={handleOnClick}
                >
                  Restart Application
                </Button>
              </TooltipTrigger>
              <TooltipContent className="text-5xl bg-destructive text-white rounded-full pt-4">
                <p>This will restart the app!!!! Also this button is in beta...</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </main>
      {/* </MainContextMenu> */}
    </div>
  );
}
