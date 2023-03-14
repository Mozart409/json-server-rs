clear:
	clear

dev: fmt lint clear
	cargo watch -x run

up: clear
	docker compose up -d --remove-orphans

fmt: clear
	cargo fmt

lint: clear
	cargo clippy --fix --allow-dirty
