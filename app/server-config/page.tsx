"use client";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerConfig from "@/components/server-config";
import ServerSwitcher from "@/components/server-switcher";
import { defaultServers } from "@/lib/utils";
import { Server, Servers } from "@/types";
import { invoke } from "@tauri-apps/api/core";
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
        <ServerSwitcher
          className=""
          selectedServer={selectedServer}
          setSelectedServer={setSelectedServer}
        />
        <MainNav server={selectedServer} />
      </header>
      <ServerConfig
        className="flex-auto h-full"
        showLog={showLogs}
        server={selectedServer}
        setSelectedServer={setSelectedServer}
      />
      {/* </MainContextMenu> */}
    </div>
  );
}
