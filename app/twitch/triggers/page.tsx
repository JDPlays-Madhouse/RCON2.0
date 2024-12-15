"use client";
import IntegrationStatus from "@/components/integration-status";
// import { MainContextMenu } from "@/components/main-context-menu";
import MainNav from "@/components/main-navbar";
import ServerControl from "@/components/server-control";
import ServerDashboard from "@/components/server-dashboard";
import LogArea from "@/components/server-log";
import ServerSwitcher from "@/components/server-switcher";
import {
    ImperativePanelHandle,
    ResizableHandle,
    ResizablePanel,
    ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Server, Servers } from "@/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import React, { useEffect, useRef, useState } from "react";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ChannelPointRewardsDashboard from "@/components/channel-point-reward-table-dashboard";
import { ScrollArea } from "@/components/ui/scroll-area";

export default function Home() {
    const [selectedServer, setSelectedServer] = React.useState<Server>();
    const [showLogs, setShowLogs] = useState<boolean>(false);

    useEffect(() => {
        invoke<boolean>("get_config_bool", { key: "show_logs" }).then((show) =>
            setShowLogs(show),
        );
    }, []);
    useEffect(() => {
        invoke<Server>("get_default_server").then((server: Server) => {
            setSelectedServer(server);
        });
    }, []);
    const logRef = useRef<ImperativePanelHandle>(null);

    useEffect(() => {
        if (logRef && logRef.current && showLogs) {
            logRef.current.expand();
        } else if (logRef && logRef.current && showLogs === false) {
            logRef.current.collapse();
        }
    }, [showLogs]);

    return (
        <div className="flex flex-col h-dvh bg-background">
            {/*<MainContextMenu
        selectedServer={selectedServer}
        setSelectedServer={setSelectedServer}
        servers={servers}
      >*/}
            <header className="px-10 py-5 flex flex-row justify-between w-full max-w-[2560px] mx-auto border border-t-none border-x-none flex-initial">
                <div className="flex flex-row gap-4 items-center">
                    <ServerSwitcher
                        className=""
                        selectedServer={selectedServer}
                        setSelectedServer={setSelectedServer}
                    />
                    <ServerControl selectedServer={selectedServer} />
                    <IntegrationStatus />
                </div>
                <MainNav server={selectedServer} />
            </header>

            <ResizablePanelGroup direction="vertical" className="border max-w-dvw">
                <ResizablePanel defaultSize={70}>
                    <div className="flex flex-col justify-center items-center gap-2 px-10">
                        <div className="text-2xl p-4 font-bold">Triggers</div>
                        <Tabs
                            defaultValue="channelpointrewards"
                            className="w-full items-center justify-center flex flex-col"
                        >
                            <TabsList>
                                <TabsTrigger value="chat">Chat</TabsTrigger>
                                <TabsTrigger value="channelpointrewards">
                                    Channel Point Rewards
                                </TabsTrigger>
                            </TabsList>
                            <TabsContent value="chat">Chat pattern.</TabsContent>
                            <TabsContent value="channelpointrewards" className="w-full mx-10">
                                <ChannelPointRewardsDashboard />
                            </TabsContent>
                        </Tabs>
                    </div>
                </ResizablePanel>
                <ResizableHandle withHandle />

                <ResizablePanel
                    ref={logRef}
                    defaultSize={30}
                    collapsible={true}
                    className="flex"
                >
                    <LogArea />
                </ResizablePanel>
            </ResizablePanelGroup>

            {/* </MainContextMenu> */}
        </div>
    );
}
