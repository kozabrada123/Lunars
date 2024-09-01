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

<br/>

## Functionality:

### Old, v1 ranking system

Lunars v1 was an adapted [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system), in which each player is assigned an elo value determining their skill level.

Depending on players' performances we then lowered or raised this elo value.

In our rating system players were assigned leagues based on which range of ranking points they were in. These leagues are as follows:

| From | To   | League   |
|------|------|----------|
| 2750 | 3000 | Champion |
| 2500 | 2750 | Master   |
| 2250 | 2500 | Pro      |
| 2000 | 2250 | Skilled  |
| 1750 | 2000 | Amateur  |
| 1500 | 1750 | Padawan  |
| 1000 | 1500 | Neophyte |

For a more in depth explanation of how the rating system worked, take a look at the [Lunaro Rating Specification](assets/lunaro-rating-specification.pdf), [written by quonnz](#credits)

### New, v2 ranking system

Lunars v2 is based on the [Glicko-2 ranking system](https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm), which adds rank deviation (how unsure we are of a player's true rating) and rank volatility (how inconsistent a player is).

Lunars v2 uses the same modification for player latency as Lunars v1.

Lunars v2 also utilises the fractional rating period modification seen in [instant-glicko-2](https://github.com/gpluscb/instant-glicko-2) (and Lichess' system). 

This system wouldn't have been possible without the following resources:
- [deepy/glicko2](https://github.com/deepy/glicko2) - helpful for writing the base glicko math in code 
- [gplusbc/instant-glicko-2](https://github.com/gpluscb/instant-glicko-2) and [So You Want To Use Glicko-2 For Your Game's Ratings](https://gist.github.com/gpluscb/302d6b71a8d0fe9f4350d45bc828f802) - fractional rating math, rating system theory

A heartfelt thanks to their authors!

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
  $ ./target/release/lunars       # Run binary
  $ docker-compose up -d --build  # Run in Docker
  ```

- Now you can interact with the API via [the defined endpoints](https://github.com/kozabrada123/Lunars/wiki/Endpoints).

## Credits

**[quonnz](https://github.com/imatpot)** - Document author, curator, developer

**koza1brada** - curator, developer

**Yujas** - architect, curator
