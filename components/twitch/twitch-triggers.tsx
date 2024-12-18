import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ChannelPointRewardsDashboard from "@/components/twitch/triggers/channel-point-reward-table-dashboard";

export default function TwitchTrigger() {
  return (
    <div className="flex flex-col justify-center items-center gap-2 px-10">
      <div className="text-2xl p-4 font-bold">Triggers</div>
      <Tabs
        defaultValue="channelpointrewards"
        className="w-full items-center justify-center flex flex-col"
      >
        <TabsList>
          <TabsTrigger value="chat">Chat</TabsTrigger>
          <TabsTrigger value="channelpointrewards">
            Channel Point Rewards
          </TabsTrigger>

          <TabsTrigger value="subscribe">Subscription</TabsTrigger>
        </TabsList>
        <TabsContent value="chat">Chat pattern.</TabsContent>
        <TabsContent value="channelpointrewards" className="w-full mx-10">
          <ChannelPointRewardsDashboard />
        </TabsContent>
        <TabsContent value="subscribe">
          Both new subscriptions and resubscriptions.
        </TabsContent>
      </Tabs>
    </div>
  );
}
