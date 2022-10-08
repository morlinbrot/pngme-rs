default:
	just -l -u

build:
	cargo build

run:
	cargo run

test: 
	cargo watch -x "test" -c
alias t := test

nextest:
	cargo watch -x "nextest run" -c
alias nt := nextest
