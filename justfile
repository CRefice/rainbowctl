deploy:
	cargo build --release --target=arm-unknown-linux-gnueabihf
	scp target/arm-unknown-linux-gnueabihf/release/rainbowctl pi@192.168.1.130:~

run-tx:
	cargo build --release --bin=equalizer-tx
	cava -p cava.conf &
	./target/release/equalizer-tx
