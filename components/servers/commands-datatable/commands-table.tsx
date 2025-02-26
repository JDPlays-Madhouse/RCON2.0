"use client";

import {
  Command,
  GameServerTrigger,
  RconCommandPrefix,
  RconLuaCommand,
  Server,
} from "@/types";
import {
  ColumnDef,
  ColumnFiltersState,
  SortingState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { MoreHorizontal, Plus } from "lucide-react";
import * as React from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
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

export const columns: ColumnDef<Command>[] = [
  {
    accessorKey: "name",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Name
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "prefix",
    accessorKey: "rcon_lua.prefix",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Prefix
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell(props) {
      const value = props.getValue() as RconCommandPrefix;
      if (value.prefix != "Custom") {
        return value.prefix;
      } else {
        return `Custom - "${value.data}"`;
      }
    },
  },
  {
    id: "rcon_lua_type",
    accessorKey: "rcon_lua.lua_command.commandType",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Lua Type
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "rcon_lua",
    accessorKey: "rcon_lua.lua_command",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Lua
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell(props) {
      const value = props.getValue() as RconLuaCommand;
      switch (value.commandType) {
        case "File":
          return value.command.relative_path;
        case "Inline":
          return value.command;
        default:
          return "Error";
      }
    },
  },
  {
    accessorKey: "server_triggers",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Trigger Count
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ getValue }) => {
      const value = getValue() as GameServerTrigger[];
      return <div>{value.length}</div>;
    },
  },
  {
    id: "sendCommand",
    header: "Send Command",
    cell: ({ row, table }) => {
      const sendCommand = () => {
        const command: Command = row.original;
        // @ts-expect-error using custom command
        table.options.meta.sendCommand(command)
      }
      return <Button onClick={() => sendCommand()}>
        Send Command
      </Button>
    }
  },
  {
    id: "actions",
    cell: ({ row }) => {
      const command: Command = row.original;

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
                navigator.clipboard.writeText(command.name);
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

export default function CommandsTable({ selectedServer }: DataTableProps) {
  const [data, setData] = useState<Command[]>([]);
  const [count, setCount] = useState(0);

  const [commandFormOpen, setCommandFormOpen] = useState(true);
  const [triggerFormOpen, setTriggerFormOpen] = useState(true);

  useEffect(() => {
    handleUpdateData();
  }, [selectedServer]);

  const handleUpdateData = () => {
    setCount((c) => c + 1);
    if (selectedServer) {
      invoke<Command[]>("commands").then((commands) => {
        setData(commands);
      });
    }
  };

  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    [],
  );
  function updateNestedValue<tValue, tObj>(
    keyArr: string[],
    value: tValue,
    obj: { [index: string]: tObj | tValue },
  ): { [index: string]: tObj | tValue } {
    const key = keyArr.shift();
    if (!key) {
      return {};
    } else if (keyArr.length === 0 && key) {
      obj[key] = value;
      return obj;
    } else {
      // @ts-expect-error incorrect type
      obj[key] = updateNestedValue(keyArr, value, obj[key]);
      return obj;
    }
  }
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
      sendCommand: function f(command: Command) {
        invoke("send_command_to_server", { server: selectedServer, command })
          .then(console.log)
          .catch(console.warn)
      },
    },
  });
  // send_command_to_server
  return (
    <div className="w-full">
      <div className="flex items-center py-4 justify-between flex-row w-full">
        <Input
          placeholder="Filter Command..."
          value={(table.getColumn("name")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("name")?.setFilterValue(event.target.value)
          }
          className="max-w-sm"
        />
        <div className="flex flex-row gap-2">
          <Button
            variant="secondary"
            className="flex flex-row text-secondary-foreground text-base justify-center gap-1 pl-[11px]"
            onClick={() => setTriggerFormOpen((f) => !f && !commandFormOpen)}
            disabled
          >
            <Plus />
            <span className="pt-1">Trigger</span>
          </Button>
          <Button
            variant="secondary"
            className="flex flex-row text-secondary-foreground text-base justify-center gap-1 pl-[11px]"
            onClick={() => setCommandFormOpen((f) => !f && !triggerFormOpen)}
            disabled
          >
            <Plus />
            <span className="pt-1">Command</span>
          </Button>
        </div>
      </div>
      <div className="rounded-md border w-full">
        <Table>
          <TableHeader className="text-center justify-center">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id} className="text-center">
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
