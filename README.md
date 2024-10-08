# RCON2.0

## Requirements

- [ ] Read twitch events directly i.e. no Streamer.bot etc.
  - [ ] Events including subs/bits/follows/hype trains
  - [ ] Channel Points
  - [ ] Chat messages
- [ ] Detect certain messages and parse the message for battleship. (Only 1 command)
  - [ ] Example: /muppet_streamer_schedule_explosive_delivery target targetPosition
- [ ] Convert the parsed message into a valid command.
  - [ ] have default values for commands so invalid data with a valid command becomes valid command with default data.
- [ ] Read from SteamLabs/streamelements Patreon (own api) and Humble notifications and donations.
- [ ] Read from YouTube for chat/subs/memberships/supers.
- [ ] Have a Pause button or Api end point to pause for bio breaks.

- [ ] Have a RCON app/interface that takes in specific Factorio commands so other games can use it.
- [x] Rcon interface needs to take configurations for any rcon server.
- [?] Ensure that the amount of data is below the max per tick amount.

- [ ] Provide visiual feedback through an OBS overlay (website) to give feedback on things like the boom factor.
      ![Example of OBS overlay](./docs/Example_visual_feedback.png)
- [ ] From twitch events read hype trains and be able to respond.
  - JDGOESBoom with count down, if redeamed again dead factor goes up and restart count down.
- [ ] Be able to add RCON commands, modify, delete, display (CRUD), including default values like deadliness.
- [ ] Be able to test when adding commands.
- [ ] Output a log with raw output for debugging
  - [ ] ESPECIALLY "custom-reward-id" from twitch channel points as ill need that data for adding new points rewards through streamer.bot. Or Work out what the ID code.
- [ ] Has to support some sort of user comments in the script so i can keep track/notes on new code.

### Definitions

- JD Goes Boom factorio becomes a progress bar, like a power up bar.
- Bar goes up from donations being bits or paypal, ALSO sub points (being sub points/2 treated as a number) or any other data that is being captured.
- Progress bar has a Nades, then cluster nade, then arty, then nukes for maximum effect.

### Assumptions

- One app.
- Rcon needs a frontend display, webpage or app?

## Todo

1. Start with youtube and twitch
