# **Discordia**
<a href="https://www.artstation.com/666kart">
    <p align="center">
    <img src="/data/icon/avatar.png"/>
    </p>
</a>

This is a simple Discord Bot created in [Rust](https://www.rust-lang.org/) using [serenity-rs](https://github.com/serenity-rs/serenity) to be used in conjunction with a [Redis](https://redis.io/) server.

## Features
* Play music and manage playlist directly from *youtube*
* *Leveling* and *Economy* systems based on RPG adventures
* Responses based on the affinity of the bot towards users. 
  Using other bots will reduce affinity and can possibly make the bot jealous making it ignore your comands

## Download
Just clone the repository content on your machine or server.
```
git clone https://github.com/z1ko/discordia
```


## Running
We will use [Docker](https://www.docker.com/) to run the database and bot containers, the repository already contains a *docker-compose.yml* to allow for fast deployment on the user machine.

```
docker-compose build
docker-compose up -d
```