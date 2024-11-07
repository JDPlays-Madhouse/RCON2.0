import { Game, games, Servers } from "@/types";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function defaultServers(): Servers {
  // @ts-expect-error(Constructing servers.)
  const servers: Servers = {};
  for (const game of games) {
    servers[Game[game]] = gameBuilder(Game[game]);
  }
  return servers;
}

function gameBuilder(game: Game) {
  return {
    label: game,
    servers: [],
  };
}
