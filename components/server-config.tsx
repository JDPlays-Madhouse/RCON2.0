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
import { ServerConfigForm } from "./forms/server-config-form";
import { Server } from "@/types";

interface ServerConfigProps extends React.ComponentProps<"div"> {
    showLog: boolean;
    server?: Server;
}

export default function ServerConfig({
    className,
    showLog,
    server,
    ...props
}: ServerConfigProps) {
    const logRef = useRef<ImperativePanelHandle>(null);

    useEffect(() => {
        if (logRef && logRef.current && showLog) {
            logRef.current.expand();
        } else if (logRef && logRef.current && showLog === false) {
            logRef.current.collapse();
        }
    }, [showLog]);

    return (
        <div className={cn("", className)} {...props}>
            <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
                <ResizablePanel defaultSize={80} className="flex m-auto">
                    <ServerConfigForm className="m-auto" server={server} />
                </ResizablePanel>
                <ResizableHandle withHandle />

                <ResizablePanel
                    ref={logRef}
                    defaultSize={20}
                    collapsible={true}
                    className="flex"
                >
                    <LogArea />
                </ResizablePanel>
            </ResizablePanelGroup>
        </div>
    );
}
