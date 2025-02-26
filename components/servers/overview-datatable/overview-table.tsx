"use client";

import { Command, Game, GameServerTrigger, Server, Trigger, TriggerType } from "@/types";
import {
  ColumnDef,
  ColumnFiltersState,
  Row,
  SortingState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { Check, Cross, MoreHorizontal, Plus, X } from "lucide-react";
import * as React from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ArrowUpDown } from "lucide-react";
import { Input } from "@/components/ui/input";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DataTablePagination } from "@/components/datatables/pagination";
import EditableCheckBox from "@/components/datatables/EditableCheckBox";
import ServerFormDialog from "../server-form-dialog";
import { TriggerForm } from "@/components/forms/trigger-form";

export type CommandTrigger = {
  serverTrigger: GameServerTrigger;
  command?: Command;
};

export const columns: ColumnDef<CommandTrigger>[] = [
  {
    id: "enabled",
    accessorKey: "serverTrigger.enabled",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => {
            column.toggleSorting(column.getIsSorted() === "asc");
          }}
        >
          Enabled
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: EditableCheckBox,
  },
  {
    id: "server",
    accessorKey: "serverTrigger.server.name",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Server
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "triggerType",
    accessorKey: "serverTrigger.trigger.trigger",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Trigger Type <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "trigger",
    accessorKey: "serverTrigger.trigger",
    header: "Trigger",
    cell: ({ cell }) => {
      const trigger: Trigger = cell.getValue() as Trigger;
      switch (trigger.trigger) {
        case TriggerType.Chat:
          return (
            <CellToolTip
              helper="The pattern that will trigger the command."
              className="cursor-text"
            >
              {trigger.data.pattern}
            </CellToolTip>
          );
        case TriggerType.ChatRegex:
          return (
            <CellToolTip
              helper="The pattern that will trigger the command."
              className="cursor-text"
            >
              {trigger.data.pattern}
            </CellToolTip>
          );
        case TriggerType.ChannelPointRewardRedeemed:
          return (
            <CellToolTip
              helper="{Channel Points Reward Name} - {Twitch ID for reward}"
              className="cursor-text"
            >
              {`${trigger.data.title} - ${trigger.data.id}`}
            </CellToolTip>
          );
        case TriggerType.Subscription:
          return (
            <CellToolTip helper="When a user subscribes to your channel">
              Subscription
            </CellToolTip>
          );
      }
    },
  },
  {
    id: "commandName",
    accessorKey: "command.name",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Command
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      const commandTrigger: CommandTrigger = row.original;

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-8 w-8 p-0">
              <span className="sr-only">Open menu</span>
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuLabel>Actions</DropdownMenuLabel>
            <DropdownMenuItem
              onClick={() => {
                if (commandTrigger.command) {
                  navigator.clipboard.writeText(commandTrigger.command.name);
                }
              }}
            >
              Copy Command Name
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];

interface DataTableProps {
  selectedServer?: Server;
}
type TriggerCommand = [GameServerTrigger, Command];

export enum FormOpen {
  None,
  Trigger,
  Command
}

