##
# This file is part of the IVMS Online.
#
# @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
##

SHELL:=bash

default: build

clean:
	cargo clean

build:
	cargo build --release --target x86_64-unknown-linux-musl

build-dev:
	cargo build --target x86_64-unknown-linux-musl

package: $(shell find target/x86_64-unknown-linux-musl/release/ -maxdepth 1 -executable -type f | sed s@x86_64-unknown-linux-musl/release/\\\(.*\\\)\${$}@\\1.zip@)

test:
	cargo tarpaulin --all-features --out Xml --bins

test-local:
	docker run -d --rm --name dynamodb -p 8000:8000 amazon/dynamodb-local:2.0.0
	make test
	docker stop dynamodb

test-integration:
	cargo test --test "*"

check:
	cargo fmt --check -- --config max_width=120,newline_style=Unix,edition=2021
	cargo clippy
	cargo udeps

check-local:
	cargo audit

doc:
	cargo doc --no-deps

# generic targets
target/%.zip: target/x86_64-unknown-linux-musl/release/%
	upx --best $<
	zip -j $@ $^
	printf "@ $(<F)\n@=bootstrap\n" | zipnote -w $@

full: clean build-dev test-local check check-local doc
