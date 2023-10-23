# Monarch Launcher
![alt text](src-tauri/icons/Square310x310Logo.png)

## What is it?
Monarch launcher was created in response to the number of launchers needed to play PC games in 2023. Monarch is still in development, but has the goal of simplifying the gaming experience by removing the need to use so many launchers. With Monarch you no longer need to remember what platform you have each game, instead Monarch keeps track of all of them in one place.

As stated above Monarch is still in development (not even an alpha), therefor features mentioned below are subject to change. But the core goal of Monarch will still be to become the only launcher ever needed. We are planning on not only managing games already installed on your system, but also help you download new ones through Monarch.

## Features:
These are some of the features we want to include in Monarch and their development status. Not all are going to be developed in the order below and not all will be in the first official release of Monarch, but the core features planned in the first release are **game management**, **quicklaunch**, **game collections** and some **quality of life** features. 

| Feature                  | Status |
| -------                  | ------ |
| Find games automatically | 🟡 Developing |
| Launch games             | 🟡 Developing |
| Manage Steam/Epic games  | 🟡 Developing |
| Game collections/folders | 🟢 Done       |
| Quick launch             | 🟡 Developing |
| Launch arguments         | 🔴 Planned    |
| Download scripting       | 🔴 Planned    |
| UI overhaul              | 🔴 Planned    |
| Better stats/descriptions| 🔴 Planned    |
| Friends                  | 🔴 Planned    |
| Plugin system            | 🔴 Planned    |

## Other benefits

### Blazingly fast!
There's a meme that anything written in [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) is "blazing fast". However in Monarch's case there's a grain of truth to it. Thanks to the use of [Tauri](https://tauri.app/) and [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) Monarch uses a lot less RAM and CPU than the competitors. It also doesn't slow you down by forcing you to wait while Monarch looks for updates before actually starting. 

### Open-source
We believe in transparency, which is why Monarch is open-sourced. This allows anyone to take a peak underneath the hood to understand and possibly improve Monarch. We believe that this approach both builds better trust but also improves the development of Monarch.

### Quicklaunch
One of our goals with Monarch is to make the gaming experience as easy as possible and focused on the important parts, which is gaming. Therefor we implemented a feature we nicknamed Quicklaunch. Quicklaunch allows you to use a keyboard shortcut to open a small search window, the quicklaunch window, where you can type the name of the game you want to play and launch by simply highlighting it and pressing Enter. If you allow Monarch to run at start-up (given it's low resource usage) you could have your favourite game up and running in only a few seconds.

## How do I get it?
There are two ways of getting Monarch:
1. Download the latest version from [releases](https://github.com/an0nymoos3/Monarch/releases) (Recommended for most).

2. Compile it yourself. It's as easy as 1,2,3:

    1) Download the source code via the green **code** button.
    2) Make sure you have both [Rust/Cargo](https://www.rust-lang.org/) and [Yarn](https://yarnpkg.com/) installed.
    3) Open a terminal in the project folder and run: `yarn` followed by `yarn build`.

Then you run the installer that you either downloaded or built youself. If you built it yourself you'll find it in the project folder under `src-tauri/target/release/bundle/`.

## Who are we?
We are two university students from Sweden and Belguim. We started this project in our spare time as we are studying full-time. We are also new to the FOSS comunity, so if there is anything you think looks weird about the way Monarch is setup or managed, please reach out and help us improve.

Due to the time limit of being full-time students we are looking to expand the team. Anyone interested in helping out on the project is welcome to reach out. 

## How can I contribute?
Any help is appreciated. If you know programming and know of a feature which is missing or needs improvement, feel free to contribute. We currently don't have an official way of reporting issues, we'll update this page as soon as we do. 

Would you like to become a regular contributer/maintainer? Dm me on discord @an0nymoos3 to let me know about why you would like to help aswell as a brief description of your relevant experience.