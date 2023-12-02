**This project is not accepting contributions!** Feel free to fork and modify to your heart's content, but I will not be accepting contributions back into this repo.

An ARPG, being worked on in free time and with no planning whatsoever.

## Game Data
Game data lives in the projects that contain there implementation. For example, cypher-item contains item.json.

This data is copied to cypher-game as a build script. These copied files are in the .gitignore

## Building Server Docker Image
`docker build -f docker/Server.Dockerfile -t cypher-server .`

## Special Thanks
- [KenneyNL](https://www.kenney.nl/), for game assets distributed under CC0.
