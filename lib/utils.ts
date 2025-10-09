import {
  Api,
  apis,
  Game,
  games,
  IntegrationStatusMap,
  Servers,
  SystemTime,
} from "@/types";
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

export function defaultIntegrationStatus(): IntegrationStatusMap {
  // @ts-expect-error(Constructing status.)
  const status: IntegrationStatusMap = {};
  for (const api of apis) {
    status[Api[api]] = { status: "Unknown" };
  }
  return status;
}

export function systemTimeToDate(systemTime: SystemTime) {
  return new Date(
    systemTime.secs_since_epoch * 1000 + systemTime.nanos_since_epoch / 1000000
  );
}
