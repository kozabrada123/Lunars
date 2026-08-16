<p align="center">
  <img src="assets/images/lunaro_ranking.png" alt="project logo" height="100px"/>
</p>

<h1 align="center"> Lunars </h1>

![GitHub](https://img.shields.io/github/license/kozabrada123/Lunars?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kozabrada123/Lunars?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/kozabrada123/Lunars?style=flat-square)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/kozabrada123/Lunars?style=flat-square)
![GitHub issues](https://img.shields.io/github/issues/kozabrada123/Lunars?style=flat-square)

Lunars (Lunaro-scores) is a community-made ranking system for Lunaro, a PvP game mode in Digital Extremes' [Warframe](https://warframe.com).

This repo serves as the server (backend) for the rating system.

<br/>

## Functionality:

### v2 (glicko-based) ranking system

Lunars v2 is based on the [Glicko-2 ranking system](https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm), which adds rank deviation (how unsure we are of a player's true rating) and rank volatility (how inconsistent a player is).

Lunars v2 uses the same modification for player latency and score as Lunars v1.

Lunars v2 also utilises the fractional rating period modification seen in [instant-glicko-2](https://github.com/gpluscb/instant-glicko-2) (and Lichess' system).

Lunars v2 wouldn't have been possible without the following resources:
- [deepy/glicko2](https://github.com/deepy/glicko2) - helpful for writing the base glicko math in code
- [gplusbc/instant-glicko-2](https://github.com/gpluscb/instant-glicko-2) and [So You Want To Use Glicko-2 For Your Game's Ratings](https://gist.github.com/gpluscb/302d6b71a8d0fe9f4350d45bc828f802) - fractional rating period modification, rating system theory

A heartfelt thanks to their authors!

### Old, v1 ranking system

Lunars v1 was an adapted [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system), in which each player was assigned an elo value determining their skill level.

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

For a more in depth explanation of how the v1 rating system worked, take a look at the [Lunaro Rating Specification](assets/lunaro-rating-specification.pdf), [written by quonnz](#credits)

## Usage

- Download a release binary and .env file or git clone this repo.

- When cloning, you can generate a release binary for your local machine using
  ```sh
  $ cargo build --release
  ```
  You can then find the binary in `/target/release`.

- Before running the server, we need to create a json file that will have our hashed API keys.

- By default the server will look for a user.json, but this can be set in the .env

- Your json file should look something like this:
  ```json
  [
    {
      "hash" : "sha256hashoftheapikey"
    }
  ]
  ```

- You should also examine the values in the .env before running.

  **If you will be running behind a reverse proxy, such as Nginx, be sure to set the
  `X-Real-IP` header (on the proxy side) and set `USE_REVERSE_PROXY=true` in the .env:**

  .env
  ```
  # Include this is you will be running behind a reverse proxy;
  #
  # Configures the server to rely on X-Real-IP as the canonical
  # source of the requester's IP, to perform rate-limiting.
  #
  # Incorrectly configuring this can be detrimental to security
  #
  USE_REVERSE_PROXY="true"
  ```

  nginx.conf
  ```
	server {
		server_name lunars.example.com www.lunars.example.com;
		location / {
			proxy_set_header X-Real-IP $remote_addr;
			proxy_pass http://127.0.0.1:8081;
		}
	}
  ```

- Note the `proxy_pass` to the loopback interface: you should ensure the service
  is only reachable through the reverse proxy.

- After your setup is done, you can run the lunars executable, or run using Docker
  ```sh
  $ ./target/release/lunars       # Run binary
  $ docker-compose up -d --build  # Run with Docker compose
  ```

- Now you can interact with the API via [the defined endpoints](https://lunars.o7.si/swagger-ui/index.html).

## Credits

**[quonnz](https://github.com/imatpot)** - v1 specification author, curator, developer

**koza1brada** - curator, developer

**Yujas** - v1 architect, curator
