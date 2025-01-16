# AmandaDB
AmandaDB is a simple Redis clone built with Rust standart library, compatible with Redis CLI.

## Features
- Basic redis commands
- RESP3 support (expect for Big numbers and Verbatim strings)
- `redis-cli` support
- Multithreading
- Transactions

## Installation
> [!IMPORTANT]
> For now, it's only possible to install it on Arch Linux.

### Arch
You can install AmandaDB by cloning the PKGBUILD and building with makepkg:

Make sure you have the `base-devel` package group installed.
```
$ sudo pacman -S --needed git base-devel
$ git clone https://aur.archlinux.org/amandadb.git
$ cd amandadb
$ makepkg -si
```

## Usage
To run AmandaDB with the default configuration, simply type:
```
$ amandadb
```

It will start running at localhost:6379.
If you have a config file (.conf) with `key=value` syntax, you have to run it using an additional parameter:
```
$ amandadb /path/to/file.conf
```
The config file can provide port, database name (called as "aof") and number of threads.
The database name should not contain single or double quotes

As a Redis clone, you can play with it directly with the "redis-cli" command.
The database created persists at an append-only file (AOF).

## Docs
Although this current project isn't fully compatible, you can read the official docs for Redis in <https://redis.io/docs/latest/>
