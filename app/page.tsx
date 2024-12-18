"use client";
import IntegrationStatus from "@/components/integration-status";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerConfig from "@/components/servers/server-config";
import ServerControl from "@/components/servers/server-control";
import ServerDashboard from "@/components/servers/server-dashboard";
import LogArea from "@/components/servers/server-log";
import ServerSwitcher from "@/components/servers/server-switcher";
import Settings from "@/components/settings/settings";
import TwitchTrigger from "@/components/twitch/twitch-triggers";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  type ImperativePanelHandle,
} from "@/components/ui/resizable";
import { Page, Server, Servers } from "@/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import React, { useEffect, useRef, useState } from "react";

export default function Home() {
  const [selectedServer, setSelectedServer] = React.useState<Server>();
  const [showLogs, setShowLogs] = useState<boolean>(false);
  const [page, setPage] = useState<Page>(Page.Dashboard);
  const logRef = useRef<ImperativePanelHandle>(null);

  useEffect(() => {
    if (logRef && logRef.current && showLogs) {
      logRef.current.expand();
    } else if (logRef && logRef.current && showLogs === false) {
      logRef.current.collapse();
    }
  }, [showLogs]);

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
        <MainNav  page={page} setPage={setPage} />
      </header>
      <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
        <ResizablePanel defaultSize={70}>
          <Router page={page} selectedServer={selectedServer} setSelectedServer={setSelectedServer}/>
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
      {/* </MainContextMenu> */}
    </div>
  );
}

type RouterProps = {
  page: Page;
  selectedServer?: Server;
  setSelectedServer: React.Dispatch<React.SetStateAction<Server | undefined>>
};

function Router({ page, selectedServer, setSelectedServer }: RouterProps) {
  switch (page) {
    case Page.Dashboard:
      return (
        <ServerDashboard
          className="flex-auto h-full"
          showLog={true}
          selectedServer={selectedServer}
        />
      );
    case Page.ServerSettings: 
      return (
      <ServerConfig
        className=""
        server={selectedServer}
        setSelectedServer={setSelectedServer}
      />
      )
    case Page.TwitchTriggers:
      return (
      <TwitchTrigger/>
      )
    case Page.Settings:
      return <Settings />
  }
}
