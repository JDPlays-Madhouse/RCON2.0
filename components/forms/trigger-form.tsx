"use client";

import {
  Command,
  ComparisonOperator,
  Server,
  Trigger,
  TriggerType,
} from "@/types";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
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
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

const tierEnum = z.enum(["Tier1", "Tier2", "Tier3", "Prime", "Custom"]);

const triggerFormSchema = z.object({
  command: z.string().max(255),
  trigger: z.string().toLowerCase(),
  pattern: z.string().max(255).optional(),
  title: z.string().min(1).optional(),
  id: z.string().min(1).optional(),
  tier: tierEnum.optional(),
  customTier: z.string().min(1).optional(),
  comparisonOperator: z.enum(ComparisonOperator).optional(),
});

export function TriggerForm({
  className,
  trigger,
  onClickSubmit = () => {},
  onClickReset = () => {},
  command,
}: {
  className?: string;
  trigger?: Trigger;
  command?: Command;
  server: Server;
  onClickSubmit?: () => void;
  onClickReset?: () => void;
}) {
  const initialData = {
    command: command,
    trigger: trigger?.trigger,
    // @ts-expect-error using the undefined feature of JS
    pattern: trigger?.data.pattern,
    // @ts-expect-error using the undefined feature of JS
    title: trigger?.data.title,
    // @ts-expect-error using the undefined feature of JS
    id: trigger?.data.id,
    // @ts-expect-error using the undefined feature of JS
    tier: trigger?.data.tier,
    // @ts-expect-error using the undefined feature of JS
    customTier: trigger?.data.tier,
    // @ts-expect-error using the undefined feature of JS
    comparisonOperator: trigger?.data.comaparison_operator
      ? // @ts-expect-error using the undefined feature of JS
        trigger?.data.comaparison_operator
      : ComparisonOperator.Any,
  };
  const [triggerType, setTriggerType] = useState(initialData.trigger);
  const [commands, setCommands] = useState<{ [key: string]: Command }>({});

  useEffect(() => {
    invoke<Command[]>("commands").then((c) => {
      console.log(c);
      c.forEach((comm) => (commands[comm.name] = comm));
      setCommands(commands);
    });
  }, []);

  const form = useForm<z.infer<typeof triggerFormSchema>>({
    resolver: zodResolver(triggerFormSchema),
    // @ts-expect-error using the undefined feature of JS
    defaultValues: initialData,
    resetOptions: {
      keepDirtyValues: false, // user-interacted input will be retained
      keepErrors: false, // input errors will be retained with value update
    },
    mode: "onChange",
  });

  function onSubmit(values: z.infer<typeof triggerFormSchema>) {
    onClickSubmit();
    console.log(values);

    // invoke("")
  }

  return (
    <Form {...form}>
      <form
        // @ts-expect-error unknown cause
        // BUG: #6 Unknown type error.
        onSubmit={form.handleSubmit(onSubmit)}
        className={cn("space-y-3 w-full", className)}
      >
        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="command"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Command</FormLabel>
              <Select
                onValueChange={(value) => {
                  field.onChange(value);
                }}
                defaultValue={field.value}
              >
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select the command." />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {Object.keys(commands).map((c) => {
                    return (
                      <SelectItem key={c} value={c}>
                        {c}
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="trigger"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Trigger Type</FormLabel>
              <Select
                onValueChange={(value) => {
                  field.onChange(value);
                  setTriggerType(
                    TriggerType[value as keyof typeof TriggerType]
                  );
                }}
                defaultValue={field.value}
              >
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a trigger type" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {Object.keys(TriggerType).map((triggertype) => {
                    return (
                      <SelectItem
                        key={triggertype}
                        value={triggertype}
                        onSelect={() => {
                          setTriggerType(
                            TriggerType[triggertype as keyof typeof TriggerType]
                          );
                        }}
                      >
                        {TriggerType[triggertype as keyof typeof TriggerType]}
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="pattern"
          render={({ field }) =>
            triggerType === TriggerType.Chat ||
            triggerType === TriggerType.ChatRegex ? (
              <FormItem>
                <FormLabel>Pattern for {triggerType}</FormLabel>
                <FormControl>
                  <Input placeholder="Pattern" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            ) : (
              <></>
            )
          }
        />
        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="title"
          render={({ field }) =>
            triggerType === TriggerType.ChannelPointRewardRedeemed ? (
              <FormItem>
                <FormLabel>Title</FormLabel>
                <FormControl>
                  <Input placeholder="Title" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            ) : (
              <></>
            )
          }
        />

        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="comparisonOperator"
          render={({ field }) =>
            triggerType === TriggerType.Subscription ? (
              <FormItem>
                <FormLabel>Comparison Operator</FormLabel>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a comparison operator." />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    {Object.values(ComparisonOperator).map((op) => {
                      return (
                        <SelectItem key={op} value={op}>
                          {op}
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
                <FormMessage />
              </FormItem>
            ) : (
              <></>
            )
          }
        />
        <FormField
          // @ts-expect-error unknown cause
          // BUG: #6 Unknown type error.
          control={form.control}
          name="tier"
          render={({ field }) =>
            triggerType === TriggerType.Subscription &&
            form.getValues().comparisonOperator != ComparisonOperator.Any ? (
              <FormItem>
                <FormLabel>Subscription Tier</FormLabel>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a subscription tier" />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    {tierEnum.options.map((tier) => {
                      return (
                        <SelectItem key={tier} value={tier}>
                          {tier}
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
                <FormMessage />
              </FormItem>
            ) : (
              <></>
            )
          }
        />

        <div>
          <Button type="submit" variant="default" className="text-foreground">
            {trigger ? "Update Trigger" : "Create Trigger"}
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
