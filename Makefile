.PHONY: clean tex-rust

SRC_FORMAT=TeXinputs/plain.tex

FORMAT_FILE=$(basename $(notdir $(SRC_FORMAT))).fmt

tex-rust:
	printf '\\dump' | cargo run -- -ini $(SRC_FORMAT)
	cargo build --release

clean:
	cargo clean
	rm -f $(FORMAT_FILE)
