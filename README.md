# Pookie: Homecoming!! 🚀

A 2D space shooter built solo for the "Do You Wanna Jam?! 2026" game jam (theme: **Unstable**), made in **Rust** using the **Bevy** game engine.

**[Play it on itch.io →](https://myselfaditya23.itch.io/pookie-homecoming)**

## About

Your ship gets hit by an asteroid and starts limping home, badly damaged. As your **Ship Stability** drops during combat, your controls get progressively worse — misfiring lasers, inaccurate aim, and at critical stability, fully inverted movement controls. Fight through 3 escalating waves, follow your Pookie AI's logged checkpoints, and make it home to Pookietopia. An Endless mode unlocks after winning, for chasing a high score.

## Built in 9 days, from zero

This was my **first Rust project ever** and my **first finished game ever**. I had no prior Rust or Bevy experience going in — I built this while learning both simultaneously, under the jam's deadline. I used AI assistance heavily throughout the build (debugging, structuring systems, working through Bevy's API), since I was learning as I went rather than coding fluently from prior knowledge. I'm being upfront about that because I think the "in-progress, honestly-built" story is more useful to share than pretending otherwise.

## Features

- Mouse-aimed combat with independent WASD movement
- A damage-tiered instability system tied directly to the game's theme
- 3-wave campaign + Endless mode with a scoring/accuracy grade system
- A full game state machine (intro, story, gameplay, pause, win/lose, score screens)
- Procedurally synthesized sound effects, licensed background music

## Tech stack

- **Rust**
- **Bevy** (game engine, ECS architecture)
- `rand` crate for randomization (crit chance, spawn positions, etc.)
- `winresource` for the Windows executable icon

## Project structure

src/
├── main.rs — App setup, plugin registration
├── state.rs — Game state enum, volume settings
├── ship.rs — Player ship, movement, Stability resource
├── laser.rs — Player weapon logic
├── enemy.rs — Enemy AI, waves, collisions, explosions
├── camera.rs — Camera follow logic
├── star.rs — Procedural infinite starfield
├── crosshair.rs — Mouse aim crosshair
├── ui.rs — Compass, stability bar, checkpoints, HUD
└── screens.rs — Intro/story/pause/win/game-over/score screens

## Running locally

```bash
cargo run --release
```

## Credits

Music: "Mesmerizing Galaxy" by Kevin MacLeod (incompetech.com), licensed under [CC BY 4.0](http://creativecommons.org/licenses/by/4.0/)

## What's next

I'm hoping to keep developing this into a fuller roguelike/roguelite with items and build variety — this jam version is the foundation, not the final stop.
