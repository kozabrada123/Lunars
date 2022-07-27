
<p align="center">
 <img style="display: block; margin-left: auto; margin-right: auto; width:30%;" src="https://raw.githubusercontent.com/kozabrada123/PyLunaroRPC/main/assets/images/Lunaro-logo.png" alt="project logo" width="30%"/>
 </p>

<h2 align="center"> Lunars </h2>

<div>


![AppVeyor](https://img.shields.io/appveyor/build/kozabrada123/Lunars?style=flat-square)
![GitHub](https://img.shields.io/github/license/kozabrada123/Lunars?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kozabrada123/Lunars?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/kozabrada123/Lunars?style=flat-square)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/kozabrada123/Lunars?style=flat-square)
![GitHub issues](https://img.shields.io/github/issues/kozabrada123/Lunars?style=flat-square)

</div>

Lunars (Lunaro-scores) is a community-made ranking system for Lunaro, a minigame in Digital Extremes' Warframe

This repo serves as the server (backend) for the rating system.



## Functionality: 

Lunars is an adapted [elo system](https://en.wikipedia.org/wiki/Elo_rating_system), in which each player is assigned an elo value determining their skill level.
Depending on players' performances we then lower or raise this elo value.

<br/>

In our rating system players are assigned ranks / titles based on which elo group they are in.
<br/>
These ranks / titles are:
<br/>

2750   +          Champion Candidate
<br/>
2500   –  2750    Master
<br/>
2250   –  2500    Pro
<br/>
2000   –  2250    Skilled
<br/>
1750   –  2000    Amateur
<br/>
1500   –  1750    Padawan
<br/>
1000   –  1500    Neophyte
<br/>


<br/>

For a more in depth explanation of how the rating system works, take a look at the [Lunaro Rating Specification](https://github.com/kozabrada123/Lunars/blob/main/resources/lunaro-rating-specification.pdf), [written by quonnz](#credits)

## Usage:

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

You can also your database file path before running.

After your setup is done, you can run the rlunaroratings executable.

Once the server is running, you can make json requests to several different endpoints:

```
GET /api/players
--> returns all players, TODO: filters like ?order_by=name|elo|id|, ?sort=asc|desc
ex: [
{"id":1,"name":"aname","elo":1000},
..
]

GET /api/players/:qtype
--> returns the player with the given name or id :qtype (e.g. /api/players/yujas). Names are non case sensitive.
ex: {"id":1,"name":"aname","elo":1000}

GET /api/matches
--> returns all matches. TODO: filters like ?before=, ?after= (linux timestamps), ?has_player=<qtype>
ex:[
{"id":1,"player_1":1,"player_2":2,"player_1_score":5,"player_2_score":6,"player_1_elo_change":2,"player_2_elo_change":-2,"epoch":1658084340936},
..
]

GET /api/matches/:id
--> returns the match with the given id.
ex: {"id":1,"player_1":1,"player_2":2,"player_1_score":5,"player_2_score":6,"player_1_elo_change":2,"player_2_elo_change":-2,"epoch":1658084340936}

POST /api/players/add
--> adds a player to the database. names are non case sensitive.
ex: {"token":"yoursecrettoken", "name":"yourname", "elo":1000}

POST /api/matches/add
--> adds a match to the database.
ex: {"token":"yoursecrettoken","user_a":"user_a_name","user_b":"user_b_name","score_a":5,"score_b":6,"ping_a":0,"ping_b":10}
```

## Credits:

**[quonnz](https://github.com/imatpot)** - Document author, curator, developer
<br/>
**koza1brada** - curator, developer
<br/>
**Yujas** - architect, curator
