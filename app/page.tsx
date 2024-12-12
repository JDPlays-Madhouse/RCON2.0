"use client";
import IntegrationStatus from "@/components/integration-status";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerControl from "@/components/server-control";
import ServerDashboard from "@/components/server-dashboard";
import ServerSwitcher from "@/components/server-switcher";
import { Server, Servers } from "@/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import React, { useEffect, useState } from "react";

export default function Home() {
  const [selectedServer, setSelectedServer] = React.useState<Server>();
  const [showLogs, setShowLogs] = useState<boolean>(false);

  useEffect(() => {
    invoke<boolean>("get_config_bool", { key: "show_logs" }).then((show) =>
      setShowLogs(show),
    );
  }, []);
  useEffect(() => {
    invoke<Server>("get_default_server").then((server: Server) => {
      setSelectedServer(server);
    });
  }, []);

  return (
    <div className="flex flex-col h-dvh bg-background">
      {/*<MainContextMenu
        selectedServer={selectedServer}
        setSelectedServer={setSelectedServer}
        servers={servers}
      >*/}
      <header className="px-10 py-5 flex flex-row justify-between w-full max-w-[2560px] mx-auto border border-t-none border-x-none flex-initial">
        <div className="flex flex-row gap-4 items-center">
          <ServerSwitcher
            className=""
            selectedServer={selectedServer}
            setSelectedServer={setSelectedServer}
          />
          <ServerControl selectedServer={selectedServer} />
          <IntegrationStatus />
        </div>
        <MainNav server={selectedServer} />
      </header>
      <ServerDashboard
        className="flex-auto h-full"
        showLog={showLogs}
        selectedServer={selectedServer}
      />
      {/* </MainContextMenu> */}
    </div>
  );
}
