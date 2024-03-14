all:
	@cargo build --release

install: all
	@cp target/release/ssb /usr/bin

uninstall:
	@rm /usr/bin/ssb

.PHONY: all install uninstall
