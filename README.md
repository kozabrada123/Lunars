<p align="center">
  <img src="assets/images/lunaro_ranking.png" alt="project logo" height="100px"/>
</p>

<h1 align="center"> Lunars </h1>

![AppVeyor](https://img.shields.io/appveyor/build/kozabrada123/Lunars?style=flat-square)
![GitHub](https://img.shields.io/github/license/kozabrada123/Lunars?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kozabrada123/Lunars?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/kozabrada123/Lunars?style=flat-square)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/kozabrada123/Lunars?style=flat-square)
![GitHub issues](https://img.shields.io/github/issues/kozabrada123/Lunars?style=flat-square)

Lunars (Lunaro-scores) is a community-made ranking system for Lunaro, a PvP game mode in Digital Extremes' [Warframe](https://warframe.com).

This repo serves as the server (backend) for the rating system.

## Functionality:

Lunars is an adapted [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system), in which each player is assigned an elo value determining their skill level.
Depending on players' performances we then lower or raise this elo value.

In our rating system players are assigned leagues based on which range of ranking points they are in. These leagues are as follows:

| From | To   | League   |
|------|------|----------|
| 2750 | 3000 | Champion |
| 2500 | 2750 | Master   |
| 2250 | 2500 | Pro      |
| 2000 | 2250 | Skilled  |
| 1750 | 2000 | Amateur  |
| 1500 | 1750 | Padawan  |
| 1000 | 1500 | Neophyte |

For a more in depth explanation of how the rating system works, take a look at the [Lunaro Rating Specification](assets/lunaro-rating-specification.pdf), [written by quonnz](#credits)

## Usage

- Download a release binary and .env file or git clone this repo.

- When cloning, you can generate a release binary for your local machine using
  ```sh
  $ cargo build --release
  ```
  You can then find the binary in `/target/release`.

- We now have the binaries, but before running the server we need to create a json file that will have our hashed keys.

- By default the server will look for a user.json but this can be set in the .env

- Your json file should look something like this:
  ```json
  [
    {
      "hash" : "yoursha256hashhere"
    }
  ]
  ```

- You should also set your database file path before running.

- After your setup is done, you can run the lunars executable, or run using Docker
  ```sh
  $ ./target/release/lunars       # Run bihttps://coolors.co/586f7c-22577a-38a3a5-cd4b13-c0212e-f71735-f3a712nary
  $ docker-compose up -d --build  # Run in Docker
  ```

- Now you can interact with the API via [the defined endpoints](https://github.com/kozabrada123/Lunars/wiki/Endpoints).

## Credits

**[quonnz](https://github.com/imatpot)** - Document author, curator, developer

**koza1brada** - curator, developer

**Yujas** - architect, curator
