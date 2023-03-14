clear:
	clear

dev: fmt clear
	cargo watch -x run

up: clear
	docker compose up -d --remove-orphans

fmt: clear
	cargo fmt

lint: clear
	cargo clippy --fix -Dclippy::perf

perf: clear
	cargo flamegraph 