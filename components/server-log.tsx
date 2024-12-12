"use client";
import { cn } from "@/lib/utils";
import { ComponentProps, forwardRef, useEffect, useRef, useState } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Channel, invoke } from "@tauri-apps/api/core";
import { Log, LogLevel, LogLevelColors, Logs } from "@/types";

interface LogAreaProps {
  className?: string;
}

export default function LogArea({ className }: LogAreaProps) {
  const [logs, setLogs] = useState<Logs>([]);

  const handleSetLogs = (new_log: Log) => {
    setLogs((current_logs) => {
      current_logs.push(new_log);
      return filterLogs(current_logs);
    });
  };
  const lastLineRef = useRef(null);
  useEffect(() => {
    const onEvent = new Channel<Log>();
    onEvent.onmessage = (message) => {
      handleSetLogs(message);
    };

    const subscription_promise = invoke("subscribe_logging_channel", {
      channel: onEvent,
    })
      .then((uuid) => {
        invoke("fetch_all_logs").then((old_logs) => {
          setLogs(old_logs as Logs);
        });

        return uuid;
      })
      .then((uuid) => {
        invoke("log_to_channel", {
          level: "DEBUG",
          target: "UI::Logs",
          message: "Connected to logger.",
          uuid,
        });

        return uuid;
      });
    return () => {
      subscription_promise
        .then((uuid) => invoke("unsubscribe_logging_channel", { uuid }))
        .catch((err) => console.error(err));
    };
  }, []);
  useEffect(() => {
    if (logs.length > 0) {
      // @ts-expect-error(doesn't like the null)
      lastLineRef.current?.scrollIntoView({ behavior: "smooth" }); //Use scrollIntoView to automatically scroll to my ref
    }
  }, [logs.length]);

  return (
    <div className="m-5 w-full flex flex-col">
      <div className="px-2 text-lg font-semibold">Logs</div>
      <ScrollArea
        className={cn(
          "w-full py-2 border flex-auto h-full rounded-md",
          className,
        )}
      >
        <div className="mx-3">Welcome to RCON2.0</div>
        {logs.map((log, index, logs) => (
          <LogLine
            key={`${log.uuid}`}
            level={log.level}
            time={log.time}
            message={log.message}
            location={log.target}
            className="mx-3"
            ref={index + 1 === logs.length ? lastLineRef : null}
          />
        ))}
        <div>&nbsp;</div>
      </ScrollArea>
    </div>
  );
}

interface LogProps extends ComponentProps<"div"> {
  level: LogLevel;
  time: string;
  message: string;
  location: string;
}

export const LogLine = ({
  className,
  level,
  time,
  message,
  location,
  ...props
}: LogProps) => {
  let parsedtime;
  if (time === "") {
    parsedtime = new Date(Date.now());
  } else {
    parsedtime = new Date(Date.parse(time));
  }
  const levelColor: LogLevelColors = {
    INFO: "RoyalBlue",
    ERROR: "red",
    WARNING: "orange",
    CRITICAL: "red",
    DEBUG: "green",
    TRACE: "blue",
  };

  return (
    <div
      className={cn("font-normal tracking-wide", className)}
      suppressHydrationWarning
      {...props}
    >
      {parsedtime.toLocaleTimeString()} -{" "}
      <span style={{ color: levelColor[level] }} className="font-bold">
        {level}
      </span>{" "}
      - {location} - {message}
    </div>
  );
};

const filterLogs = (logs: Logs) => {
  const uuids = new Set();

  const log_filtered = logs.filter((log) => {
    if (uuids.has(log.uuid)) {
      return false;
    }
    uuids.add(log.uuid);
    return true;
  });
  // console.log({ log_filtered });
  return log_filtered;
};
