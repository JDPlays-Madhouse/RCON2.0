"use client";
import { cn } from "@/lib/utils";
import { ServerConfigForm } from "@/components/forms/server-config-form";
import { Server } from "@/types";

interface ServerConfigProps extends React.ComponentProps<"div"> {
    server?: Server;
    setSelectedServer: React.Dispatch<React.SetStateAction<Server | undefined>>;
}

export default function ServerConfig({
    className,
    server,
    setSelectedServer,
    ...props
}: ServerConfigProps) {

    return (
        <div className={cn("m-auto max-w-[500px] max-h-fit flex flex-col content-center mt-5", className)} {...props}>
            <div className="text-center text-2xl font-medium">Rcon Server Config</div>
                    <ServerConfigForm
                        className=""
                        server={server}
                        setSelectedServer={setSelectedServer}
                    />

        </div>
    );
}
