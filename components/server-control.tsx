import { cn } from "@/lib/utils";
import { Server } from "@/types";
import { Pause, Play, StopCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Channel, invoke } from "@tauri-apps/api/core";

type ServerStatus =
    | {
        event: "connecting";
        data: { server: Server };
    }
    | { event: "connected"; data: { server: Server } }
    | { event: "checking"; data: { server: Server } }
    | { event: "error"; data: { msg: string; server: Server } }
    | {
        event: "disconnected";
        data: { server?: Server };
    };

interface ServerControlProps extends React.ComponentProps<"div"> {
    selectedServer?: Server;
}

export default function ServerControl({
    selectedServer,
    className,
    ...props
}: ServerControlProps) {
    const onEvent = new Channel<ServerStatus>();
    const [status, setStatus] = useState<ServerStatus>({
        event: "disconnected",
        data: { server: selectedServer },
    });
    const connected = () => status.event === "connected";
    useEffect(() => {
        handleServerCheck();
    }, [selectedServer]);
    onEvent.onmessage = (message) => {
        console.log(message.data.server, selectedServer);
        if (message.data.server?.id === selectedServer?.id) {
            setStatus(message);
        }
    };
    function handleOnClick() {
        if (["disconnected", "error"].includes(status.event)) {
            handleServerConnect();
        } else if (status.event == "connected") {
            handleServerDisconnect();
        }
    }
    function handleServerConnect() {
        invoke("connect_to_server", { server: selectedServer, channel: onEvent });
    }
    function handleServerCheck() {
        if (selectedServer){
        invoke<ServerStatus>("check_connection", { server: selectedServer }).then(
            (s) => {
                setStatus(s);
            },
        ).catch(console.error);}
    }
    function handleServerDisconnect() {
        invoke<ServerStatus>("disconnect_connection", {
            server: selectedServer,
        }).then((s) => {
            setStatus(s);
        });
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
