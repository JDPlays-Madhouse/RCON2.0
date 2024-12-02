# RCON2.0

## Minimum Viable Product

- [x] Authenticate with twitch.
- [x] Connect to twitch websocket.
- [x] Handle Chat and Channel Point Reward Events.
- [x] Connect to an Rcon Server.
- [x] Send commands to Rcon Server.
- [ ] Configure through TOML commands to send to RCON server with defined triggers.

## Requirements

- [x] Read twitch events directly i.e. no Streamer.bot etc.
  - [ ] Events including subs/bits/follows/hype trains
  - [x] Channel Points
  - [x] Chat messages
- [ ] Detect certain messages and parse the message for battleship. (Only 1 command)
  - [ ] Example: /muppet_streamer_schedule_explosive_delivery target targetPosition
- [ ] Convert the parsed message into a valid command.
  - [ ] have default values for commands so invalid data with a valid command
        becomes valid command with default data.
- [ ] Read from SteamLabs/streamelements Patreon (own api) and Humble
      notifications and donations.
- [ ] Read from YouTube for chat/subs/memberships/supers.
- [x] Have a Pause button or Api end point to pause for bio breaks.

- [x] Have a RCON app/interface that takes in specific Factorio commands as well
      as any other games.
- [x] Rcon interface needs to take configurations for any rcon server.
- [?] Ensure that the amount of data is below the max per tick amount.

- [ ] Provide visiual feedback through an OBS overlay (website) to give feedback
      on things like the boom factor.
      ![Example of OBS overlay](./docs/Example_visual_feedback.png)
- [ ] From twitch events read hype trains and be able to respond.
  - JDGOESBoom with count down, if redeamed again dead factor goes up and
    restart count down.
- [ ] Be able to add RCON commands, modify, delete, display (CRUD), including
      default values like deadliness.
- [ ] Be able to test when adding commands.
- [x] Output a log with raw output for debugging
  - [x] ESPECIALLY "custom-reward-id" from twitch channel points as ill need
        that data for adding new points rewards through streamer.bot. Or Work
        out what the ID code.
- [ ] Has to support some sort of user comments in the script so i can keep
      track/notes on new code.

### Definitions

- JD Goes Boom factorio becomes a progress bar, like a power up bar.
- Bar goes up from donations being bits or paypal, ALSO sub points (being sub
  points/2 treated as a number) or any other data that is being captured.
- Progress bar has a Nades, then cluster nade, then arty, then nukes for maximum
  effect.

### Assumptions

- One app.
- Rcon needs a frontend display, webpage or app?

## Todo

1. Start with twitch
2. Be able to send commands to server over rcon.
3. Meet in middle with UI.
4. Add more integration.

-----

## Integrations

### Twitch

1. Get a Client ID and Client Secret from [dev.twitch.tv/console/apps/](https://dev.twitch.tv/console/apps/).
2. For the redirect url make sure they are exactly the same
    e.g. `http://localhost:27934/twitch/register`. The port can be changed but
    both the dev console and the config file need to match.
3. Run the application once and the config file will generate.
   1. Windows: ~\AppData\roaming\RCON2.0
   2. Linux: ~/.config/RCON2.0
   3. Apple: ~/Library/Application Support/RCON2.0
4. Add the credentials to auth.twitch.
5. websocket_subscription are the websocket events that you want to track
  defined by
  [twitch docs](https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/).
  Currently implemented are listed below, if there any not listed that you
  want, start an issue
  [JDPlays-Madhouse/RCON2.0/issues](https://github.com/JDPlays-Madhouse/RCON2.0/issues).
   1. channel.chat.message
   2. channel.channel_points_custom_reward_redemption.add
   3. channel.channel_points_custom_reward_redemption.update

### YouTube

Not yet implemented.

### Patreon

Not yet implemented.

