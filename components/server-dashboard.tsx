"use client";
import { cn } from "@/lib/utils";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  type ImperativePanelHandle,
} from "@/components/ui/resizable";
import { useEffect, useRef } from "react";
import LogArea from "./server-log";
import { invoke } from "@tauri-apps/api/core";

interface ServerDashBoardProps extends React.ComponentProps<"div"> {
  showLog: boolean;
}

export default function ServerDashboard({
  className,
  showLog,
  ...props
}: ServerDashBoardProps) {
  const logRef = useRef<ImperativePanelHandle>(null);

  useEffect(() => {
    if (logRef && logRef.current && showLog) {
      logRef.current.expand();
    } else if (logRef && logRef.current && showLog === false) {
      logRef.current.collapse();
    }
  }, [showLog]);
  const handleOnClick = () => {
    invoke("log", {
      level: "INFO",
      target: "Server_Dashboard",
      message: "Hello from the renderer process!",
    });
  };
  return (
    <div className={cn("", className)} {...props}>
      <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
        <ResizablePanel defaultSize={50}>
          <div
            className="flex h-full items-center justify-center p-6 my-auto"
            onClick={handleOnClick}
          >
            <span className="font-semibold">Dashboard</span>
          </div>
        </ResizablePanel>
        <ResizableHandle withHandle />

        <ResizablePanel
          ref={logRef}
          defaultSize={50}
          collapsible={true}
          className="flex"
        >
          <LogArea />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
