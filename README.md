# Overview

This project is a restful backend for retrograde, an online agile retrospective tool.

# Running Locally

1. Start a postgres instance:

```sh
$ docker run --name postgres -e POSTGRES_PASSWORD=postgres -d -p 5432:5432 postgres:11
```

2. Create a "retrograde" database:

```sh
$ psql -h 127.0.0.1 -U postgres -c "create database retrograde;"
```

3. Start retrograde:

```sh
$ cargo run
```

## Docker Compose

Alternatively you can bring up an instance using docker-compose:

```sh
$ docker-compose up api postgres
```

After the docker image is built and started you can access the API on `localhost:8000`

# MVP:
  - make a board ✓
  - view board as owner ✓
  - add/edit/delete ranks ✓
  - public link to view board as participant ✓
  - add/edit/delete cards ✓
  - button to stop new cards ✓
  - button to open voting ✓
  - button to close voting ✓
  - vote on a card ✓
  - remove a vote ✓
  - sort by vote count ✓
  - single list result view sorted by vote count ✓

# Notes:
  - no accounts
  - no card comments
  - boards expire after some time of no use
  - all users get a cookie token (participant id)
  - max votes are based on the cookie participant id
  - board admin is based on the creators participant id cookie
  - google cloud run
  - google cloud (p)sql

# Future ideas:
  - reactions on cards (emoji)
  - save completed board as image/pdf
  - participant and voting stats per board