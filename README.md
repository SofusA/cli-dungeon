# cli-dungeon
Inspired by dice-based table-top RPGs, `cli-dungeon` brings dungeon crawling to your terminal with a twist of automation and scripting.

```
It is Sofusa's turn!
Bandit took 5 damage
Bandit took 3 damage
It is Bandit's turn!
Bandit missed
Bandit missed
It is Sofusa's turn!
Sofusa missed
Bandit took 1 damage
It is Bandit's turn!
Sofusa took 2 damage
Bandit missed
It is Sofusa's turn!
Sofusa missed
Bandit took 4 damage
Bandit died  
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
```bash
alias cd="cli-dungeon play; cd"
```

### Example in `fish`
This is my personal fish prompt.
`cli-dungeon play` on every prompt in the terminal.

```fish
function fish_prompt
    cli-dungeon play
    set -l last_status $status
    set -l stat
    if test $last_status -ne 0
        set stat (set_color red)"[$last_status]"(set_color normal)
    end

    string join '' -- (set_color blue) (prompt_pwd) (set_color normal) $stat '> '
end

```

## Basic rules
Every command has a `--help` flag which gives the up-to-date documentation.

Start by creating a character with `cli-dungeon create-character`.
You start with some gold. You can spend it with `cli-dungeon shop`.

Equip your purchase with `cli-dungeon character equip`.
See your character's current status with `cli-dungeon character status`.

You are now ready to meet your first enemy!

## Action script
`cli-dungeon` creates an `encounter.rhai` file in your configuration directory. Usually `.config/cli-dungeon` on linux and macos.

This [`rhai`](https://rhai.rs/book/) script is run to determine your character's action and bonus action in encounters.
You can list your characters available actions with `cli-dungeon character actions`.

A good start would be to implement drinking a potion of health if your health gets critical.

## Future plans
See the [issues page](https://github.com/SofusA/cli-dungeon/issues) for future and planned features.

## Inspiration
[`rpg-cli`](https://github.com/facundoolano/rpg-cli) has been a great inspiration for this project. `cli-dungeon` aims to be more interactive.
