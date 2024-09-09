set dotenv-load

export EDITOR := 'nvim'

alias f := fmt
alias r := run

default:
  just --list

fmt:
  cargo fmt

run:
  cargo run

test:
  cargo test
