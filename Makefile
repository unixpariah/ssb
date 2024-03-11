all:
	@cargo build --release

install: all
	@cp target/release/ssb /usr/bin

clean:
	@cargo clean

uninstall:
	@rm /usr/bin/ssb

nix:
	@nix build -f Cargo.nix rootCrate.build

.PHONY: all clean install uninstall nix
