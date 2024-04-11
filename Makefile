ebpf:
	@RUST_LOG=info cargo xtask run

server:
	@cd ./http_server && cargo run

run:
	@cd ./runner && cargo run
