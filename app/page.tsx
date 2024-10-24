"use client";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerDashboard from "@/components/server-dashboard";
import ServerSwitcher from "@/components/server-switcher";
import React from "react";
const servers = [
  {
    label: "Factorio",
    servers: [
      {
        label: "Local Server",
        id: "factorio1",
        game: "Factorio",
      },
      {
        label: "Bilbo's Server",
        id: "factorio2",
        game: "Factorio",
      },
    ],
  },
  {
    label: "Satisfactory",
    servers: [
      {
        label: "Bilbo's Sat Server",
        id: "sat2",
        game: "Satisfactory",
      },
    ],
  },
];
function findServer(id: string) {
  for (const game of servers) {
    for (const server of game.servers) {
      if (id === server.id) {
        return server;
      }
    }
  }
  return undefined;
}
export type Servers = typeof servers;
export type Server = (typeof servers)[number]["servers"][number];
export default function Home() {
  const [selectedServer, setSelectedServer] = React.useState<Server>(
    servers[0].servers[0],
  );
  const [showLog, setShowLog] = React.useState(true);

  return (
    <div className="flex flex-col h-dvh ">
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
      <ServerDashboard
        className="flex-auto h-full"
        showLog={showLog}
      // setShowLog={setShowLog}
      />
      {/* </MainContextMenu> */}
    </div>
  );
}
