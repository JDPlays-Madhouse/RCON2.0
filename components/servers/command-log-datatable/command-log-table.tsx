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
  PaginationState,
  SortingState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import * as React from "react";
import { Button } from "@/components/ui/button";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ArrowUpDown, Check, X } from "lucide-react";
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
import { ButtonGroup } from "@/components/ui/button-group";

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
    accessorKey: "trigger.server.name",
    header({ column }) {
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
    id: "sendCommand",
    header: "Resend",
    cell: ({ row, table }) => {
      const resendCommand = () => {
        const command_log = row.original;
        // @ts-expect-error using custom command
        table.options.meta.resendCommand(command_log);
      };
      const resendEvent = () => {
        const command_log = row.original;
        // @ts-expect-error using custom command
        table.options.meta.resendEvent(command_log);
      };
      return (
        <ButtonGroup className="mx-auto">
          <Button
            variant="secondary"
            className="bg-primary/50 hover:bg-primary active:bg-primary/50"
            onClick={() => resendCommand()}
          >
            Command
          </Button>
          <Button
            variant="secondary"
            className="bg-green-800 hover:bg-green-500 active:bg-green-500/20"
            onClick={() => resendEvent()}
          >
            Event
          </Button>
        </ButtonGroup>
      );
    },
  },
];

interface DataTableProps {
  selectedServer?: Server;
}

export default function CommandLogTable({ selectedServer }: DataTableProps) {
  const [thisServerOnly, setThisServerOnly] = useState(false);
  const [data, setData] = useState<CommandLog[]>([]);
  const [count, setCount] = useState(0);
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const [pagination, setPagination] = React.useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const updateTable = () => {
    setCount((c) => c + 1);
  };

  useEffect(() => {
    handleUpdateData();
    if (!autoRefresh) {
      return;
    }
    const intervalId = setInterval(handleUpdateData, 1000);
    return () => {
      clearInterval(intervalId);
    };
  }, [selectedServer, autoRefresh]);

  // TODO: Change to just fetch new data.
  const handleUpdateData = () => {
    // updateTable();
    if (selectedServer) {
      invoke<CommandLog[]>("get_command_logs").then((commands) => {
        // if (commands.length === data.length) {
        //   return;
        // }
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
  const [sorting, setSorting] = React.useState<SortingState>([
    { id: "time", desc: false },
  ]);
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    onPaginationChange: setPagination,
    getPaginationRowModel: getPaginationRowModel(),
    autoResetPageIndex: false,
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    onColumnFiltersChange: setColumnFilters,
    getFilteredRowModel: getFilteredRowModel(),
    state: {
      sorting,
      columnFilters,
      pagination,
    },
    meta: {
      resendCommand: function f(commandLog: CommandLog) {
        invoke("resend_command", { commandLog })
          .then(console.log)
          .catch(console.warn);
      },
      resendEvent: function f(commandLog: CommandLog) {
        invoke("resend_event", { commandLog })
          .then(console.log)
          .catch(console.warn);
      },
    },
  });
  // send_command_to_server
  return (
    <div className="w-full">
      <div className="flex items-center py-4 justify-center flex-row w-full gap-2">
        <Input
          placeholder="Filter by Username..."
          value={
            (table.getColumn("username")?.getFilterValue() as string) ?? ""
          }
          onChange={(event) =>
            table.getColumn("username")?.setFilterValue(event.target.value)
          }
          className="max-w-sm border-secondary-foreground/50"
        />
        <Input
          placeholder="Filter by Command Name..."
          value={(table.getColumn("name")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("name")?.setFilterValue(event.target.value)
          }
          className="max-w-sm border-secondary-foreground/50"
        />
        <Input
          placeholder="Filter by Trigger Name..."
          value={(table.getColumn("trigger")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("trigger")?.setFilterValue(event.target.value)
          }
          className="max-w-sm border-secondary-foreground/50"
        />
        <Button
          className="flex flex-row text-foreground items-center justify-center gap-1"
          variant="secondary"
          onClick={() => {
            setThisServerOnly(!thisServerOnly);
          }}
        >
          {thisServerOnly ? <Check className="" /> : <X />}{" "}
          <span>This server only.</span>
        </Button>
        <ButtonGroup>
          <Button
            onClick={() => setAutoRefresh((r) => !r)}
            variant="secondary"
            className={
              autoRefresh
                ? "bg-green-500/70 hover:bg-green-500/100 active:bg-green-500/30"
                : "bg-amber-500/70 hover:bg-amber-500/100 active:bg-amber-500/30"
            }
          >
            {autoRefresh ? "Stop auto refreshing" : "Start auto refreshing"}
          </Button>
          <Button
            onClick={() => {
              handleUpdateData();
              setAutoRefresh(autoRefresh);
            }}
            variant="secondary"
            className="active:bg-green-500/30"
          >
            Manual Refresh
          </Button>
        </ButtonGroup>
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
