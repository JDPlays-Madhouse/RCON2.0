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
import { ClipboardIcon, MoreHorizontal } from "lucide-react";
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
import { DataTablePagination } from "./datatables/pagination";

export type CustomChannelPointRewardInfo = {
  id: string;
  title: string;
  is_enabled: boolean;
  is_paused: boolean;
  is_in_stock: boolean;
  cost: number;
  is_user_input_required: boolean;
};

export const column: ColumnDef<CustomChannelPointRewardInfo>[] = [
  {
    id: "id",
    accessorKey: "id",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => {
            console.log(column.getIsSorted());
            column.toggleSorting(column.getIsSorted() === "asc");
          }}
        >
          ID
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "title",
    accessorKey: "title",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Title <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },

  {
    id: "is_enabled",
    accessorKey: "is_enabled",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Enabled <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ cell }) => {
      return <Checkbox checked={cell.getValue() as boolean} />;
    },
  },
  {
    id: "is_paused",
    accessorKey: "is_paused",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Paused <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ cell }) => {
      return <Checkbox checked={cell.getValue() as boolean} />;
    },
  },
  {
    id: "is_in_stock",
    accessorKey: "is_in_stock",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          In Stock <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ cell }) => {
      return <Checkbox checked={cell.getValue() as boolean} />;
    },
  },
  {
    id: "is_user_input_required",
    accessorKey: "is_user_input_required",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          In Stock <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ cell }) => {
      return <Checkbox checked={cell.getValue() as boolean} />;
    },
  },
  {
    id: "cost",
    accessorKey: "cost",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          Cost <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
  },
  {
    id: "actions",
    header: "Copy Info for Config",
    cell: ({ row }) => {
      const reward: CustomChannelPointRewardInfo = row.original;
      const copy_text = `trigger_type = "ChannelPointRewardRedeemed"\nid =  "${reward.id}"\ntitle = "${reward.title}"`;
      return (
        <CellToolTip
          helper={
            <div>
              It will copy, for pasting in config file:
              <br />
              trigger_type = &quot;ChannelPointRewardRedeemed&quot;
              <br />
              id = &quot;{reward.id}&quot;
              <br />
              title = &quot;{reward.title}&quot;
            </div>
          }
        >
          <Button
            className="w-6 h-6 p-[4px]"
            variant="secondary"
            size="icon"
            onClick={() => {
              navigator.clipboard.writeText(copy_text);
            }}
          >
            <ClipboardIcon />
          </Button>
        </CellToolTip>
      );
    },
  },
];

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
}
export default function ChannelPointRewardsTable<TData, TValue>({
  columns,
  data,
}: DataTableProps<TData, TValue>) {
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    [],
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
          placeholder="Filter Channel Point Reward Title..."
          value={(table.getColumn("title")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("title")?.setFilterValue(event.target.value)
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
                  No Channel Point Rewards found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
        <div className="flex items-center justify-end space-x-2 py-4">
          <DataTablePagination table={table} />
        </div>
      </div>
    </div>
  );
}

function CellToolTip({
  helper,
  children,
}: {
  helper: string | React.ReactNode;
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

