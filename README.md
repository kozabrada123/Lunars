
# Lunars (WIP)
<div>


![AppVeyor](https://img.shields.io/appveyor/build/kozabrada123/Lunars?style=flat-square)
![GitHub](https://img.shields.io/github/license/kozabrada123/Lunars?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kozabrada123/Lunars?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/kozabrada123/Lunars?style=flat-square)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/kozabrada123/Lunars?style=flat-square)
![GitHub issues](https://img.shields.io/github/issues/kozabrada123/Lunars?style=flat-square)

</div>

Lunars (Lunaro-scores) is a community-made ranking system for Lunaro, a minigame in Digital Extremes' Warframe



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

Once the api is running, you can make json POST requests to /get/player/, /get/match, /add/ and /submit/ respectively.

/get/player/ is used to get players from the database, a request will look like `{"qtype":"id","value":"43"}` or `{"qtype":"name","value":"PlayerName"}`

/get/match/ is the same, expect you can only get matches by their id's

/add/ adds a player, a request will look like `{"token":"supersecrettoken","name":"PlayerName","elo":500}`

/submit/ adds a match, a request will look like 
`{"token":"supersecrettoken","user_a":"PlayerName","ping_a":0,"score_a":5,"user_b":"Playername2","ping_b":20,"score_b":6}`

## Credits:

**[quonnz](https://github.com/imatpot)** - Document author, curator, developer
<br/>
**koza1brada** - curator, developer
<br/>
**Yujas** - architect, curator
