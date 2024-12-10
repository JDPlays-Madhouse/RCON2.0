import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { IntegrationStatus } from "@/types";
import { TooltipTriggerProps } from "@radix-ui/react-tooltip";

interface IntegrationProps extends TooltipTriggerProps {
    status: IntegrationStatus;
    primary_color: string;
}

export default function IntegrationLogo({
    status,
    className = "",
    children,
    primary_color,
    ...props
}: IntegrationProps) {
    console.log(children);

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger {...props}>{children}</TooltipTrigger>
                <TooltipContent>
                    <div>Twitch: {status.status}</div>
                    <div>
                        {status.status === "Connected"
                            ? "Click to check status."
                            : "Click to connect."}
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}
