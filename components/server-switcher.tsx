"use client";
import * as React from "react";
import {
  CaretSortIcon,
  CheckIcon,
  PlusCircledIcon,
} from "@radix-ui/react-icons";

import { cn, defaultServers } from "@/lib/utils";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

import FactorioLogo from "@/app/images/factorio.ico";
import { GameString, Server, Servers, games } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { ServerConfigForm } from "./forms/server-config-form";
import { useEffect, useState } from "react";

type PopoverTriggerProps = React.ComponentPropsWithoutRef<
  typeof PopoverTrigger
>;

interface ServerSwitcherProps extends PopoverTriggerProps {
  selectedServer?: Server;
  setSelectedServer: React.Dispatch<React.SetStateAction<Server | undefined>>;
}

export default function ServerSwitcher({
  className,
  selectedServer,
  setSelectedServer,
}: ServerSwitcherProps) {
  const [open, setOpen] = React.useState(false);
  const [showNewServerDialog, setShowNewServerDialog] = React.useState(false);
  const handleServerSwitch = (server: Server) => {
    invoke("set_default_server", { serverName: server.name });
    setSelectedServer(server);
    setOpen(false);
  };
  const [servers, setServers] = useState(defaultServers());

  useEffect(() => {
    if (open == false) {
      const temp_servers = defaultServers();
      invoke<Server[]>("list_game_servers")
        .then((list_of_servers: Server[]) => {
          list_of_servers.forEach((server) =>
            temp_servers[server.game].servers.push(server),
          );
        })
        .catch((e) => console.log(e));
      setServers(temp_servers);
    }
  }, [open]);

  return (
    <Dialog open={showNewServerDialog} onOpenChange={setShowNewServerDialog}>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            aria-label="Select a server"
            className={cn("w-[200px] justify-between", className)}
          >
            <Avatar className="mr-2 h-5 w-5">
              <AvatarImage src={FactorioLogo.src} alt={selectedServer?.name} />
              <AvatarFallback>Fa</AvatarFallback>
            </Avatar>
            {selectedServer ? selectedServer.name : "Select a Server"}
            <CaretSortIcon className="ml-auto h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[200px] p-0">
          <Command>
            <CommandInput placeholder="Search server..." />
            <CommandList>
              <CommandEmpty>No server found.</CommandEmpty>
              {games.map((game: GameString) => {
                if (servers[game].servers.length === 0) {
                  return;
                }
                return (
                  <CommandGroup key={game} heading={game}>
                    {servers[game].servers.map((server: Server) => (
                      <CommandItem
                        key={server.id}
                        onSelect={() => {
                          handleServerSwitch(server);
                        }}
                        className="text-sm"
                      >
                        <Avatar className="mr-2 h-5 w-5">
                          <>
                            <AvatarImage
                              src={FactorioLogo.src}
                              alt={server.name}
                            />
                            <AvatarFallback>SC</AvatarFallback>
                          </>
                          <AvatarFallback>SC</AvatarFallback>
                        </Avatar>
                        {server.name}
                        <CheckIcon
                          className={cn(
                            "ml-auto h-4 w-4",
                            selectedServer?.id === server.id
                              ? "opacity-100"
                              : "opacity-0",
                          )}
                        />
                      </CommandItem>
                    ))}
                  </CommandGroup>
                );
              })}
            </CommandList>
            <CommandSeparator />
            <CommandList>
              <CommandGroup>
                <DialogTrigger asChild>
                  <CommandItem
                    onSelect={() => {
                      setOpen(false);
                      setShowNewServerDialog(true);
                    }}
                  >
                    <PlusCircledIcon className="mr-2 h-5 w-5" />
                    Create Server
                  </CommandItem>
                </DialogTrigger>
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create server</DialogTitle>
          <DialogDescription>Add a new rcon server.</DialogDescription>
        </DialogHeader>
        <ServerConfigForm
          onClickReset={() => setShowNewServerDialog(false)}
          onClickSubmit={() => {
            setShowNewServerDialog(false);
          }}
          setSelectedServer={setSelectedServer}
        />
      </DialogContent>
    </Dialog>
  );
}
