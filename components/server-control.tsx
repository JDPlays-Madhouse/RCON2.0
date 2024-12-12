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
    const [autoConnect, setAutoConnect] = useState<boolean>(false);
    const [status, setStatus] = useState<ServerStatus>({
        event: "disconnected",
        data: { server: selectedServer },
    });
    const connected = () => status.event === "connected";
    const connecting = () => status.event === "connecting";
    const checking = () => status.event === "checking";
    // console.log({ status, connected: connected() });

    useEffect(() => {
        invoke<boolean>("get_config_bool", {
            key: "servers.autostart",
        }).then((connect) => {
            setAutoConnect(connect);
        });
    }, []);

    useEffect(() => {
        if (selectedServer && channel) {
            if (!connected() && autoConnect && !connecting()) {
                handleServerConnect();
            } else {
                handleServerCheck();
            }
        }
    }, [selectedServer, autoConnect, channel]);

    useEffect(() => {
        const channel = new Channel<ServerStatus>();
        channel.onmessage = (message) => {
            // console.log("New Message: ", message);
            if (message.data.server?.id === selectedServer?.id) {
                setStatus(message);
            }
        };
        setChannel(channel);
    }, []); // Needs empty array..

    function handleOnClick() {
        if (["disconnected", "error"].includes(status.event)) {
            handleServerConnect();
        } else if (status.event == "connected") {
            handleServerDisconnect();
        } else {
            handleServerCheck();
        }
    }

    function handleServerConnect() {
        if (channel && selectedServer) {
            setStatus({ event: "connecting", data: { server: selectedServer } });
            invoke<ServerStatus>("connect_to_server", {
                server: selectedServer,
                channel,
            })
                .then((status) => {
                    // console.log(
                    //     `Connected to server with channel: ${status.data.server?.name}`,
                    // );
                    setStatus(status);
                })
                .catch((e: ServerStatus) => {
                    // console.log(`Error: ${e.event === "error" ? e.data.msg : "unknown"}`);
                });
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
                variant={connected() || connecting() ? "default" : "destructive"}
                size="icon"
                onClick={handleOnClick}
                disabled={["connecting", "checking"].includes(status.event)}
            >
                {connected() || connecting() ? <StopCircle /> : <Play />}
            </Button>
            <span onClick={handleServerCheck}>{handleMessage()}</span>
        </div>
    );
}
