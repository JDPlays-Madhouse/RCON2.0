import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { invoke } from "@tauri-apps/api/core";
import { DarkModeToggle } from "../dark-mode-button";

export default function Settings() {
  const handleOnClick = () => {
    invoke("restart").then();
  };
  return (
    <div className="flex flex-col w-full h-full justify-center align-center gap-5">
      <DarkModeToggle className="mx-auto text-4xl h-[4.5rem] w-[4.5rem] p-4 b-10" />
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="destructive"
              className="text-4xl h-max pt-4 px-9 rounded-full max-w-fit mx-auto"
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
