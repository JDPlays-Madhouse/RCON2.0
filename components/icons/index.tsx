import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { IntegrationStatus } from "@/types";
import { TooltipTriggerProps } from "@radix-ui/react-tooltip";
import React, { useEffect } from "react";
import CountdownTimer from "../countdown-timer";

export interface LogoProps {
  fill: string;
  className: string;
}

interface IntegrationProps extends TooltipTriggerProps {
  status: IntegrationStatus;
  primaryColor?: string;
  secondaryColor?: string;
  Logo: React.FC<LogoProps>;
  name: string;
}

export default function IntegrationLogo({
  name,
  Logo,
  status,
  className = "w-8 h-8",
  primaryColor = "#FFFFFF",
  secondaryColor = "#1e293b",
  ...props
}: IntegrationProps) {
  const [fill, setFill] = React.useState(secondaryColor);
  const [displayStatus, setDisplayStatus] = React.useState<string>(
    status.status,
  );
  const [displayText, setDisplayText] = React.useState<string>(status.status);
  useEffect(() => {
    setFill(fillColor());
    setDisplayStatus(handleDisplayStatus());
    setDisplayText(handleDisplayText());
  }, [status]);

  const fillColor = () => {
    return status.status === "Connected" ? primaryColor : secondaryColor;
  };

  function handleDisplayStatus() {
    let displayStatus;
    switch (status.status) {
      case "Error": {
        if (status.api.error.error === "NotImplemented") {
          displayStatus = "Not Implemented Yet";
        } else {
          displayStatus = "Error - " + status.api.error.error;
        }
        break;
      }
      default:
        displayStatus = status.status;
        break;
    }
    return displayStatus;
  }

  function handleDisplayText() {
    let displayText;
    switch (status.status) {
      case "Connected":
        displayText = "Click to check status.";
        break;
      case "Disconnected":
        displayText = "Click to connect.";
        break;
      case "Error":
        switch (status.api.error.error) {
          case "Token":
            displayText = "Click to reconnect.";
            break;
          case "NotImplemented":
            displayText = "Annoy the dev until he implements this.";
            break;
          default:
            displayText = "Check the logs for more info.";
            break;
        }
        break;
      default:
        displayText = "Click to update status.";
    }
    return displayText;
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger {...props}>
          <Logo fill={fill} className={className} />
        </TooltipTrigger>
        <TooltipContent className="mt-2 text-s bg-secondary" key={name + status.status}>
          <div>
            {name}: {displayStatus}
            {status.status == "Connected" && status.api.expires_at ? (
              <CountdownTimer seconds={status.api.expires_at} preText="Token: "/>
            ) : (
              <></>
            )}
          </div>
          <div>{displayText}</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
