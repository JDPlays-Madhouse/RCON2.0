const Rcon = require("rcon");
const tmi = require("tmi.js");

const username = "jd-plays";
const password = "oauth:";
const channels = ["jdplays"];

// Create a client with our options
const client = new tmi.client({ identity: { username, password }, channels });

// Register our event handlers (defined below)
client.on("message", onMessageHandler);
client.on("connected", onConnectedHandler);

// Connect to Twitch:
client.connect();

// Create an RCon client
const game = new Rcon("43.243.201.67", 51296, "dsasdfsdf");
game.on("auth", onAuth).on("response", onResponse).on("end", onEnd);

// Connect to RCon
game.connect();

let running = true;

// Called every time a message comes in
function onMessageHandler(target, context, msg, self) {
  // Ignore messages from the bot
  if (self) {
    return;
  }

  console.log({ context, target, msg, self });

  // Allow for toggling integration state
  let pause = msg.match(/(HOLD|RESUME)/);
  if (pause && target == '#jdplays' && context.username == 'jdplays') {
    if (pause[1] == 'HOLD') {
      client.say("#jdplays", "*** Integration is paused while JD is away. ***");
      running = false;
    } else {
      client.say("#jdplays", "*** Integration is now enabled. ***");
      running = true;
    }
  }

  // if running process integrations
  if (running) {
    // Each message grows a tree
    if (context.username != 'jdplays' && context.username != 'jdplays' && context.username != 'Nightbot' && context.username != 'streamelements') {
      game.send(`/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"tree", "radiusMax":20, "radiusMin":5, "existingEntities":"avoid", "density": ${msg.length/25000}}`);
    }
  }

  // Respond to channel point commands
  const customRewardId = context["custom-reward-id"];
  const commandsToRun = commands(context, msg)[customRewardId];

  if (typeof commandsToRun !== 'undefined') {
    if (commandsToRun.length) {
      commandsToRun.forEach(cmd => {
        let command = cmd.replace(/%%USERNAME%%/g, context.username) // MUPPET - Puts the username in to command text. Just delete/comment out this line to disable it.
        game.send(command); // MUPPET - replace `command` with `cmd` to undo Username insertion attempt.
        console.log(`* Executed command for reward: ${customRewardId}`);
      });
    } else {
      console.log(`* Unknown command ${commandName}`);
    }
  }
}

function onAuth() {
  console.log("RCon Authed!");
}

function onResponse(str) {
  console.log("Got response: " + str);
}

function onEnd() {
  console.log("Socket closed!");
  process.exit();
}

// Called every time the bot connects to Twitch chat
function onConnectedHandler(addr, port) {
  console.log(`* Connected to ${addr}:${port}`);
}

