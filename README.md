# Monarch Launcher
![alt text](src-tauri/icons/Square310x310Logo.png)

## What is it?
Monarch Launcher was created in response to the number of launchers needed to play PC games in 2023. Monarch is still in development, but it has the goal of simplifying the gaming experience by removing the need to use so many launchers. With Monarch, you no longer need to remember what platform you have for each game; instead, Monarch keeps track of all of them in one place.

As stated above, Monarch is still in development (not even an alpha); therefore, the features mentioned below are subject to change. But the core goal of Monarch will still be to become the only launcher you need. We are planning on not only managing games already installed on your system but also helping you download new ones through Monarch.

## Current state
I (an0nymoos3) feel the need to preface that Monarch is currently being developed. This means that if you are simply here to try Monarch out, you are going to be disappointed. While we will be working as fast as possible to get Monarch ready, we have other obligations, such as studying or working. Therefore, you are welcome to join the Discord server in the meantime to follow our progress. If you still want to give Monarch a spin, you can do so by following the instructions on how to compile it under **How do I get it?**.

## Features
These are some of the features we want to include in Monarch and their development status. Not all are going to be developed in the order below, and not all will be in the first official release of Monarch, but the core features planned for the first release are **game management**, **quicklaunch**, **game collections**, and some **quality of life** features.

| Feature                  | Status |
| -------                  | ------ |
| Find games automatically | 🟡 Steam only |
| Launch games             | 🟡 Steam only |
| Manage Steam games       | 🟡 Developing |
| Manage Epic games        | 🔴 Planned    |
| Game collections/folders | 🟢 Done       |
| Quicklaunch              | 🟡 Developing |
| Find/buy new games       | 🟡 Developing |
| Launch arguments         | 🔴 Planned    |
| Download scripting       | 🔴 Planned    |
| UI overhaul              | 🔴 Planned    |
| User stats               | 🔴 Planned    |
| Friends                  | 🔴 Planned    |
| Plugin system            | 🔴 Planned    |

## Other benefits

### Blazingly fast!
There's a meme that anything written in [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) is "blazing fast." However, in Monarch's case, there's a grain of truth to it. Thanks to the use of [Tauri](https://tauri.app/) and [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) Monarch uses less RAM and CPU than its competitors. It also doesn't slow you down by forcing you to wait while Monarch looks for updates before actually starting.

### Open-source
We believe in transparency, which is why Monarch is open source. This allows anyone to take a peek underneath the hood to understand and possibly improve Monarch. We believe that this approach not only builds better trust but also improves the development of Monarch.

### Quicklaunch
One of our goals with Monarch is to make the gaming experience as easy as possible and focus on the important parts: gaming. We therefore implemented a feature we call Quicklaunch. Quicklaunch allows you to use a keyboard shortcut to open a small search window, the quicklaunch window, where you can type the name of the game you want to play and launch by simply highlighting it and pressing Enter. If you allow Monarch to run at start-up (given its low resource usage), you could have your favorite game up and running in only a few seconds.

### Cross-platform
Tauri allows us to build Monarch cross-platform, meaning that you can download Monarch on Windows, MacOS, and Linux. Given that the majority of gamers use Windows, they will likely have the latest and greatest features first, as it makes the most sense to build for Windows first. However, we still aim to fully support MacOS and Linux.

## How do I get it?
There are two ways of getting Monarch:
1. Download the latest version from [releases](https://github.com/Monarch-Launcher/Monarch/releases) (Recommended for most).

2. Compile it yourself. It's as easy as 1,2,3:
    1) Download the source code via the green **code** button.
    2) Make sure you have both [Rust/Cargo](https://www.rust-lang.org/) and [Yarn](https://yarnpkg.com/) installed.
    3) Open a terminal in the project folder and run: `yarn` followed by `yarn build`.

Then you run the installer that you either downloaded or built youself. If you built it yourself you'll find it in the project folder under `src-tauri/target/release/bundle/`.

## Who are we?
We started out as two university students doing this in our free time. Due to time constraints and growing complexity, we are currently expanding the team working on Monarch. If you would like to be part of this journey, feel free to reach out to an0nymoos3 on Discord.  

Feel free to join our [Discord server](https://discord.gg/YjvJPPdbCw).

## How can I contribute?
Any help is appreciated. If you know programming and know of a feature that is missing or needs improvement, feel free to start working on it. We currently don't have an official way of reporting issues; we'll update this page as soon as we do. 

## License
We don't know much about copyright law or law in general, so we'll keep this section short. We have decied to license Monarch under the GNU General Public License v3.0 (GLPv3). We won't explain exactly what that means ourselves, as we don't want to accidently mislead anyone. Instead you can either read about it in [COPYING](https://github.com/Monarch-Launcher/Monarch/blob/main/COPYING), or on these online resources.
[chooselicense.com](https://choosealicense.com/licenses/gpl-3.0/), [wikipedia](https://en.wikipedia.org/wiki/GNU_General_Public_License)
