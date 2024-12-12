"use client";

import Twitch from "@/components/icons/twitch";
import YouTube from "@/components/icons/youtube";
import { cn, defaultIntegrationStatus } from "@/lib/utils";
import React, { useEffect, useState } from "react";
import { Api, apis, IntegrationStatus, IntegrationStatusMap } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import IntegrationLogo from "./icons";
import Patreon from "@/components/icons/patreon";
import StreamLabs from "@/components/icons/streamlabs";

interface IntegrationStatusProps extends React.ComponentProps<"div"> {}

export default function IntegrationStatusBar({
  className = "",
  ...props
}: IntegrationStatusProps) {
  let c = 0;
  const [statuses, setStatuses] = useState<IntegrationStatusMap>(
    defaultIntegrationStatus()
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

  function handleSetStatuses(status: IntegrationStatus, api: Api) {
    if (statuses[api].status === status.status) return;
    c += 1;
    console.log("handleSetStatuses", c, status, api);
    statuses[api] = status;
    setStatuses({ ...statuses });
  }

  function handleConnectToIntegration(api: Api) {
    // handleSetStatuses({ status: "Connecting", api }, api);
    invoke<IntegrationStatus>("connect_to_integration", { api })
      .then((status) => {
        handleSetStatuses(status, api);
      })
      .catch((error) => {
        handleSetStatuses({ status: "Error", api: { api, error } }, api);
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
        handleSetStatuses(status, api);
      })
      .catch((error) => {
        handleSetStatuses({ status: "Error", api: { api, error } }, api);
      });
  }

  function handleOnClick(api: Api) {
    switch (statuses[api].status) {
      case "Connected":
        handleIntegrationStatusCheck(api);
        break;
      case "Unknown":
      case "Error":
      case "Disconnected":
        statuses[api] = { status: "Connecting", api: Api.Twitch };
        setStatuses(statuses);
        handleConnectToIntegration(api);
        break;
    }
  }

  return (
    <div
      className={cn("flex flex-row gap-2 content-center", className)}
      {...props}
    >
      <IntegrationLogo
        status={statuses.Twitch}
        Logo={Twitch}
        primaryColor="#9146FF"
        onClick={() => handleOnClick(Api.Twitch)}
      />
      <IntegrationLogo
        status={statuses.YouTube}
        Logo={YouTube}
        primaryColor="#FF0000"
        onClick={() => handleOnClick(Api.Twitch)}
      />
      <IntegrationLogo
        status={statuses.Patreon}
        Logo={Patreon}
        primaryColor="#000000"
        onClick={() => handleOnClick(Api.Patreon)}
      />
      <IntegrationLogo
        status={statuses.StreamLabs}
        Logo={StreamLabs}
        primaryColor="#80F5D2"
        onClick={() => handleOnClick(Api.StreamLabs)}
      />
    </div>
  );
}