export default function DashboardTable({ selectedServer }: DataTableProps) {
  const [data, setData] = useState<CommandTrigger[]>([]);
  const [count, setCount] = useState(0);
  const [formOpen, setFormOpen] = useState(FormOpen.None);
  const [thisServerOnly, setThisServerOnly] = useState(false);
  useEffect(() => {
    handleUpdateData()
  }, [selectedServer]);


  const handleUpdateData = () => {
    setCount((c) => c + 1);
    if (selectedServer) {
      invoke<TriggerCommand[]>("server_trigger_commands", {
        server: selectedServer,
      }).then((commandtriggers) => {
        const ret_vec: CommandTrigger[] = [];
        for (const [trig, comm] of commandtriggers) {
          ret_vec.push({ serverTrigger: trig, command: comm });
        }
        setData(ret_vec);
      });
    }
  };

  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    [],
  );
  function updateNestedValue<tValue, tObj>(
    keyArr: string[],
    value: tValue,
    obj: { [index: string]: tObj | tValue }
  ): { [index: string]: tObj | tValue } {
    const key = keyArr.shift();
    if (!key) {
      return {}
    }
    else if (keyArr.length === 0 && key) {
      obj[key] = value;
      return obj;
    } else {
      // @ts-expect-error incorrect types
      obj[key] = updateNestedValue(keyArr, value, obj[key])
      return obj
    }
  };
  const [sorting, setSorting] = React.useState<SortingState>([]);
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    onColumnFiltersChange: setColumnFilters,
    getFilteredRowModel: getFilteredRowModel(),
    state: {
      sorting,
      columnFilters,
    },
    meta: {
      updateData: function f<coloumType>(
        rowIndex: number,
        columnId: string,
        value: coloumType,
        commandName: string
      ) {
        const newData = data.map((row, index) => {
          if (index === rowIndex) {
            return updateNestedValue(columnId.split("."), value, data[rowIndex]);
          }
          else { return row; }
        });

        const serverTriggers: GameServerTrigger[] = [];
        newData.forEach((v) => {
          // @ts-expect-error unsure
          if (v.command?.name === commandName) {
            // @ts-expect-error unsure
            serverTriggers.push(v.serverTrigger)
          }
        });

        invoke("update_server_trigger", { commandName, serverTriggers })
        // @ts-expect-error unsure
        setData(newData);
      },
    },
  });

  useEffect(() => {
    table.getColumn("server")?.setFilterValue(thisServerOnly ? selectedServer?.name : undefined)
  }, [thisServerOnly])

  return (
    <div className="w-full">
      <div className="flex items-center py-4 justify-between flex-row w-full">
        <div className="flex flex-1 flex-row gap-2">
          <Input
            placeholder="Filter Command..."
            value={
              (table.getColumn("commandName")?.getFilterValue() as string) ?? ""
            }
            onChange={(event) =>
              table.getColumn("commandName")?.setFilterValue(event.target.value)
            }
            className="max-w-sm"
          />
          <Button
            className="flex flex-row text-foreground items-center justify-center gap-1"
            variant="secondary"
            onClick={() => {
              setThisServerOnly(!thisServerOnly)
            }}>{thisServerOnly ? <Check className="" /> : <X />} <span>This server only.</span></Button>
        </div>
        <div className="flex flex-1 flex-row gap-2 justify-end">
          <ServerFormDialog formTitle="New Trigger" form={<TriggerForm server={selectedServer as Server} />}>
            <Button
              variant="secondary"
              className="flex flex-row text-secondary-foreground text-base justify-center gap-1 pl-[11px]"
              onClick={() => setFormOpen((prev) => prev === FormOpen.None ? FormOpen.Trigger : FormOpen.None)}
              disabled
            >
              <Plus />
              <span className="pt-1">Trigger</span>
            </Button>
          </ServerFormDialog>
          <Button
            variant="secondary"
            className="flex flex-row text-secondary-foreground text-base justify-center gap-1 pl-[11px]"
            onClick={() => setFormOpen((prev) => prev === FormOpen.None ? FormOpen.Command : FormOpen.None)}
            disabled
          >
            <Plus />
            <span className="pt-1">Command</span>
          </Button>
        </div>
      </div>
      <div className="rounded-md border w-full">
        <Table>
          <TableHeader className="text-center justify-center" >
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id} className="text-center" colSpan={header.colSpan}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && "selected"}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="text-center">
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
        <div className="flex items-center justify-end space-x-2 py-4">
          <DataTablePagination table={table} />{" "}
        </div>
      </div>
    </div>
  );
}

function CellToolTip({
  helper,
  children,
  className = "",
}: {
  helper: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger className={className}>{children}</TooltipTrigger>
        <TooltipContent className={cn("mt-2 text-s bg-secondary")}>
          <div>{helper}</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

// function handleOnClick() {
//    console.log("HandleOnClick");
//    if (command && selectedServer) {
//      invoke<string>("send_command_to_server", {
//        server: selectedServer,
//        command,
//      })
//        .then(console.log)
//        .catch(console.log);
//    }
//  }
//  <Button onClick={handleOnClick} variant="secondary">
//   Send:{" "}
//   {command?.rcon_lua?.lua_command.commandType == "Inline"
//     ? command?.rcon_lua?.lua_command.command
//     : command?.rcon_lua?.lua_command.command.command}
// </Button>
