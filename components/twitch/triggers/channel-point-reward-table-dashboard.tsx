"use client";

import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChannelPointRewardsTable, {
  CustomChannelPointRewardInfo,
  column,
} from "./channel-point-reward-table";

interface ServerDashBoardProps extends React.ComponentProps<"div"> {}

export default function ChannelPointRewardsDashboard({
  className = "",
  ...props
}: ServerDashBoardProps) {
  const [rewards, setRewards] = useState<CustomChannelPointRewardInfo[]>([]);

  useEffect(() => {
    invoke<CustomChannelPointRewardInfo[]>("get_channel_point_rewards", {
      testing: true,
    }).then((channel_rewards) => {
      setRewards(channel_rewards);
    });
  }, []);

  return (
    <div className={className} {...props}>
      <ChannelPointRewardsTable data={rewards} columns={column} />
    </div>
  );
}
