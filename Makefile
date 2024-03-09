all:
	@cargo build --release

install: all
	@cp target/release/ssb /usr/bin

clean:
	@cargo clean


.PHONY: all clean install
