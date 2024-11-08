"use client";
import { Server, Servers } from "@/types";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";

export function MainContextMenu({
  children,
  servers,
  selectedServer,
  setSelectedServer,
}: {
  children: React.ReactNode;
  selectedServer: Server;
  setSelectedServer: React.Dispatch<React.SetStateAction<Server>>;
  servers: Servers;
}) {
  return (
    <ContextMenu>
      <ContextMenuTrigger className="">{children}</ContextMenuTrigger>
      <ContextMenuContent className="w-64">
        <ContextMenuItem key="Back" inset>
          Back
          <ContextMenuShortcut>⌘[</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem key="Forward" inset disabled>
          Forward
          <ContextMenuShortcut>⌘]</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem key="Reload" inset>
          Reload
          <ContextMenuShortcut>⌘R</ContextMenuShortcut>
        </ContextMenuItem>
        {/*
                <ContextMenuSub key="moreTools">
                    <ContextMenuSubTrigger inset>More Tools</ContextMenuSubTrigger>
                    <ContextMenuSubContent className="w-48">
                        <ContextMenuItem>
                            Save Page As...
                            <ContextMenuShortcut>⇧⌘S</ContextMenuShortcut>
                        </ContextMenuItem>
                        <ContextMenuItem>Create Shortcut...</ContextMenuItem>
                        <ContextMenuItem>Name Window...</ContextMenuItem>
                        <ContextMenuSeparator />
                        <ContextMenuItem>Developer Tools</ContextMenuItem>
                    </ContextMenuSubContent>
                </ContextMenuSub>*/}
        <ContextMenuSeparator />
        {/* <ContextMenuRadioGroup key="servers" value={selectedServer.id}>
          <ContextMenuLabel key="serversTitle" inset>
            Servers
          </ContextMenuLabel>
          {servers.map((group: Servers[number]) => (
            <>
              <ContextMenuSeparator key={"separator" + group.label} />
              <ContextMenuLabel key={group.label} inset>
                {group.label}
              </ContextMenuLabel>
              {group.servers.map((server: Server) => (
                <ContextMenuRadioItem
                  key={server.id}
                  value={server.id}
                  onClick={() => {
                    setSelectedServer(server);
                  }}
                >
                  {server.label}
                </ContextMenuRadioItem>
              ))}
            </>
          ))}
        </ContextMenuRadioGroup> */}
      </ContextMenuContent>
    </ContextMenu>
  );
}
