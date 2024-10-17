"use client";
import { DarkModeToggle } from "@/components/dark-mode-button";
import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerSwitcher from "@/components/server-switcher";
import React from "react";
import { Server } from "../page";
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

export default function Settings() {
  const [selectedServer, setSelectedServer] = React.useState<Server>(
    servers[0].servers[0],
  );

  return (
    <div className="h-full">
      <MainContextMenu
        selectedServer={selectedServer}
        setSelectedServer={setSelectedServer}
        servers={servers}
      >
        <header className="p-10 flex flex-row justify-between w-full max-w-[2560px] mx-auto">
          <ServerSwitcher
            className=""
            selectedServer={selectedServer}
            setSelectedServer={setSelectedServer}
            servers={servers}
          />
          <MainNav server={selectedServer} />
        </header>
        <main className="h-full w-full">
          Settings
          <div>Hello</div>
        </main>
      </MainContextMenu>
    </div>
  );
}
