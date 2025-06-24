# cli-dungeon
Inspired by dice-based table-top RPGs, `cli-dungeon` brings dungeon crawling to your terminal with a twist of automation and scripting.

```
New encounter: Rat, Swordington
Swordington attacked Rat: Swordington missed
Swordington attacked Rat: Swordington missed
Rat attacked Swordington: Rat missed
Swordington attacked Rat: 3 damage
Swordington attacked Rat: Swordington missed
Rat attacked Swordington: Rat missed
Swordington attacked Rat: 5 damage
Rat died
+5 gold
```

## Introduction
In cli-dungeon you control your character through the depths of your terminal, battling monsters and foes alike in a quest for loot.
Develop a script to determine the optimal action in your encounters with the evil.

## Installation
### From Releases
You can download an executable in the Releases page.
This needs to be put on your `$PATH`.

### With cargo
Dependencies: `cargo`
```bash
cargo install --git https://github.com/SofusA/cli-dungeon
```

## Setup
There is a change of *something happens* whenever `cli-dungeon play` is run, and your character advancis their quest for loot.
`cli-dungeon` is meant to be set up as part of your cli prompt, so that everytime you run a command, something might happen. 

A simple setup would be to alias `cd`:
- fish
```fish
function cd
    cli-dungeon play
    builtin cd $argv
end
```
- bash

```bash
cd () {
    cli-dungeon play
    builtin cd "$@"
}
```

## Basic rules
Every command has a `--help` flag which gives the up-to-date documentation.

Start by creating a character with `cli-dungeon create-character`.
You start with some gold. You can spend it with `cli-dungeon shop`.

Equip your purchase with `cli-dungeon character equip`.
See your character's current status with `cli-dungeon character status`.

You are now ready to meet your first enemy!

## Resting
You can rest your character with `cli-dungeon character rest long` if you need to silence `cli-dungeon` for a while.
You need to set your character to quest again with `cli-dungeon character quest`.

## Action script
`cli-dungeon` creates an `encounter.rhai` file in your configuration directory. Usually `.config/cli-dungeon` on linux and macos.

This [`rhai`](https://rhai.rs/book/) script is run to determine your character's action and bonus action in encounters.
You can list your characters available actions with `cli-dungeon character actions`.

A good start would be to implement drinking a potion of health if your health gets critical.

## Future plans
See the [issues page](https://github.com/SofusA/cli-dungeon/issues) for future and planned features.

## Inspiration
[`rpg-cli`](https://github.com/facundoolano/rpg-cli) has been a great inspiration for this project. `cli-dungeon` aims to be more interactive.
