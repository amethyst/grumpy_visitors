# Grumpy Visitors

[![GitHub pull requests](https://img.shields.io/github/issues/mvlabat/grumpy_visitors/good%20first%20issue?label=good%20first%20issues&color=7057ff)](https://github.com/mvlabat/grumpy_visitors/issues)

Grumpy Visitors is a top-down 2D action arcade game.
It's written in [Rust](https://www.rust-lang.org/) and running on [Amethyst](amethyst.rs) game engine.

This project is highly inspired by Evil Invasion game. Unfortunately its official site is no longer available,
as it's quite old and not really popular, but here is some recorded demo gameplay to get the idea:
https://youtu.be/bWpWJzb9JdE.

This is my first gamedev project (not the first attempt to make one though). My ultimate goal is to make
a finished product (and maybe start selling this game if I ever accomplish it, who knows).
At the same time I want to keep this project open-source and available for everyone to build it on their local machine
and play for free.

![Grumpy Visitors screenshot](header_screenshot.png)

**Planned gameplay features:**
- Campaign and survival modes 
- Character development system with persistent progress between levels
- Co-op multiplayer (up to 4? players)

These are more like high-level goals for me, as I don't have a clearly formulated vision or design for this game.
Everything's still just in my head and therefore almost anything is a subject to change.

## Building
```bash
cargo build -p gv_server # if you want to host a server for multiplayer
cargo build -p gb_client
```

## Fetching Game Assets

Grumpy Visitors relies on `git-lfs` to fetch game assets (images, etc.) See [git-lfs](https://github.com/git-lfs/git-lfs) for installation instructions if you don't already have it on your system. Then:

```bash
git lfs install
git lfs fetch
git lfs pull
```

You can use `git lfs ls-files` to see that the assets have been fetched.

**Please note** that specifying just a binary without a package (`cargo build --bin gv_server`) won't work.
Cargo tries to merge all the features of common dependencies among all the members of workspace,
which leads the build process to fail.
**[Feature selection in workspace depends on the set of packages compiled (cargo#4463)](https://github.com/rust-lang/cargo/issues/4463)**

Supported platforms:
- Windows 10 (Vulkan)
- Linux (Vulkan)
  - though I myself failed to build it on my system because of
  [(shaderc-rs#61)](https://github.com/google/shaderc-rs/issues/61)
- MacOS (Metal)
  - you'll need to comment out Vulkan support and uncomment Metal in `bins/client/Cargo.toml`
  [(cargo#1197)](https://github.com/rust-lang/cargo/issues/1197) 

## Current state
This project is in its early stage of development. There are only some very basic features implemented:
- Multiplayer
- Casting a spell (homing ~~missile~~ red dot)
- Spawning monsters
- Monster AI (actually just randomly walking around the map and starting to chase a player if they're close enough)
- Character moving
- Sprite animations
- Custom shaders (health HUD)
- Menu states and transitions

### Roadmap to 0.2
- [x] Rewrite networking with the upcoming version of `amethyst_net`
- [ ] Try to rework `ActionSystem` (current approach to run systems several times in 1 frame sucks)
- [x] Add profiling
- [ ] Implement possibility to pause/unpause writing profiler traces
- [ ] Look for weak spots in performance and optimize them if found and possible
- [ ] Add visual indicators for better debugging (mobs health, network state, fps, latency etc)
- [ ] Refactor UI code (current definition files and the system are huge)
- [ ] Better visuals (polishing animations, adding some nice shaders for spells)
- [ ] Implement defining spells and monsters with *.ron files (no more hardcoded values)
- [ ] Unit tests

### Known issues
- Starting a multiplayer game before the connected peers pop up in the players list will cause a crash
- Starting a multiplayer game with two or more players will start a paused (with no way to unpause) game
([amethyst#2024](https://github.com/amethyst/amethyst/issues/2024))
- The multiplayer game will eventually crash because of `ExceededMaxPacketSize` error
- You have only one chance to join or host a game. Anything went wrong? Just restart it.
(A lot of menu transitions polishing still needs to be done.)
- Leaving a multiplayer room will cause redundant players to spawn in single player
- In multiplayer missiles are a little bit clunky when launching and sometimes desync

## License
The code is shared under the [MIT license](LICENSE).

All the assets are shared under the **CC BY-NC 4.0** license
(see [assets/LICENSE](assets/LICENSE) and [resources/assets/LICENSE](resources/assets/LICENSE)). 

## Contributing
Every contribution is really welcome! Please feel free to submit pull requests and create your own issues.
I'll also try to be open to new ideas as much as I can, though I can't make a solid promise about that...
Author's vision, you know. :)

This project's scope seems to be small, but the codebase may scare you. I can't say I've made architecture
decisions to be referentially good, and it's already easy to get lost in the code.

But nevertheless don't hesitate to take a look at "good first issues", so you can get a grasp of it and bring
something good to this game: 

[![GitHub pull requests](https://img.shields.io/github/issues/mvlabat/grumpy_visitors/good%20first%20issue?label=good%20first%20issues&color=7057ff)](https://github.com/mvlabat/grumpy_visitors/issues)

## Credits
Special thanks to
- [Klaudia Jankowska](https://klaudiajankowskaart.myportfolio.com/) for the awesome assets
- The great Amethyst community for being extremely welcoming and helpful
- [Erlend](https://github.com/erlend-sh) for noticing this game, giving me motivation and useful advice
