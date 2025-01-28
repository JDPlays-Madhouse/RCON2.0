"use client";
import { cn } from "@/lib/utils";

import { Server } from "@/types";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import OverviewTable from "./overview-datatable/overview-table";
import CommandsTable from "./commands-datatable/commands-table";

interface ServerDashBoardProps extends React.ComponentProps<"div"> {
  showLog: boolean;
  selectedServer?: Server;
}

export default function ServerDashboard({
  selectedServer,
  className,
  showLog,
  ...props
}: ServerDashBoardProps) {
  return (
    <div
      className={cn(
        "flex flex-col h-full w-full items-center justify-start p-6 my-auto gap-2",
        className,
      )}
      {...props}
    >
      <div className="font-semibold">Dashboard</div>
      <Tabs
        defaultValue="overview"
        className="w-full items-center justify-center flex flex-col"
      >
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="commands">Commands</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" className="w-full">
          <OverviewTable selectedServer={selectedServer} />
        </TabsContent>
        <TabsContent value="commands" className="w-full">
          <CommandsTable selectedServer={selectedServer}/>
        </TabsContent>
      </Tabs>
    </div>
  );
}
