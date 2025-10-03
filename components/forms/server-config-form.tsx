"use client";

import { Game, games, GameString, Server } from "@/types";
import { ipv4, ipv6, number, preprocess, string, z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { useEffect, useState } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { EyeIcon, EyeOffIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

const gamesEnum = z.enum(Game);

const serverConfigFormSchema = z.object({
  name: z.string().min(2).max(50).toLowerCase(),
  rcon_address: z.string().min(2).max(256),
  rcon_port: z.preprocess<number, z.ZodInt, number>((val) => {
    if (typeof val === "string") {
      return Number.parseInt(val, 10);
    }
    return val;
  }, z.int().lte(65535).gt(0)),
  password: z.string().min(0).max(256),
  game: gamesEnum,
  server_name: z.optional(string().min(0).max(1024)),
  game_ip_address: z.optional(ipv4().or(ipv6())),
  game_port: z.optional(
    preprocess<number, z.ZodInt, number>((val) => {
      if (typeof val === "string") {
        return Number.parseInt(val, 10);
      }
      return val;
    }, z.int().lte(65535).gt(0))
  ),
  commands: z.object({
    start: z.string().optional(),
    stop: z.string().optional(),
  }),
});

export function ServerConfigForm({
  className,
  server,
  onClickSubmit = () => {},
  onClickReset = () => {},
  setSelectedServer,
}: {
  className?: string;
  server?: Server;
  onClickSubmit?: () => void;
  onClickReset?: () => void;
  setSelectedServer: React.Dispatch<React.SetStateAction<Server | undefined>>;
}) {
  const [showPassword, setShowPassword] = useState(false);
  let game_ip_address, game_port;
  if (server?.game_address) {
    const list = server?.game_address?.split(":");
    console.log({ list });
    if (list.length == 2) {
      game_ip_address = list[0];
      game_port = Number(list[1]);
    }
  }
  const form = useForm<z.infer<typeof serverConfigFormSchema>>({
    resolver: zodResolver(serverConfigFormSchema),
    defaultValues: {
      name: server?.name || "Local",
      rcon_address: server?.rcon_address || "localhost",
      rcon_port: server?.rcon_port || 2345,
      password: server?.password || "PleaseChangeMe",
      game: server?.game || Game.Factorio,
      server_name: server?.server_name ? server?.server_name : undefined,
      game_ip_address: game_ip_address,
      game_port: game_port,
    },
    resetOptions: {
      keepDirtyValues: false, // user-interacted input will be retained
      keepErrors: false, // input errors will be retained with value update
    },
    mode: "onChange",
  });

  useEffect(() => {
    if (server) {
      let game_ip_address, game_port;
      if (server.game_address) {
        const list = server?.game_address?.split(":");
        if (list.length == 2) {
          game_ip_address = list[0];
          game_port = Number(list[1]);
        }
      }
      form.reset({
        name: server.name,
        rcon_address: server.rcon_address,
        rcon_port: server.rcon_port,
        password: server.password,
        game: server.game,
        server_name: server.server_name,
        game_ip_address,
        game_port,
        commands: server.commands,
      });
    }
  }, [server]);

  function onSubmit(values: z.infer<typeof serverConfigFormSchema>) {
    const game_address = values.game_ip_address
      ? values.game_ip_address +
        ":" +
        (values.game_port ? values.game_port : 34197)
      : undefined;
    console.log({ game_address });
    const new_server: Server = {
      id: values.game + ":" + values.name,
      name: values.name,
      rcon_address: values.rcon_address,
      rcon_port: values.rcon_port,
      password: values.password,
      game: values.game,
      server_name: values.server_name,
      game_address: values.game_ip_address
        ? values.game_ip_address +
          ":" +
          (values.game_port ? values.game_port : 34197)
        : undefined,
      commands: values.commands,
    };
    console.log({ values });
    if (server) {
      console.log("update");
      invoke<Server>("update_server", {
        server: new_server,
        oldServerName: server.name,
      })
        .then((s: Server) => setSelectedServer(s))
        .catch((e) => console.log(e));
    } else {
      console.log("new");
      invoke<Server>("new_server", { server: new_server })
        .then((s: Server) => setSelectedServer(s))
        .catch((e) => console.log(e));
    }
    onClickSubmit();
  }
  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className={cn("space-y-3", className)}
      >
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Name</FormLabel>
              <FormControl>
                <Input placeholder="server name" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="rcon_address"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Server Address</FormLabel>
              <FormControl>
                <Input placeholder="localhost" {...field} />
              </FormControl>
              <FormDescription>
                This is the address of the server, either IP or FQDN
                (google.com).
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="rcon_port"
          render={({ field }) => (
            <FormItem>
              <FormLabel>RCON Port</FormLabel>
              <FormControl>
                <Input placeholder="2345" type="number" {...field} />
              </FormControl>
              <FormDescription>
                The RCON port number not the factorio port number.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <div className="relative">
                  <Input
                    placeholder="supersecret"
                    type={showPassword ? "text" : "password"}
                    {...field}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent"
                    onClick={() => setShowPassword((prev) => !prev)}
                  >
                    {showPassword ? (
                      <EyeIcon className="h-4 w-4" aria-hidden="true" />
                    ) : (
                      <EyeOffIcon className="h-4 w-4" aria-hidden="true" />
                    )}
                    <span className="sr-only">
                      {showPassword ? "Hide password" : "Show password"}
                    </span>
                  </Button>
                </div>
              </FormControl>
              <FormDescription>
                The RCON port number not the factorio port number.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="game"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Game</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a game" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {games.map((game: GameString) => {
                    return (
                      <SelectItem key={game} value={game}>
                        {game}
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />
        {form.getValues().game == Game.Factorio ? (
          <>
            <FormField
              control={form.control}
              name="server_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Server Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="My awesome multiplayer server"
                      {...field}
                    />
                  </FormControl>
                  <FormDescription>Server name if appicable.</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="game_ip_address"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Game Address</FormLabel>
                  <FormControl>
                    <Input placeholder="127.0.0.1" {...field} />
                  </FormControl>
                  <FormDescription>
                    Must be an IP address of the server you are connecting to.
                    Ideally an external address if availble.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="game_port"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Game Port</FormLabel>
                  <FormControl>
                    <Input placeholder="34197" type="number" {...field} />
                  </FormControl>
                  <FormDescription>The game port number.</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="commands.start"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Server Start Command</FormLabel>
                  <FormControl>
                    <Input placeholder="echo 'hello world'" {...field} />
                  </FormControl>
                  <FormDescription>
                    A command to start the server. Must be able to be run in the
                    terminal of the computer which this is running.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="commands.stop"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Server Stop Command</FormLabel>
                  <FormControl>
                    <Input placeholder="echo 'hello world'" {...field} />
                  </FormControl>
                  <FormDescription>
                    A command to stop the server. Must be able to be run in the
                    terminal of the computer which this is running.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
          </>
        ) : (
          ""
        )}
        <div>
          <Button type="submit" variant="default">
            {server ? "Update Server" : "Create Server"}
          </Button>
          <Button
            type="reset"
            variant="outline"
            onClick={() => {
              form.reset();
              onClickReset();
            }}
          >
            Reset
          </Button>
        </div>
      </form>
    </Form>
  );
}
