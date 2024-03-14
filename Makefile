all:
	@cargo build --release

install: all
	@cp target/release/ssb /usr/bin

clean:
	@cargo clean

uninstall:
	@rm /usr/bin/ssb

.PHONY: all clean install uninstall
