"use client";
import { Server } from "@/app/page";
import {
  NavigationMenu,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";
import { cn } from "@/lib/utils";
import NextLink from "next/link";
import { DarkModeToggle } from "./dark-mode-button";

function Link({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <NextLink href={href} legacyBehavior passHref>
      <NavigationMenuLink className={navigationMenuTriggerStyle()}>
        {children}
      </NavigationMenuLink>
    </NextLink>
  );
}

export default function MainNav({
  className = "",
  server,
}: {
  className?: string;
  server: Server;
}) {
  return (
    <NavigationMenu
      className={cn("flex items-center space-x-4 lg:space-x-6", className)}
    >
      <NavigationMenuList>
        <NavigationMenuItem key="Dashboard">
          <Link href={"/"}>Dashboard</Link>
        </NavigationMenuItem>
        <NavigationMenuItem key="mainnavServerSettings">
          <Link href={"/settings/" + server.id}>Server Config</Link>
        </NavigationMenuItem>
        <NavigationMenuItem key="mainnavSettings">
          <Link href="/settings">Settings</Link>
        </NavigationMenuItem>
        <NavigationMenuIndicator />
        {/* TODO: Work out indicator or remove it */}
      </NavigationMenuList>
      <DarkModeToggle />
    </NavigationMenu>
  );
}
