PKGNAME := amandadb
PKGDIR :=
PREFIX := /usr/bin
CONFIG := 

clean:
	@rm -rf target/*
	@cargo clean

run:
	cargo run $(CONFIG)

build: @cargo build --release

install:
	install -Dm755 target/release/$(PKGNAME) $(PKGDIR)$(PREFIX)/$(PKGNAME)

