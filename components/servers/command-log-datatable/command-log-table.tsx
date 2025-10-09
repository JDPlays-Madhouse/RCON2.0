"use client";

import {
  Command,
  CommandLog,
  GameServerTrigger,
  RconCommandPrefix,
  RconLuaCommand,
  Server,
  SystemTime,
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
import { cn, systemTimeToDate } from "@/lib/utils";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DataTablePagination } from "@/components/datatables/pagination";

export const columns: ColumnDef<CommandLog>[] = [
  {
    accessorKey: "time",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Time
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell({ cell }) {
      return systemTimeToDate(cell.getValue() as SystemTime).toLocaleString();
    },
  },
  {
    accessorKey: "username",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Username
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ getValue }) => {
      const value = getValue<string>();
      return <div>{value.slice(0, 100)}</div>;
    },
  },
  {
    id: "name",
    accessorKey: "command.name",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Command Name
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },

  {
    accessorKey: "trigger",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Trigger
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ getValue }) => {
      const value = getValue<GameServerTrigger>();
      return <div>{value.trigger.trigger}</div>;
    },
  },

  {
    accessorKey: "message",
    header({ column }) {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Message
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "sendCommand",
    header: "Send Command",
    cell: ({ row, table }) => {
      const sendCommand = () => {
        const command: Command = row.original.command;
        // @ts-expect-error using custom command
        table.options.meta.sendCommand(command);
      };
      return (
        <Button
          variant="secondary"
          className="hover:bg-primary active:bg-primary/50"
          onClick={() => sendCommand()}
        >
          Resend Command
        </Button>
      );
    },
  },
];

interface DataTableProps {
  selectedServer?: Server;
}

export default function CommandLogTable({ selectedServer }: DataTableProps) {
  const [data, setData] = useState<CommandLog[]>([]);
  const [count, setCount] = useState(0);

  const updateTable = () => {
    setCount((c) => c + 1);
  };

  useEffect(() => {
    const intervalId = setInterval(handleUpdateData, 5000);
    return () => {
      clearInterval(intervalId);
    };
  }, [selectedServer]);

  const handleUpdateData = () => {
    updateTable();
    if (selectedServer) {
      invoke<CommandLog[]>("get_command_logs").then((commands) => {
        setData(commands);
      });
    }
  };

  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
  function updateNestedValue<tValue, tObj>(
    keyArr: string[],
    value: tValue,
    obj: { [index: string]: tObj | tValue }
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
          .catch(console.warn);
      },
    },
  });
  // send_command_to_server
  return (
    <div className="w-full">
      <div className="flex items-center py-4 justify-between flex-row w-full">
        <Input
          placeholder="Filter by Username..."
          value={
            (table.getColumn("username")?.getFilterValue() as string) ?? ""
          }
          onChange={(event) =>
            table.getColumn("username")?.setFilterValue(event.target.value)
          }
          className="max-w-sm"
        />
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
                            header.getContext()
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
                        cell.getContext()
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
