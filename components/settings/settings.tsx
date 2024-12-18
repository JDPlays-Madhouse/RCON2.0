import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { invoke } from "@tauri-apps/api/core";

export default function Settings() {
  const handleOnClick = () => {
    invoke("restart").then();
  };
  return (
    <div
      
      className="flex flex-col w-full h-full justify-center align-center"
    >
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="destructive"
              className="text-7xl h-max pt-6 px-9 rounded-full max-w-fit mx-auto"
              onClick={handleOnClick}
            >
              Restart Application
            </Button>
          </TooltipTrigger>
          <TooltipContent className="text-5xl bg-destructive text-white rounded-full pt-4">
            <p>This will restart the app!!!! Also this button is in beta...</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  );
}
