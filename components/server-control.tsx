"use client";

import { cn } from "@/lib/utils";
import { Server, ServerStatus } from "@/types";
import { Pause, Play, StopCircle } from "lucide-react";
import React, { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Channel, invoke } from "@tauri-apps/api/core";

interface ServerControlProps extends React.ComponentProps<"div"> {
    selectedServer?: Server;
}

export default function ServerControl({
    selectedServer,
    className,
    ...props
}: ServerControlProps) {
    const [channel, setChannel] = useState<Channel<ServerStatus>>();
    const [status, setStatus] = useState<ServerStatus>({
        event: "disconnected",
        data: { server: selectedServer },
    });
    const connected = () => status.event === "connected";

    useEffect(() => {
        if (selectedServer) {
            if (!connected() && status.data.server?.id === selectedServer?.id) {
                invoke<boolean>("get_config_bool", {
                    key: "servers.autostart",
                }).then((connect) => {
                    if (connect) {
                        handleServerConnect();
                    }
                });
            } else {
                handleServerCheck();
            }
        }
    }, [selectedServer]);

    useEffect(() => {
        const channel = new Channel<ServerStatus>();
        channel.onmessage = (message) => {
            // console.log(message.data.server, selectedServer);
            if (message.data.server?.id === selectedServer?.id) {
                setStatus(message);
            }
            setChannel(channel);
        };
    });

    function handleOnClick() {
        if (["disconnected", "error"].includes(status.event)) {
            handleServerConnect();
        } else if (status.event == "connected") {
            handleServerDisconnect();
        }
    }

    function handleServerConnect() {
        if (!channel) {
            const channel = new Channel<ServerStatus>();
            channel.onmessage = (message) => {
                // console.log(message.data.server, selectedServer);
                if (message.data.server?.id === selectedServer?.id) {
                    setStatus(message);
                }
                setChannel(channel);
            };
            invoke<Server>("connect_to_server", {
                server: selectedServer,
                channel,
            })
                .then((server) => {
                    console.log(`Connected to server: ${server.name}`);
                })
                .catch(console.error);
        } else {
            invoke<Server>("connect_to_server", {
                server: selectedServer,
                channel,
            })
                .then((server) => {
                    console.log(`Connected to server: ${server.name}`);
                })
                .catch(console.error);
        }
    }

    function handleServerCheck() {
        if (selectedServer) {
            invoke<ServerStatus>("check_connection", { server: selectedServer })
                .then((s) => {
                    setStatus(s);
                })
                .catch(console.error);
        }
    }
    function handleServerDisconnect() {
        invoke<ServerStatus>("disconnect_connection", {
            server: selectedServer,
        })
            .then((s) => {
                setStatus(s);
            })
            .catch(console.error);
    }
    const handleMessage = () => {
        switch (status.event) {
            case "connected":
                return `Connected to ${status.data.server.name}`;
            case "connecting":
                return `Connecting to ${status.data.server.name}...`;
            case "checking":
                return `Checking ${status.data.server.name}...`;
            case "error":
                return `Error: ${status.data.msg}.`;
            case "disconnected":
                if (status.data.server) {
                    return `Disconnected from ${status.data.server.name}`;
                } else {
                    return "Disconnected";
                }
        }
    };

    if (!selectedServer) {
        return <></>;
    }

    return (
        <div
            className={cn("flex flex-row align-center items-center gap-2", className)}
            {...props}
        >
            <Button
                variant={!connected() ? "destructive" : "default"}
                size="icon"
                onClick={handleOnClick}
            >
                {connected() ? <StopCircle /> : <Play />}
            </Button>
            <span onClick={handleServerCheck}>{handleMessage()}</span>
        </div>
    );
}
