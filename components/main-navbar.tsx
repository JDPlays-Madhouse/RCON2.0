import { Page, Server } from "@/types";
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
import React from "react";

interface MainNavProps {
  className?: string;
  page: Page;
  setPage: React.Dispatch<React.SetStateAction<Page>>;
}
interface ListItemProps extends Omit<React.ComponentPropsWithRef<"a">, "href"> {
  page: Page;
  setPage: React.Dispatch<React.SetStateAction<Page>>;
}

function Link({
  href,
  children,
  setPage,
}: {
  href: Page;
  children: React.ReactNode;
  setPage: React.Dispatch<React.SetStateAction<Page>>;
}) {
  return (
    <NavigationMenuLink
      onClick={() => {
        setPage(href);
      }}
      className={cn(navigationMenuTriggerStyle(), "cursor-pointer")}
    >
      {children}
    </NavigationMenuLink>
  );
}

function ListItem({
  ref,
  className,
  title,
  page,
  children,
  setPage,
  ...props
}: ListItemProps) {
  return (
    <li className="cursor-pointer">
      <NavigationMenuLink
        asChild
        onClick={() => {
          setPage(page);
        }}
      >
        <a
          ref={ref}
          className={cn(
            "block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
            className
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
}
ListItem.displayName = "ListItem";

export default function MainNav({
  className = "",
  page,
  setPage,
}: MainNavProps) {
  return (
    <NavigationMenu
      className={cn("flex items-center space-x-4 lg:space-x-6", className)}
    >
      <NavigationMenuList>
        <NavigationMenuItem key="Dashboard">
          <Link href={Page.Dashboard} setPage={setPage}>
            Dashboard
          </Link>
        </NavigationMenuItem>
        <NavigationMenuItem key="Twitch">
          <NavigationMenuTrigger>Twitch</NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid gap-3 p-4 min-w-[300px]">
              <ListItem
                page={Page.TwitchTriggers}
                setPage={setPage}
                title="Triggers"
              >
                Trigger specific for Twitch.
              </ListItem>
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>
        <NavigationMenuItem key="mainnavServerSettings">
          <Link href={Page.ServerSettings} setPage={setPage}>
            Server Config
          </Link>
        </NavigationMenuItem>
        <NavigationMenuItem key="mainnavSettings">
          <Link href={Page.Settings} setPage={setPage}>
            Settings
          </Link>
        </NavigationMenuItem>
        <NavigationMenuIndicator />
        {/* TODO: Work out indicator or remove it */}
      </NavigationMenuList>
    </NavigationMenu>
  );
}
