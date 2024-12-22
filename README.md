# AmandaDB
AmandaDB is a simple Redis clone built with Rust standart library, compatible with Redis CLI.

## Getting Started

Cloning the git repository:
```
git clone https://github.com/Pedrinjs/AmandaDB
```

Running the application:
```
cargo run
```

To send a request, you can do the following:
```
# 6379 is the default port for Redis
printf "[your command]" | netcat localhost 6379

# or type the command with redis-cli using its command
redis-cli
```

## Features
- Basic redis commands
- Full RESP2 support
- `redis-cli` support
- Multithreading
- Transactions

## Docs
Although this current project isn't fully compatible, you can read the official docs for Redis in <https://redis.io/docs/latest/>
