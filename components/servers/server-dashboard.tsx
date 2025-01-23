"use client";
import { cn } from "@/lib/utils";

import { Server } from "@/types";
import DashboardTable, {
} from "@/components/servers/dashboard-table";

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
        "flex flex-col h-full items-center justify-start p-6 my-auto gap-2",
        className,
      )}
      {...props}
    >
      <div className="font-semibold">Dashboard</div>
      <DashboardTable selectedServer={selectedServer} />
    </div>
  );
}
