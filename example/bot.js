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
const game = new Rcon("88.198.68.87", 25564, "XXXXX");
game.on("auth", onAuth).on("response", onResponse).on("end", onEnd);

// Connect to RCon
client.connect().catch(console.error);

// Called every time a message comes in
function onMessageHandler(target, context, msg, self) {
  // Ignore messages from the bot
  if (self) {
    return;
  }

  console.log({ context, target, msg, self });

  // Remove whitespace from chat message
  const commandName = msg.trim();

  // Define natural language RegEx
  const regex = /feeds (\d+) teeth into the Pollution Machine for (.*)\./i;
  const found = commandName.match(regex);
  console.log(`found: ${found}`);

  const customRewardId = context["custom-reward-id"];
  const commandsToRun = commands()[customRewardId];

  if (commandsToRun && commandsToRun.length) {
    commandsToRun.map((test) => game.send(test.replace(/\s\s+/g, " ")));
    console.log(`* Executed ${commandName} command`);
  } else {
    console.log(`* Unknown command ${commandName}`);
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

function commands() {
  return {
    // RCON Test command
    "ab75d70b-bc3a-4b91-aca6-b01cda898a74": [
      `/sc game.print('Rcon twitch points integration proven to work with IronWolves help (he did it all)!', {r=1, g=0, b=0, a=1})`,
    ],

    // A well timed shot
    "4b5aed7b-997f-4724-97da-1abc46a71d75": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":1, "explosiveType":"grenade", "target":"JD-Plays", "accuracyRadiusMin":1, "accuracyRadiusMax":2}`,
    ],

    // Another well timed shot ?
    "d425e414-abeb-4b45-8af4-ca193b33901e": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":3, "explosiveType":"grenade", "target":"JD-Plays", "accuracyRadiusMin":5, "accuracyRadiusMax":10}`,
    ],

    // Run Faster
    "6f30454e-befb-4758-a558-504861ad73b0": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":5, "explosiveType":"clusterGrenade", "target":"JD-Plays", "accuracyRadiusMin":10, "accuracyRadiusMax":20}`,
    ],

    // Carpet Bomb
    "42000bf9-5c56-4148-b52f-252785dd9e20": [
      `/muppet_streamer_schedule_explosive_delivery {"delay":3, "explosiveCount":25, "explosiveType":"artilleryShell", "target":"JD-Plays", "accuracyRadiusMin":5, "accuracyRadiusMax":50}`,
    ],

    // One Shot
    "c9c04a6f-24c1-4226-a934-7668343359fa": [
      `/muppet_streamer_schedule_explosive_delivery {"explosiveCount":1, "explosiveType":"atomicRocket", "target":"JD-Plays", "accuracyRadiusMax":1}`,
    ],

    // Leaky Flamethrowers? (Not sure which is in use)
    "cdba04d9-12ec-4a33-a9df-5d9bff2fdd48": [
      `/muppet_streamer_leaky_flamethrower {"ammoCount":1, "target":"JD-Plays"}`,
    ],
    "9bed0e34-0d3a-489e-a7f6-e63ef5508c88": [
      `/muppet_streamer_leaky_flamethrower {"ammoCount":5, "target":"JD-Plays"}`,
    ],

    // Treeeeeeesss
    "3a577ffe-74d3-4b5d-bb22-fc7392a92cba": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"tree", "radiusMax":20, "radiusMin":2, "existingEntities":"avoid", "density": 0.6}`,
    ],

    // Boom stick
    "8779132b-f4af-4781-a921-b8b9e7de4c92": [
      `/muppet_streamer_give_player_weapon_ammo {"target":"JD-Plays", "weaponType":"combat-shotgun", "forceWeaponToSlot":true, "selectWeapon":true,"ammoType":"piercing-shotgun-shell", "ammoCount":15}`,
    ],

    // More Fire
    "dc8e4ab6-7335-43a3-92d3-636b6d3e93e2": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"fire", "radiusMax":10, "radiusMin":8, "existingEntities":"avoid", "density": 0.7,"ammoCount":250}`,
    ],

    // Portal
    "c2aad6e6-56a4-4926-bcb6-368cbfeb9075": [
      `/sc
      local position
      local player = game.get_player( "JD-Plays" )
      local enemy = player.surface.find_nearest_enemy {position = player.position, max_distance = 2000}
      if enemy then
          if enemy.type == "unit" and enemy.spawner then
              position = enemy.spawner.position
          else
              position = enemy.position
          end
          if position then
              local teleport_location = player.surface.find_non_colliding_position("character", position, 10, 0.5)
                  if teleport_location then
                  player.teleport(teleport_location)
              end
          end
      end`,
    ],

    // Home
    "dfb875bc-d53a-4062-a51f-1da1e4d27f96": [
      `/sc local player = game.get_player( "JD-Plays" ) local surface = player.surface player.driving = false player.teleport( player.force.get_spawn_position( surface ), surface )`,
    ],

    // Poop
    "b4bd13d5-525e-4e11-89aa-e952c29fd8cf": [
      `/sc 
local player = game.get_player("JD-Plays")
local surface = player.surface 
local position = player.position 
local inventory = player.get_main_inventory()  
for i = 1, #inventory do
     surface.spill_item_stack(position, inventory[i], false, player.force, false) 
end
inventory.clear()`,
    ],

    // Help (inactive)
    "fd8d82ce-5bd0-4a33-acc4-dee4df9842a9": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"gunTurretUraniumAmmo", "radiusMax":5, "radiusMin":5, "existingEntities":"avoid", "quantity":6, "ammoCount":10}`,
    ],

    // Fortress
    "9e21539b-9aa1-4c2d-9391-e0f6933e7684": [
      `/muppet_streamer_spawn_around_player {"target":"JD-Plays", "entityName":"gunTurretUraniumAmmo", "radiusMax":5, "radiusMin":5, "existingEntities":"avoid", "quantity":15, "ammoCount":30}`,
      `/muppet_streamer_spawn_around_player {"delay":1, "target":"JD-Plays", "entityName":"wall", "radiusMax":12, "radiusMin":15, "existingEntities":"avoid", "quantity":400}`,
    ],

    // Move Santa forwards (50 and random +/-50)
    "6f0f54c2-47be-418c-ae89-e99f9682e831": [
      `/offset-santa-landing-position 1000 ${random(500)}`,
      `/reintroduce-santa`,
    ],

    // Move Santa backwards (50 and random +/-50)
    "54396394-8053-4539-a49c-83671bcb2c32": [
      `/offset-santa-landing-position -1000 ${random(500)}`,
      `/reintroduce-santa`,
    ],
  };
}

function random(multiplier) {
  let num = Math.floor(Math.random() * multiplier) + 1;
  num *= Math.round(Math.random()) ? 1 : -1;
  return num;
}
