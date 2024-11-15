"use client";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerDashboard from "@/components/server-dashboard";
import ServerSwitcher from "@/components/server-switcher";
import { defaultServers } from "@/lib/utils";
import { Server, Servers } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import React, { useEffect } from "react";

export default function Home() {
  const [selectedServer, setSelectedServer] = React.useState<Server>();
  const servers: Servers = defaultServers();
  const [showLog, setShowLog] = React.useState(true);
  useEffect(() => {
    invoke<Server>("get_default_server").then((server: Server) => {
      setSelectedServer(server);
    });
  }, []);
  useEffect(() => {
    invoke<Server[]>("list_game_servers").then((list_of_servers: Server[]) => {
      list_of_servers.map((server) =>
        servers[server.game].servers.push(server)
      );
    });
  });

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
          servers={servers}
        />
        <MainNav server={selectedServer} />
      </header>
      <ServerDashboard className="flex-auto h-full" showLog={showLog} />
      {/* </MainContextMenu> */}
    </div>
  );
}
