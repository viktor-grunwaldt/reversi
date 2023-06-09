# reversi
Reversi cli engine written for my AI class using a minimax variant.

## Installing

- Make sure you have rustup installed [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- Clone this repository
- `cd reversi`
- `cargo build --release`(first compilation will take a while)

## How to use

This engine is meant to be used with the `ai_dueler.py` I got from my ai course. If you already have it, great. 
In order to have an AI duel, you will have to create a shell script first:
- make sure you have the git repository in your current folder
- create `tournament.sh` file
- paste 
```sh
#!/usr/bin/env sh

./reversi/target/release/reversi tournament-mode
```
- run `chmod +x tournament.sh` to make it executable
- run `python ai_dueler.py --num-games {x} reversi tournament.sh {your_bot}`
