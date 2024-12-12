"use client";

import { Game, games, GameString, Server } from "@/types";
import { z } from "zod";
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

const gamesEnum = z.nativeEnum(Game);

const serverConfigFormSchema = z.object({
  name: z.string().min(2).max(50).toLowerCase(),
  address: z.string().min(2).max(256),
  port: z.number({ coerce: true }).int().lte(65535).gt(0),
  password: z.string().min(0).max(256),
  game: gamesEnum,
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
  const form = useForm<z.infer<typeof serverConfigFormSchema>>({
    resolver: zodResolver(serverConfigFormSchema),
    defaultValues: {
      name: server ? server.name : "Local",
      address: server ? server.address : "localhost",
      port: server ? server.port : 2345,
      password: server ? server.password : "Testing",
      game: server ? server.game : Game.Factorio,
    },
    resetOptions: {
      keepDirtyValues: false, // user-interacted input will be retained
      keepErrors: false, // input errors will be retained with value update
    },
    mode: "onChange",
  });

  useEffect(() => {
    if (server) {
      form.reset(server);
    }
  }, []);

  function onSubmit(values: z.infer<typeof serverConfigFormSchema>) {
    if (server) {
      console.log("update");
      invoke<Server>("update_server", {
        server: values,
        oldServerName: server.name,
      })
        .then((s: Server) => setSelectedServer(s))
        .catch((e) => console.log(e));
    } else {
      console.log("new");
      invoke<Server>("new_server", { server: values })
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
          name="address"
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
          name="port"
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
