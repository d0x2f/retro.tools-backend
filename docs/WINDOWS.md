# Overview

This guide is to configure prerequisites and run retrograde.rs within the windows subsystem for linux (WSL).

# Prerequisites

1. Install rustup:

```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Add cargo binaries to your $PATH:

```sh
$ source $HOME/.cargo/env
```

3. Install build-essential and postgres libs

```sh
$ sudo apt update
$ sudo apt install build-essential libpq-dev
```

4. Select the nightly rust toolchain

```sh
$ rustup default nightly
```

5. Build retrograde and install project dependencies:

```sh
$ cargo build
```

6. Install docker:

Install docker for windows from here: https://hub.docker.com/editions/community/docker-ce-desktop-windows

Then install docker client in WSL like so:

```sh
# Install Docker's package dependencies.
sudo apt install apt-transport-https ca-certificates curl software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo apt-key fingerprint 0EBFCD88
sudo add-apt-repository \
   "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
   $(lsb_release -cs) \
   stable"
sudo apt update
sudo apt install docker-ce
sudo usermod -aG docker $USER
newgrp docker
echo "export DOCKER_HOST=tcp://localhost:2375" >> ~/.bashrc && source ~/.bashrc
```

# Running

1. Start a postgres instance:

```sh
$ docker run --name postgres -e POSTGRES_PASSWORD=postgres -d -p 5432:5432 postgres:11
```

2. Create a "retrograde" database:

```sh
$ psql -h 127.0.0.1 -U postgres -c "create database retrograde;"
```

3. Run the database migrations using diesel:

```sh
$ diesel migration run
```

4. Start retrograde:

```sh
cargo run
```

The API should now be available at http://localhost:8000
