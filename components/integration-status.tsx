"use client";

import Twitch from "@/components/icons/twitch";
import YouTube from "@/components/icons/youtube";
import { cn, defaultIntegrationStatus } from "@/lib/utils";
import React, { useEffect, useState } from "react";
import { Api, apis, IntegrationStatus, IntegrationStatusMap } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import IntegrationLogo from "./icons";

interface IntegrationStatusProps extends React.ComponentProps<"div"> { }

export default function IntegrationStatusBar({
    className = "",
    ...props
}: IntegrationStatusProps) {
    const [statuses, setStatuses] = useState<IntegrationStatusMap>(
        defaultIntegrationStatus(),
    );
    const [integrations, setIntegrations] = useState<Api[]>([]);

    useEffect(() => {
        invoke<boolean>("get_config_bool", {
            key: "auth.twitch.auto_connect",
        }).then((connect) => {
            if (connect) {
                handleConnectToIntegration(Api.Twitch);
            } else {
                handleIntegrationStatusCheck(Api.Twitch);
            }
        });
    }, [integrations]);

    useEffect(() => {
        for (const api of apis) {
            handleIntegrationStatusCheck(Api[api]);
        }
    }, [integrations]);

    useEffect(handleListOfIntegrations, []);

    function handleConnectToIntegration(api: Api) {
        invoke<IntegrationStatus>("connect_to_integration", { api })
            .then((status) => {
                setStatuses((statuses) => {
                    statuses[api] = status;
                    console.log("Connect: ", statuses);
                    return statuses;
                });
            })
            .catch((error) => {
                if (error.error === "NotImplemented") {
                    setStatuses((statuses) => {
                        statuses[api] = {
                            status: "Error",
                            api: { api: Api[api], error: error },
                        };
                        console.log("Connect: ", statuses);
                        return statuses;
                    });
                }
            });
    }
    function handleListOfIntegrations() {
        invoke<Api[]>("list_of_integrations").then((value) => {
            setIntegrations(value);
        });
    }
    function handleIntegrationStatusCheck(api: Api) {
        invoke<IntegrationStatus>("integration_status", { api })
            .then((status) => {
                setStatuses((statuses) => {
                    statuses[api] = status;
                    console.log("Check: ", statuses);
                    return statuses;
                });
            })
            .catch((error) => {
                setStatuses((statuses) => {
                    statuses[api] = {
                        status: "Error",
                        api: { api: Api[api], error: error },
                    };
                    console.log("Connect: ", statuses);
                    return statuses;
                });
            });
    }

    function handleOnClick(api: Api) {
        console.log(statuses);
        switch (statuses[api].status) {
            case "Connected":
                handleIntegrationStatusCheck(api);
                break;
            case "Unknown":
            case "Disconnected":
                statuses[api] = { status: "Connecting", api: Api.Twitch };
                setStatuses(statuses);
                handleConnectToIntegration(api);
                break;
        }
    }
// TODO: Fix here next.
    return (
        <div
            className={cn("flex flex-row gap-2 content-center", className)}
            {...props}
        >
            <IntegrationLogo status={statuses.Twitch} primary_colour="#9146FF" onClick={() => handleOnClick(Api.Twitch)}>
            <Twitch
                className="h-6 w-6"
            />
            </IntegrationLogo>
            <YouTube status={statuses.YouTube} className="h-6 w-7" />
        </div>
    );
}
