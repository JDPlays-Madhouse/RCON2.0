"use client";
import { Server } from "@/types";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";
import { cn } from "@/lib/utils";
import NextLink from "next/link";
import { DarkModeToggle } from "./dark-mode-button";
import React from "react";
import Twitch from "./icons/twitch";

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
}: {
  className?: string;
  server?: Server;
}) {
  return (
    <NavigationMenu
      className={cn("flex items-center space-x-4 lg:space-x-6", className)}
    >
      <NavigationMenuList>
        <NavigationMenuItem key="Dashboard">
          <Link href={"/"}>Dashboard</Link>
        </NavigationMenuItem>
        <NavigationMenuItem key="Twitch">
          <NavigationMenuTrigger>Twitch</NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid gap-3 p-4 md:w-[400px] lg:w-[500px] lg:grid-cols-[.75fr_1fr]">
                            <ListItem href="/twitch/triggers" title="Triggers">
                Trigger specific for Twitch.
              </ListItem>
              <ListItem href="#" title="Some Helpful Information">
                Some holder text
              </ListItem>
              <ListItem href="#" title="Some other Helpful Information">
                Some holder text
              </ListItem>
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>
        <NavigationMenuItem key="mainnavServerSettings">
          <Link href="/server-config">Server Config</Link>
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
const ListItem = React.forwardRef<
  React.ElementRef<"a">,
  React.ComponentPropsWithoutRef<"a">
>(({ className, title, children, ...props }, ref) => {
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          ref={ref}
          className={cn(
            "block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
            className,
          )}
          {...props}
        >
          <div className="text-sm font-medium leading-none">{title}</div>
          <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">
            {children}
          </p>
        </a>
      </NavigationMenuLink>
    </li>
  );
});
ListItem.displayName = "ListItem";
