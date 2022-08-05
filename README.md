<p align="center">
  <img style="display: block; margin-left: auto; margin-right: auto; width:30%;" src="https://raw.githubusercontent.com/kozabrada123/PyLunaroRPC/main/assets/images/Lunaro-logo.png" alt="project logo" width="30%"/>
</p>

<h2 align="center"> Lunars </h2>

![AppVeyor](https://img.shields.io/appveyor/build/kozabrada123/Lunars?style=flat-square)
![GitHub](https://img.shields.io/github/license/kozabrada123/Lunars?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kozabrada123/Lunars?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/kozabrada123/Lunars?style=flat-square)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/kozabrada123/Lunars?style=flat-square)
![GitHub issues](https://img.shields.io/github/issues/kozabrada123/Lunars?style=flat-square)

Lunars (Lunaro-scores) is a community-made ranking system for Lunaro, a minigame in Digital Extremes' Warframe

This repo serves as the server (backend) for the rating system.

## Functionality:

Lunars is an adapted [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system), in which each player is assigned an elo value determining their skill level.
Depending on players' performances we then lower or raise this elo value.

In our rating system players are assigned leagues based on which elo group they are in. These leagues are as follows:

| From | To   | League |
|------|------|--------|
| 2750 | 3000 | League |
| 2500 | 2750 | League |
| 2250 | 2500 | League |
| 2000 | 2250 | League |
| 1750 | 2000 | League |
| 1500 | 1750 | League |
| 1000 | 1500 | League |

For a more in depth explanation of how the rating system works, take a look at the [Lunaro Rating Specification](https://github.com/kozabrada123/Lunars/blob/main/resources/lunaro-rating-specification.pdf), [written by quonnz](#credits)

## Usage

Firstly, download a release binary and .env file or git clone this repo.

If cloning you can generate a release binary for your local machine using `cargo build --release` (You can then find the binary in /target/release/).

We now have the binaries, but before running the server we need to create a json file that will have our hashed keys.

By default the server will look for a user.json but this can be set in the .env

Your json file should look something like this:

```
[
    {
        "hash" : "yoursha256hashhere"
    }
]
```

You can also set your database file path before running.

After your setup is done, you can run the lunars executable. (Can be run in docker via `docker-compose up -d (--build)` )

Now you can interact with the api via [different endpoints](https://github.com/kozabrada123/Lunars/wiki/Endpoints).

## Credits

**[quonnz](https://github.com/imatpot)** - Document author, curator, developer

**koza1brada** - curator, developer

**Yujas** - architect, curator
