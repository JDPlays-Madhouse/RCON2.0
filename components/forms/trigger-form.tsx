"use client";

import { ComparisonOperator, Trigger, TriggerType } from "@/types";
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
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Check, ChevronsUpDown, EyeIcon, EyeOffIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

const tierEnum = z.enum(["Tier1","Tier2","Tier3","Prime","Custom"])

const triggerFormSchema = z.object({
  trigger: z.string().min(2).max(50).toLowerCase(),
  pattern: z.string().max(255).optional(),
  title: z.string().min(1).optional(),
  id: z.string().min(1).optional(),
  tier: tierEnum,
  customTier: z.string().min(1).optional(),
  comparisonOperator: z.nativeEnum(ComparisonOperator).optional(),
});

export function TriggerForm({
  className,
  trigger,
  onClickSubmit = () => { },
  onClickReset = () => { },
}: {
  className?: string;
  trigger?: Trigger;
  onClickSubmit?: () => void;
  onClickReset?: () => void;
}) {
  const initialData = {
    trigger: trigger?.trigger,
    pattern: trigger?.data.pattern,
    title: trigger?.data.title,
    id: trigger?.data.id,
    tier: trigger?.data.tier,
    customTier: trigger?.data.tier,
    comparisonOperator: trigger?.data.comaparison_operator ? trigger?.data.comaparison_operator : ComparisonOperator.Any,
  };
  const [triggerType, setTriggerType] = useState(initialData.trigger);

  const form = useForm<z.infer<typeof triggerFormSchema>>({
    resolver: zodResolver(triggerFormSchema),
    defaultValues: initialData,
    resetOptions: {
      keepDirtyValues: false, // user-interacted input will be retained
      keepErrors: false, // input errors will be retained with value update
    },
    mode: "onChange",
  });

  function onSubmit(values: z.infer<typeof triggerFormSchema>) {
    onClickSubmit();
  }
  console.log({ triggerType });

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className={cn("space-y-3 w-full", className)}
      >
        <FormField
          control={form.control}
          name="trigger"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Trigger Type</FormLabel>
              <Select
                onValueChange={(value) => {
                  field.onChange(value);
                  setTriggerType(
                    TriggerType[value as keyof typeof TriggerType],
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
                            TriggerType[
                            triggertype as keyof typeof TriggerType
                            ],
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
          control={form.control}
          name="pattern"
          render={({ field }) =>
            triggerType === TriggerType.Chat ||
              triggerType === TriggerType.ChatRegex ? (
              <FormItem>
                <FormLabel>
                  Pattern for {triggerType}
                </FormLabel>
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
          control={form.control}
          name="comparisonOperator"
          render={({ field }) => (
            triggerType === TriggerType.Subscription ? 
            <FormItem>
              <FormLabel>Comparison Operator</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
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
            : <></>
          )}
        />
        <FormField
          control={form.control}
          name="tier"
          render={({ field }) => (
            triggerType === TriggerType.Subscription ? 
            <FormItem>
              <FormLabel>Subscription Tier</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
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
            : <></>
          )}
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
