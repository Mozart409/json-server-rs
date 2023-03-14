NAME=json-server-rs

clear:
	clear

dev: fmt clear
	cargo watch -x run

up: clear
	docker compose up -d --remove-orphans

fmt: clear
	cargo fmt

lint: clear
	cargo clippy --fix

release: clear
	cargo build --release

run: clear
	./target/release/$(NAME)

start: clear release run