"use client";

import { Command, Game, GameServerTrigger, Trigger } from "@/types";
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
import { MoreHorizontal } from "lucide-react";
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
import { Checkbox } from "@/components/ui/checkbox";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export type CommandTrigger = {
  serverTrigger: GameServerTrigger;
  command?: Command;
};

export const column: ColumnDef<CommandTrigger>[] = [
  {
    id: "enabled",
    accessorKey: "serverTrigger.enabled",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => {
            console.log(column.getIsSorted());
            column.toggleSorting(column.getIsSorted() === "asc");
          }}
        >
          Enabled
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ row, cell }) => {
      console.log({ row, cell });
      return (
        <Checkbox
          checked={cell.getValue() as boolean}
          onCheckedChange={(value) => {
            row.original.serverTrigger.enabled = !cell.getValue() as boolean;
            console.log(row.original);
          }}
          aria-label="Enable command."
        />
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
      console.log("Trigger: ", trigger);
      switch (trigger.trigger) {
        case "Chat":
          return (
            <CellToolTip helper="The pattern that will trigger the command.">
              {trigger.data.pattern}
            </CellToolTip>
          );
        case "ChatRegex":
          return (
            <CellToolTip helper="The pattern that will trigger the command.">
              {trigger.data.pattern}
            </CellToolTip>
          );
        case "ChannelPointRewardRedeemed":
          return (
            <CellToolTip helper="{Channel Points Reward Name} - {Twitch ID for reward}">
              {`${trigger.data.title} - ${trigger.data.id}`}
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
            <DropdownMenuSeparator />
            <DropdownMenuItem>View customer</DropdownMenuItem>
            <DropdownMenuItem>View payment details</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
}
export default function DashboardTable<TData, TValue>({
  columns,
  data,
}: DataTableProps<TData, TValue>) {
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
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
  });
  return (
    <div className="w-full">
      <div className="flex items-center py-4">
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
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}

function CellToolTip({
  helper,
  children,
}: {
  helper: string;
  children: React.ReactNode;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>{children}</TooltipTrigger>
        <TooltipContent className="mt-2 text-s bg-secondary">
          <p>{helper}</p>
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