// Leading spaces in commands are not guaranteed, so put a trailing `;` at the end of each line. Also put the any `/sc ` on the same line as the first Lua code command.
// Use `%%USERNAME%%` to have the name of the redeemer replaced out at run time.
function commands(userstate, message) {
  let x;
  let y;
  try {
    // Captures an X and Y value from the message of whole numbers. Very forgiving of any syntax, finds the last `x` letter and then gets the next number (multiple sequential digits) after this regardless of other characters between, then the same after the last letter `y`. This enables other messages to be before the coordinates.
    const xyRegEx = /.*[,|=|x|y| ](?<x>-?\d+).*[,|=|x|y| ](?<y>-?\d+)/g;
    const parsedXY = [...message.matchAll(xyRegEx)];
    x = parsedXY[0].groups.x;
    y = parsedXY[0].groups.y;
  } catch (ignore) {}
  return {
    // RCON Test command
    "8d9dcabc-2b9c-4e91-873d-6ab2b8b3f6a8": [
      `/sc game.print('It's %%USERNAME%% Fault', {r=1, g=0, b=0, a=1})`,
    ],
	
    // A well timed shot
    "4b5aed7b-997f-4724-97da-1abc46a71d75": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":1, "explosiveType":"grenade", "target":"JD-Plays", "accuracyRadiusMin":1, "accuracyRadiusMax":2}`,
    ],

    // Another well timed shot ? - Same as Run Faster, so should be removed here and the redeem?
    "71ef7616-906e-4d1f-84b3-04ed9202559f": [
      `/sc local grenadeType = "grenade"; if game.get_player("JD-Plays").force.technologies["military-4"].researched then; grenadeType = "clusterGrenade"; end;
      local count = math.max(math.ceil(game.forces["enemy"].evolution_factor * 5),1);
      local maxRadius = count * 3; if grenadeType == "clusterGrenade" then; maxRadius = math.ceil(maxRadius * 1.3); end;
      remote.call('muppet_streamer', 'run_command', 'muppet_streamer_schedule_explosive_delivery', {explosiveCount=count, explosiveType=grenadeType, target="JD-Plays", accuracyRadiusMin=math.floor(maxRadius/2), accuracyRadiusMax=maxRadius});`
    ],

    // Run Faster - SCALING GRENADE
    "ba67de35-aacf-4a9e-88d7-04041f44a1e8": [
      `/sc local grenadeType = "grenade"; if game.get_player("JD-Plays").force.technologies["military-4"].researched then; grenadeType = "clusterGrenade"; end;
      local count = math.max(math.ceil(game.forces["enemy"].evolution_factor * 5),1);
      local maxRadius = count * 3; if grenadeType == "clusterGrenade" then; maxRadius = math.ceil(maxRadius * 1.3); end;
      remote.call('muppet_streamer', 'run_command', 'muppet_streamer_schedule_explosive_delivery', {explosiveCount=count, explosiveType=grenadeType, target="JD-Plays", accuracyRadiusMin=math.floor(maxRadius/2), accuracyRadiusMax=maxRadius});`,
    ],

    // Carpet Bomb
    "e438f413-de1d-447d-bc51-da17a940f8db": [
      `/muppet_streamer_schedule_explosive_delivery {"delay":3, "explosiveCount":35, "explosiveType":"artilleryShell", "target":"JD-Plays", "accuracyRadiusMin":5, "accuracyRadiusMax":80}`,
    ],

    // One Shot - A true nuke.
    "3cd34a22-c15a-484d-83bf-eddc2ef9291b": [
      `/sc local targetPositions = {
  {35, {-15, 6}, {-5, 15}, {-4, -10}},
  {45, {1, -8}, {15, -8}, {30, 2}, {4, 10}, {0, 10}},
  {20, {4, -8}, {0, 10}},
};
local scale = 3;
local playerName = "JD-Plays";

local player = game.get_player(playerName); local playerPosition = player.position;
for _, targetPosition in pairs(targetPositions) do;
  local count = targetPosition[1] * scale;
  for i = 1,count do
    local t=i/count;
    local ps = {};
    for k, v in pairs(targetPosition) do ps[k]=v end;
    for j = #targetPosition,3,-1 do
      for k = 2,j-1 do
        ps[k] = {ps[k][1]*t + ps[k+1][1]*(1-t), ps[k][2]*t + ps[k+1][2]*(1-t)};
      end;
    end;
    remote.call('muppet_streamer', 'run_command', 'muppet_streamer_schedule_explosive_delivery', { explosiveCount=1, explosiveType="custom", customExplosiveType="JGB-16", target=playerName, targetPosition={x=playerPosition.x + ps[2][1]*scale, y=playerPosition.y + ps[2][2]*scale} } );
  end;
end;`,
    ],

    // Leaky Flamethrowers
    "3e1f22bc-66fb-462e-939e-d4057f3d715d": [
      `/muppet_streamer_malfunctioning_weapon {"ammoCount":5, "target":"JD-Plays"}`,
    ],

    // Treeeeeeesss
    "22a60c3d-769e-4274-9b1c-a654ff3cc3d1": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"tree", "radiusMax":20, "radiusMin":6, "existingEntities":"avoid", "density": 0.6}`,
    ],

    // Sticky Trigger Finger - Gives you a Boom Stick, fires off some EXTRA rounds, but lets you keep the origional rounds.
    "d1e4bb1a-396d-44dd-8f4e-fc6ec396d0fa": [
      `/sc remote.call('muppet_streamer', 'run_command', 'muppet_streamer_give_player_weapon_ammo', {delay=0, target="JD-Plays", weaponType="combat-shotgun", forceWeaponToSlot=true, selectWeapon=true, ammoType="piercing-shotgun-shell", ammoCount=15});
      remote.call('muppet_streamer', 'run_command', 'muppet_streamer_malfunctioning_weapon', {delay=0.1, target="JD-Plays", ammoCount=3, weaponType="combat-shotgun", ammoType="piercing-shotgun-shell"});`
    ],

    // Firewall
    "06fdb9c4-1a57-4dbd-93fc-d6f9bb43c54e": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"fire", "radiusMax":10, "radiusMin":8, "existingEntities":"avoid", "density": 0.7,"ammoCount":250}`,
    ],

    // Portal - only walkable targets: nearest biter nest within 3k tiles, otherwise random within 500-1,000 tiles.
    "6970482e-904e-4202-a4f8-2dd0b1b03861": [
	  `/muppet_streamer_teleport {"target":"JD-Plays", "destinationType":"biterNest", "maxDistance": 3000, "reachableOnly": true, "backupTeleportSettings": {"target":"JD-Plays", "destinationType":"random", "minDistance": 500, "maxDistance": 1000, "reachableOnly": true} }`,
    ],

    // Home
    "cae87f07-4a32-438b-add2-55943cab6545": [
      `/muppet_streamer_teleport {"target":"JD-Plays", "destinationType":"spawn"}`,
    ],
	
    // Battleships - the x and y values are abstracted from the redeem message by this integration automatically.
    "170b115b-6eb5-4b62-b83d-bc08f2b142b8": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":45, "explosiveType":"artilleryShell", "target":"JD-Plays", "accuracyRadiusMax":25,"targetOffset": {"x":` + (x == null ? 0 : x) + `, "y":` + (y == null ? 0 : y) + `}}`,
    ],

    // MiniNuke - same unsure on value
    "88a47fbe-0f41-42f0-8fdb-ec1d1f9c552f": [
      `/jd_goes_boom JD-Plays 2`,
    ],
	
    // Decon
    "b77bb703-b32d-46a9-887b-98f300e0d36d": [
      `/sc local radius = 25;
      local player = game.get_player("jd-plays");
      if player then;
        local force = player.force;
        for k,v in pairs(player.surface.find_entities_filtered{position=player.position, radius=radius}) do;
          v.order_deconstruction(force);
        end;
      end;`,
    ],

    // Mass Decon 
    "c82628fb-808d-405d-9067-314c29365c08": [
      `/sc local radius = 10;
      for _, player in pairs(game.connected_players) do;
        local force = player.force;
        for k,v in pairs(player.surface.find_entities_filtered{position=player.position, radius=radius}) do;
          v.order_deconstruction(force);
        end;
      end;`,
    ],

    // Poop
    "ffb332b7-34e0-4ce5-b32c-056472de3529": [
`/muppet_streamer_player_drop_inventory {"target":"JD-Plays", "quantityType":"startingPercentage", "quantityValue":100, "gap":1, "occurrences":1, "markForDeconstruction": true, "includeArmor": false, "includeWeapons": false}`,
    ],
	
    // Pants on fire
    "5082667b-c85a-49e8-9260-0ed73936addd": [
      `/muppet_streamer_pants_on_fire {"target":"JD-Plays", "flameCount": 250, "duration": 45}`,
    ],
	
	// group gastro
    "9c713aa7-cf45-437d-9f7c-dc44b9136be8": [
      `/sc for i, player in pairs(game.connected_players) do;
        remote.call('muppet_streamer', 'run_command', 'muppet_streamer_player_drop_inventory', {delay = i/4, target=player.name, quantityType="startingPercentage", quantityValue=100, gap=1, occurrences=1, markForDeconstruction=true, includeArmor=false, includeWeapons=false});
      end;`,
    ],
	
    // runs down your leg
    "5bf18721-3ba6-44b6-83b7-7c95efdc9e18": [
      `/muppet_streamer_player_drop_inventory {"target":"JD-Plays", "quantityType":"startingPercentage", "quantityValue":10, "gap":5, "occurrences":10} "dropOnBelts":true`,
    ],
	
    // Mighty Mighty Power Rangers
    "e0489a01-69f9-43be-a4ae-4dd73f1149c0": [
      `/muppet_streamer_call_for_help {"target":"JD-Plays", "whitelistedPlayerNames": "foxhound590,billbo99,muppet9010,Stinson_5,Huff", "arrivalRadius":5, "callSelection": "random", "activePercentage": 100}`,
    ],
	
    // combat
    "5ca58e65-e14e-457b-b1ba-9b961e01791d": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"gunTurretPiercingAmmo", "radiusMax":5, "radiusMin":5, "existingEntities":"avoid", "quantity":6, "ammoCount":10}`,
    ],
	
    // Inventory Shuffle (nice) - excludes armor.
    "38a933d2-8017-4e86-a275-0fa4a366f8d4": [
      `/muppet_streamer_player_inventory_shuffle {"includedPlayers":"[ALL]", "includeArmor":false}`,
    ],
	
    // Inventory Shuffle (bad) - disassembles equipment from armor.
    "a9206e00-ab0d-44be-bab7-4068dc26c3a0": [
      `/muppet_streamer_player_inventory_shuffle {"includedPlayers":"[ALL]", "includeArmor":true, "extractArmorEquipment":true}`,
    ],
	
    // UFOs
    "4b46a792-89e6-443a-ae6d-31c11bee8536": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "force": "muppet_streamer_enemy", "entityName":"distractorBot", "radiusMax":8, "radiusMin":2, "existingEntities":"overlap", "quantity": 10, "followPlayer": true}`,
    ],
	
    // JD can drive anything
    "f1f6f6a9-780d-4355-bb42-930656c150d4": [
      `/muppet_streamer_aggressive_driver {"target":"JD-Plays", "duration":60, "control": "random", "teleportDistance": 300}`,
    ],
	
    // Call for HELP!!!!!
    "c95f6365-f240-41ab-8319-f0d56570dcc5": [
      `/muppet_streamer_call_for_help {"target":"JD-Plays", "blacklistedPlayerNames": "foxhound590,billbo99,muppet9010,Stinson_5,Huff", "arrivalRadius":10, "callSelection": "nearest", "number": 3}`,
    ],
	
    // AIR Support
    "b526a2f1-cc63-4c2a-ba59-7d7f602e9ba1": [
      `/sc local playerName = "JD-Plays"; local deathSpread = 300;
      game.get_player(playerName).character_maximum_following_robot_count_bonus = 100;
      local bots = remote.call('muppet_streamer', 'run_command', 'muppet_streamer_spawn_around_player', {target=playerName, entityName="defenderBot", radiusMax=10, radiusMin=10, existingEntities="overlap", quantity=40, followPlayer=true});
      local ttl = bots[1].time_to_live;
      for _, bot in pairs(bots) do;
        bot.time_to_live = ttl + math.random(-deathSpread/2,deathSpread/2);
      end;`,
    ],
	
    // Fortress - Dynamically sized based on evo level and ammo type based on research.
    "ec3aaa03-adbd-49f2-9779-6b52093fc296": [
      `/sc local turretType = "gunTurretRegularAmmo"; if game.get_player("JD-Plays").force.technologies["uranium-ammo"].researched then; turretType = "gunTurretUraniumAmmo"; elseif game.get_player("JD-Plays").force.technologies["military-2"].researched then; turretType = "gunTurretPiercingAmmo"; end;
      local size = math.max(math.ceil(game.forces["enemy"].evolution_factor * 10),1);
      remote.call('muppet_streamer', 'run_command', 'muppet_streamer_spawn_around_player', {target="JD-Plays", entityName=turretType, radiusMin=math.floor(size/3), radiusMax=2+math.ceil(size/3), existingEntities="avoid", quantity=size, ammoCount=size*5});
      remote.call('muppet_streamer', 'run_command', 'muppet_streamer_spawn_around_player', {target="JD-Plays", entityName="wall", radiusMin=math.ceil(size*0.5) + 4, radiusMax=math.ceil(size*0.7) + 4, existingEntities="avoid", quantity=(size*25)+25});
      game.print("%%USERNAME%% gave JD a fortress with " .. size .. " guns, in a effort to save his ass");`
	],
	
    // badfox
    "d041c611-eca9-4f6b-ac88-0317ab83d70a": [
      `/muppet_streamer_call_for_help {"target":"JD-Plays", "whitelistedPlayerNames": "foxhound590", "arrivalRadius":5, "callSelection": "random", "activePercentage": 100}`,
    ],
	
    // Its Dark
    "9074bdec-483c-4f30-9764-2400e496bd12": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"custom", "customEntityName": "camp-fire", "force": "muppet_streamer_enemy", "radiusMax":15, "radiusMin":3, "existingEntities":"avoid", "quantity":5}`,
    ],
	
    // Fire Pits
    "dcd7f0df-dcc7-4cf4-918e-c9b8f5f46227": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"custom", "customEntityName": "camp-fire", "force": "muppet_streamer_enemy", "radiusMax":15, "radiusMin":3, "existingEntities":"avoid", "density": 0.2}`,
    ],
	
    // Toasted Marshmellows for all - Everyone gets a camp fire.
    "09fa7cb0-45a5-4788-82b8-1de36d7228ae": [
      `/sc for _, player in pairs(game.connected_players) do;
        remote.call('muppet_streamer', 'run_command', 'muppet_streamer_spawn_around_player', {target=player.name, entityName="custom", customEntityName="camp-fire", force="muppet_streamer_enemy", radiusMax=7, radiusMin=3, existingEntities="avoid", quantity=1});
      end;`,
    ],

    // Multiplayer Snake - Everyone gets pants on fire with biter spit.
    "f9c71f9a-94fc-4d9c-acc6-d48028f4f836": [
      `/sc for _, player in pairs(game.connected_players) do;
        rendering.draw_text({text="run !!! multiplayer snake incomming", surface=player.surface, target=(player.vehicle or player.character or player.position), color={1,1,1,1}, time_to_live=300, players={player}, alignment="center", vertical_alignment="top", scale_with_zoom=true});
        remote.call('muppet_streamer', 'run_command', 'muppet_streamer_pants_on_fire', {target=player.name, delay=5, duration=90, flameCount=250, suppressMessages=true});
      end;`,
    ],
	
    // JD Stand Still - just make the stasis effect on JD instantly. Is a small area of affect with 20 second timer (mod default).
    // Disable the stasis weapons in Stasis mod startup settings to stop players crafting them. Duration and affect forces also controlled by mod startup settings.
    "3017801e-3ace-4c62-b322-aa76f08b6e37": [
      `/sc local player = game.get_player("JD-Plays"); local position = player.position;
      player.surface.create_entity({name="stasis-grenade", position=position, force="enemy", target=position, speed=0, max_range=0});`,
    ],
	
    // JD Bullet Time - slowdown capsules instantly affect on JD and the wider area around him.
    "da97e97c-5940-4b78-a6c3-55728708d745": [
      `/sc local player = game.get_player("JD-Plays"); local surface = player.surface; local position = player.position; local spread = 15; local diagonalSpread = spread * 0.7; local thisPosition;
      thisPosition = {x=position.x, y=position.y}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x, y=position.y-spread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x+diagonalSpread, y=position.y-diagonalSpread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x+spread, y=position.y}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x+diagonalSpread, y=position.y+diagonalSpread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x, y=position.y+spread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x-diagonalSpread, y=position.y+diagonalSpread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x-spread, y=position.y}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});
      thisPosition = {x=position.x-diagonalSpread, y=position.y-diagonalSpread}; surface.create_entity({name="slowdown-capsule", position=thisPosition, force="muppet_streamer_enemy", target=thisPosition, speed=0, max_range=0});`,
    ],

    // Who Farted - 50% cance per online player gets a poison capsule explode instantly on them.
    "a90d0965-272a-41c7-a715-10a59a153b5e": [
      `/sc for _, player in pairs(game.connected_players) do;
        if math.random() > 0.5 then;
          local position = player.position
          player.surface.create_entity({name="poison-capsule", position=position, force="muppet_streamer_enemy", target=position, speed=0, max_range=0});
        end;
      end;`,
    ],
	
	// God Clearing His Throat - Lots of spit at JD.
    "65e161f9-e3e3-479a-896a-2470d2606611": [
      `/muppet_streamer_schedule_explosive_delivery {"delay":3, "explosiveCount":435, "explosiveType":"largeSpit", "target":"JD-Plays", "accuracyRadiusMin":5, "accuracyRadiusMax":60}`
    ],

    // Pet Biter  Takes in %%USERNAME%% from JD integration tool. --
    "5dd1d118-7dd2-4145-89bf-e906164d99ba": [
      `/sc local playerName = "JD-Plays"; local biterName = "%%USERNAME%%"; local biterDetailsColor = {1.0,0.2,0.2,1.0}; local biterDetailsSize = 1.5; local biterDeathMessageDuration = 1800; local biterDeathMessagePrint = "master";  local closenessRange = 5; local exploringMaxRange = 15; local combatMaxRange = 50;
local biterTypeSelection = {[0]="medium-biter", [0.2]="big-biter", [0.5]="behemoth-biter"}; local biterBonusHealthSelection = {[0]=75, [0.2]=375, [0.5]=3000, [0.9]=12000};
local biterStatusMessages_Wondering = {"Waiting for you to do something interesting", "Supervising you"}; local biterStatusMessages_Following = {"Walkies", "Are we there yet ?"}; local biterStatusMessages_Fighting = {"Off catching you a present", "Playing with new friends "}; local biterStatusMessages_CallBack = {"Aww bed time already ?", "Bringing you back a bloodied present"}; local biterStatusMessages_GuardingCorpse = {"Defending your corpse for your return", "Too dumb to notice you've died"}; local biterStatusMessages_Dead = {"They were a loyal dumb beast to the end", "Has gone to a better place to forever chase small squishy creatures"};
local player = game.get_player(playerName); if player == nil then; return; end;
local surface, playerPosition = player.surface, player.position;
local biterType, biterBonusHealthMax, biterHealingPerSecond, biterMaxHealth, biterPrototype; local enemyEvo = game.forces["enemy"].evolution_factor; for evoReq, thisBiterType in pairs(biterTypeSelection) do; if evoReq <= enemyEvo then; biterType = thisBiterType; end; end; for evoReq, thisBonusHealth in pairs(biterBonusHealthSelection) do; if evoReq <= enemyEvo then; biterBonusHealthMax = thisBonusHealth; end; end; if biterBonusHealthMax > 0 then; biterPrototype = game.entity_prototypes[biterType]; biterHealingPerSecond = biterPrototype.healing_per_tick*60; biterMaxHealth = biterPrototype.max_health; end;
local biterSpawnPosition = surface.find_non_colliding_position(biterType, playerPosition, 10, 0.1); if biterSpawnPosition == nil then; return; end;
local biter = surface.create_entity({name=biterType, position=biterSpawnPosition, force=player.force}); if biter == nil then; return; end;
biterName = biterName or ""; local biterNameRenderId, biterStateRenderId, biterHealthRenderId; if biterDetailsSize > 0 then; local stickerBox = biterPrototype.sticker_box --[[@as BoundingBox]]; local stickerBoxLargestSize = math.max(stickerBox.right_bottom.x - stickerBox.left_top.x, stickerBox.right_bottom.y - stickerBox.left_top.y) * 1.5; if biterBonusHealthMax > 0 then; biterHealthRenderId = rendering.draw_sprite({sprite = 'virtual-signal/signal-white', tint = {0.0, 200.0, 0.0}, x_scale = 0.6 * 8, y_scale = 0.6, render_layer = 'light-effect', target = biter, target_offset = {0.0, (-0.5 - 0.5 - (biterDetailsSize/2) - stickerBoxLargestSize) --[[@as float]]}, surface = biter.surface, vertical_alignment="bottom"}); end; biterNameRenderId = rendering.draw_text({text=biterName, surface=surface, target=biter, target_offset = {0.0, (-0.5 - stickerBoxLargestSize) --[[@as float]]}, color=biterDetailsColor, alignment="center", vertical_alignment="bottom", scale=biterDetailsSize}); biterStateRenderId = rendering.draw_text({text=biterStatusMessages_Wondering[math.random(#biterStatusMessages_Wondering)], surface=surface, target=biter, color=biterDetailsColor, alignment="center", vertical_alignment="top", scale=biterDetailsSize}); end;
biter.ai_settings.allow_destroy_when_commands_fail = false; biter.ai_settings.allow_try_return_to_spawner = false; biter.ai_settings.do_separation = true;
local followPlayerFunc = function(data);
    data = data --[[@as BiterPet_Data]]; if not data._surface.valid then; return; end;
    if not data._biter.valid then; if data._biterDetailsSize > 0 then; rendering.draw_text({text="RIP "..data._biterName, surface=data._surface, target=data._lastPosition, color=data._biterDetailsColor, alignment="center", vertical_alignment="bottom", scale=data._biterDetailsSize, time_to_live = data._biterDeathMessageDuration}); rendering.draw_text({text=data._biterStatusMessages_Dead[math.random(#data._biterStatusMessages_Dead)], surface=data._surface, target=data._lastPosition, color=data._biterDetailsColor, alignment="center", vertical_alignment="top", scale=data._biterDetailsSize, time_to_live = data._biterDeathMessageDuration}); end; local deathMessage = "RIP "..data._biterName.." : [gps="..math.floor(data._lastPosition.x)..","..math.floor(data._lastPosition.y)..","..data._surface.name.."]"; if data._biterDeathMessagePrint == "master" then; data._player.print(deathMessage); elseif data._biterDeathMessagePrint == "everyone" then; game.print(deathMessage); end; if data._debug then; data._player.print("biter died - TEST - "..game.tick); end; return; end; data._lastPosition = data._biter.position;
    if data._biterBonusHealthMax > 0 then; local biterHealth = data._biter.health; local healthBelowMax = data._biterMaxHealth - biterHealth; local updateHealthBar = false; if healthBelowMax > 0 then; updateHealthBar = true; local healthToRecover = math.min(healthBelowMax, data._biterBonusHealthCurrent); if healthToRecover > 0 then; data._biter.health = biterHealth + healthToRecover; data._biterBonusHealthCurrent = data._biterBonusHealthCurrent - healthToRecover; biterHealth = biterHealth + healthToRecover end; elseif data._biterBonusHealthCurrent < data._biterBonusHealthMax then; data._biterBonusHealthCurrent = math.min(data._biterBonusHealthCurrent + data._biterHealingPerSecond, data._biterBonusHealthMax); updateHealthBar = true; end; if updateHealthBar and data._biterHealthRenderId then; local x_scale_multiplier = (biterHealth + data._biterBonusHealthCurrent) / (data._biterMaxHealth + data._biterBonusHealthMax); rendering.set_x_scale(data._biterHealthRenderId, 0.6 * 8 * x_scale_multiplier); rendering.set_color(data._biterHealthRenderId, {math.floor(255 - 255 * x_scale_multiplier) --[[@as float]], math.floor(200 * x_scale_multiplier) --[[@as float]], 0.0}); end; end;
    if not data._player.valid then; return; end; local targetEntity = data._player.vehicle or data._player.character;
    if targetEntity == nil then;
        if data._hasOwner then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_GuardingCorpse[math.random(#data._biterStatusMessages_GuardingCorpse)]); end; if data._debug then; data._player.print("guarding player corpse - TEST - "..game.tick); end; data._hasOwner = false; end;
        remote.call("muppet_streamer", "add_delayed_lua", 60, data._followPlayerFuncDump, data); return;
    end;
    local biterPosition, targetEntityPosition = data._biter.position, targetEntity.position; local biterPlayerDistance = (((biterPosition.x - targetEntityPosition.x) ^ 2) + ((biterPosition.y - targetEntityPosition.y) ^ 2)) ^ 0.5;
    if not data._hasOwner then;
        if biterPlayerDistance < data._exploringMaxRange then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_Wondering[math.random(#data._biterStatusMessages_Wondering)]); end; if data._debug then; data._player.print("biter reclaimed by player - TEST - "..game.tick); end; data._hasOwner = true; data._calledBack = false; data._following = false; end;
        remote.call("muppet_streamer", "add_delayed_lua", 60, data._followPlayerFuncDump, data); return;
    end;
    if biterPlayerDistance > data._exploringMaxRange + 1 then;
        if data._biter.distraction_command ~= nil then;
            if biterPlayerDistance > data._combatMaxRange then;
                if not data._calledBack then;
                    if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_CallBack[math.random(#data._biterStatusMessages_CallBack)]); end;
                    data._biter.set_command({type=defines.command.go_to_location, destination_entity=targetEntity, radius=data._closenessRange, distraction=defines.distraction.none}); data._calledBack = true; data._fighting = false; if data._debug then; data._player.print("biter called back to player - TEST - "..game.tick); end;
                end;
            else;
                if not data._fighting then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_Fighting[math.random(#data._biterStatusMessages_Fighting)]); end; if data._debug then; data._player.print("biter started fighting far away - TEST - "..game.tick); end; data._fighting = true; end;
            end;
        elseif not data._calledBack then;
            data._biter.set_command({type=defines.command.go_to_location, destination_entity=targetEntity, radius=data._closenessRange}); if data._debug then; data._player.print("follow me - TEST".. " - "..game.tick); end;
            if not data._following then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_Following[math.random(#data._biterStatusMessages_Following)]); end; data._following = true; data._fighting = false; end;
        end;
    else;
        if data._biter.distraction_command ~= nil then;
            if not data._fighting then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_Fighting[math.random(#data._biterStatusMessages_Fighting)]); end; if data._debug then; data._player.print("biter started fighting near by - TEST - "..game.tick); end data._fighting = true; end;
        else;
            if data._calledBack or data._following or data._fighting then; if data._biterStateRenderId then; rendering.set_text(data._biterStateRenderId, data._biterStatusMessages_Wondering[math.random(#data._biterStatusMessages_Wondering)]); end; if data._debug then; data._player.print("biter either: stopped fighting near player, caught up or returned to player - TEST - "..game.tick); end; data._calledBack = false; data._following = false; data._fighting = false; end;
        end;
    end;
    remote.call("muppet_streamer", "add_delayed_lua", 60, data._followPlayerFuncDump, data);
end;
local data = { _player=player, _biter=biter, _surface=surface, _biterBonusHealthMax=biterBonusHealthMax, _biterBonusHealthCurrent=biterBonusHealthMax, _biterHealingPerSecond=biterHealingPerSecond, _biterMaxHealth=biterMaxHealth, _followPlayerFuncDump=string.dump(followPlayerFunc), _closenessRange=closenessRange, _exploringMaxRange=math.max(exploringMaxRange, 10+closenessRange), _combatMaxRange=combatMaxRange, _calledBack=false, _following=false, _biterName=biterName, _hasOwner=true, _lastPosition=biter.position, _debug=false, _biterDetailsSize=biterDetailsSize, _biterDetailsColor=biterDetailsColor, _biterNameRenderId=biterNameRenderId, _biterStateRenderId=biterStateRenderId, _biterHealthRenderId=biterHealthRenderId, _biterDeathMessageDuration=biterDeathMessageDuration, _biterDeathMessagePrint=biterDeathMessagePrint, _biterStatusMessages_Wondering=biterStatusMessages_Wondering, _biterStatusMessages_Following=biterStatusMessages_Following, _biterStatusMessages_Fighting=biterStatusMessages_Fighting, _biterStatusMessages_CallBack=biterStatusMessages_CallBack, _biterStatusMessages_GuardingCorpse=biterStatusMessages_GuardingCorpse, _biterStatusMessages_Dead=biterStatusMessages_Dead };
remote.call("muppet_streamer", "add_delayed_lua", 0, data._followPlayerFuncDump, data); local version = "1.0.1";`,
    ],

    // Rise from the dead
    "e23f6585-bcfb-45a9-8f35-8b3e7c01497c": [
      `/sc 
local player_name = "JD-Plays";
local time = 300; --[[time in seconds that this will apply for (60)]]
local interval = 5; --[[interaval in seconds of checking if someone is dead (5)]]
local search_range = 100;

function revive_players(data);

    --[[game.print("Rise");]]
    
    local player_name = data.player_name;
    local search_range = data.search_range;
    
    local target_player = game.get_player(player_name);
    
    --[[test is redone every iteration to account for the player leaving during the delay]]
    if target_player then;

        local force = target_player.force;
        local spawn_point = target_player.position;
        
        for _ , player in pairs(force.connected_players) do;
        
            --[[do nothing for living players]]
            if player.ticks_to_respawn then;
                
                --[[revive the dead]]
                player.ticks_to_respawn = nil;
                
                --[[players are to be revived at target_player's location]]
                local position = target_player.surface.find_non_colliding_position("character",spawn_point,search_range,0.25);
                
                --[[if no position is found then leave the player at spawn]]
                if position then;
                    player.teleport(position, target_player.surface);
                else;
                    game.print("No position found for " .. player.name .. ". They have been left at spawn");
                end;
                
            end;
        end;
    else;
        --[[runs if target_player is is not found]]
        game.print("Failed to retrive player by the name " .. player_name .. ".");
		game.print("    Player may be offline or player_name may be misspelled");
    end;
end;

function announce_end();
    game.print("The fallen may rest now");
end;

game.print("The fallen rise again.");

local data = {player_name = player_name , search_range = search_range};

--[[call revive_players every 'interval' secs until time 'time'
counting down instead of up is to deal with the edge case where 'interval' does not evenly divide 'time', 
in this case the fisrt interval will be shorter then expected.]]
for t= time, 0 , -interval do;
    remote.call("muppet_streamer", "add_delayed_lua", t*60, string.dump(revive_players), data);
end;

--[[if 'interval' does not evenly divide 'time' we need to call the first revive ourself]]
if time % interval == 0 then revive_players(data) end;

--[[tell players when the effect ends]]
remote.call("muppet_streamer", "add_delayed_lua", time*60, string.dump(announce_end));`,
    ],

    // JD Was here
    "32f8dac0-b479-4136-8b4b-7227ea49b9af": [
      `/sc local targetPositions = {
  {35, {-15, 6}, {-5, 15}, {-4, -10}},
  {45, {1, -8}, {15, -8}, {30, 2}, {4, 10}, {0, 10}},
  {20, {4, -8}, {0, 10}},
};
local scale = 0.5;
local playerName = "JD-Plays";

local player = game.get_player(playerName); local playerPosition = player.position;
for _, targetPosition in pairs(targetPositions) do;
  local count = targetPosition[1] * scale;
  for i = 1,count do
    local t=i/count;
    local ps = {};
    for k, v in pairs(targetPosition) do ps[k]=v end;
    for j = #targetPosition,3,-1 do
      for k = 2,j-1 do
        ps[k] = {ps[k][1]*t + ps[k+1][1]*(1-t), ps[k][2]*t + ps[k+1][2]*(1-t)};
      end;
    end;
    remote.call('muppet_streamer', 'run_command', 'muppet_streamer_schedule_explosive_delivery', { explosiveCount=1, explosiveType="custom", customExplosiveType="JGB-6", target=playerName, targetPosition={x=playerPosition.x + ps[2][1]*scale, y=playerPosition.y + ps[2][2]*scale} } );
  end;
end;`,
    ],
	
    // Where Did I park My car?
    "58d06f9d-ba2a-4df7-8c74-d99f66de99ca": [
      `/sc 
player_name = "JD-Plays"
player = game.get_player(player_name);
search_range = 200;
vehical_type = "car"

if player then

    --[[make the player ready to resive new vehical]]
    player.ticks_to_respawn = nil;
    player.driving = false;

    --[[first look for a open place big enough to fit a rocket silo, then if that fails a car. if that fails again use the current player position]]
    position = player.surface.find_non_colliding_position("rocket-silo",player.position,search_range,0.5) or
        player.surface.find_non_colliding_position(vehical_type,player.position,search_range,0.5) or 
        player.position;
    player.teleport(position);

    --[[make the car]]
    car = player.surface.create_entity{name=vehical_type,position=player.position,force=player.force,player=player,direction = player.walking_state.direction};

    --[[give the car fuel]]
    if car then car.get_fuel_inventory().insert("nuclear-fuel") end

    --[[activate aggressive driver]]
    remote.call('muppet_streamer', 'run_command', 'muppet_streamer_aggressive_driver', {target=player_name, duration=30, control="full", teleportDistance=5});

    game.print("Speeeeeeeeeeeeed");
else
    --[[runs if player is nil]]
    game.print("Failed to retrive player by the name " .. player_name .. ".");
	game.print("    Player may be offline or player_name may be misspelled");
end`,
    ],
	
    //You only NEED 1 HP
    "892c1684-7730-430d-a12f-e25bdc458b3a": [
      `/sc 
player_name = "JD-Plays";
player = game.get_player(player_name);
range = 50;

function change_entity_health(entity);

	--[[do nothing if this entity is not soposed to be damaged]]
	if not entity.health or not entity.destructible then;
		return;
	end;
	
	entity.health = 1;
	grid = entity.grid;
	
	--[[remove all shields]]
	if grid then;
		for _ , equipment in pairs(grid.equipment) do;
			if equipment.type == "energy-shield-equipment" then;
				equipment.shield = 0;
			end;
		end;
	end;
	return;
end;

if player then;

	surface = player.surface;
	entities = surface.find_entities_filtered{position = player.position, radius = range} ;
	
	for _ , entity in pairs(entities) do;
		--[[remove the health of the entity]]
		change_entity_health(entity);
	end;

    game.print("Sudden Death has begun");
else;
    --[[runs if player is nil]]
    game.print("Failed to retrive player by the name " .. player_name .. ".");
	game.print("    Player may be offline or player_name may be misspelled");
end;`,
    ],
	
    // Tanky Spider
    "cfcf8e9b-a6ea-4ab4-985c-9f3d16ba2822": [
      `/spider_give_ammo north explosiveCannonShell 20 10%`,
    ],
	
    // Who gave the spider depleted uranium shells?
    "1d5df2f7-74df-49bc-87e1-4a6fe91864f4": [
      `/spider_give_ammo north uraniumCannonShell 20 10%`,
    ],
	
    // ARMED to the Teeth, do spiders have teeth?
    "1638197d-f81e-4314-b1e4-b742356cd6a1": [
      `/spider_give_ammo north explosiveUraniumCannonShell 20 10%`,
    ],
	
    // Long Range Spider
    "62c0742b-4d98-49a1-b6c3-367dd91093f2": [
      `/spider_give_ammo north artilleryShell 10 4`,
    ],
	
    // Move Santa forwards (50 and random +/-50)
    "xxxxx": [
	  `/offset-santa-landing-position 1000 ${random(500)}`, `/reintroduce-santa`
	],
	
    // Move Santa backwards (50 and random +/-50)
    "xxxx": [
	  `/offset-santa-landing-position -1000 ${random(500)}`, `/reintroduce-santa`
	],
  };
}


function random(multiplier) {
  let num = Math.floor(Math.random() * multiplier) + 1;
  num *= Math.round(Math.random()) ? 1 : -1;
  return num;
}