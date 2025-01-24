"use client";

import {
  Command,
  CommandPrefixType,
  Game,
  games,
  GameString,
  LuaCommandType,
  Server,
} from "@/types";
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

const commandFormSchema = z.object({
  name: z.string().min(2).max(50).toLowerCase(),
  prefixType: z.nativeEnum(CommandPrefixType),
  customPrefix: z.string().optional(),
  luaCommandType: z.nativeEnum(LuaCommandType),
  command: z.string().optional(),
  commandPath: z.string().optional(),
});

export function CommandForm({
  className,
  command,
  onClickSubmit = () => { },
  onClickReset = () => { },
}: {
  className?: string;
  command?: Command
  onClickSubmit?: () => void;
  onClickReset?: () => void;
}) {
  const form = useForm<z.infer<typeof commandFormSchema>>({
    resolver: zodResolver(commandFormSchema),
    defaultValues: {},
    resetOptions: {
      keepDirtyValues: false, // user-interacted input will be retained
      keepErrors: false, // input errors will be retained with value update
    },
    mode: "onChange",
  });

 

  function onSubmit(values: z.infer<typeof commandFormSchema>) {
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
                <Input placeholder="command name" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="prefixType"
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
                  {Object.keys(CommandPrefixType).map((prefix) => {
                    return (
                      <SelectItem key={prefix} value={prefix}>
                        {prefix}
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
            {command ? "Update Command" : "Create Command"}
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
