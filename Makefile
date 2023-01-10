
c:
	cargo clippy
ca:
	cargo clippy --all-targets -- -D warnings -A deprecated	
r:
	cargo build --release
help:
	target/release/archeologit --help
e:
	target/release/archeologit --path /path/RedisNumbersStats
