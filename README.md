# Overview

This project is a restful backend for retrograde, an online agile retrospective tool.

# Running Locally

1. Start a postgres instance:

```sh
$ docker run --name postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=retrograde -d -p 5432:5432 postgres:12
```

2. Start retrograde:

```sh
$ cargo run
```

## Using Docker Compose

Alternatively you can bring up an instance using docker-compose:

```sh
$ docker-compose up api postgres
```

After the docker image is built and started you can access the API on `localhost:8000`

# Creating a New Database Migration

Ensure you have diesel installed and run the following:

```sh
$ diesel migration generate <migration-name>
```

Add you database changes to the newly created up.sql and down.sql files in the migrations directory.
