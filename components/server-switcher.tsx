import * as React from "react";
import {
  CaretSortIcon,
  CheckIcon,
  PlusCircledIcon,
} from "@radix-ui/react-icons";

import { cn } from "@/lib/utils";
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
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import FactorioLogo from "@/app/images/factorio.ico";
import { GameString, Server, Servers, games } from "@/types";

type PopoverTriggerProps = React.ComponentPropsWithoutRef<
  typeof PopoverTrigger
>;

interface ServerSwitcherProps extends PopoverTriggerProps {
  selectedServer?: Server;
  setSelectedServer: React.Dispatch<React.SetStateAction<Server | undefined>>;
  servers: Servers;
}

export default function ServerSwitcher({
  className,
  selectedServer,
  setSelectedServer,
  servers,
}: ServerSwitcherProps) {
  const [open, setOpen] = React.useState(false);
  const [showNewServerDialog, setShowNewServerDialog] = React.useState(false);

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
              {games.map((game: GameString) => (
                <CommandGroup key={game} heading={game}>
                  {servers[game].servers.map((server: Server) => (
                    <CommandItem
                      key={server.id}
                      onSelect={() => {
                        setSelectedServer(server);
                        setOpen(false);
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
              ))}
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
          <DialogDescription>Add a new server.</DialogDescription>
        </DialogHeader>
        <div>
          <div className="space-y-4 py-2 pb-4">
            <div className="space-y-2">
              <Label htmlFor="name">Server name</Label>
              <Input id="name" placeholder="Local Server" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="plan">Game</Label>
              <Select>
                <SelectTrigger>
                  <SelectValue placeholder="Select a game" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Factorio">
                    <span className="font-medium">Factorio</span>
                  </SelectItem>
                  <SelectItem value="Satisfactory">
                    <span className="font-medium">Satisfactory</span>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setShowNewServerDialog(false)}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            onClick={() => {
              setShowNewServerDialog(false);
            }}
          >
            Continue
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
