PKGNAME := amandadb
PKGDIR :=
PREFIX := /usr/bin

clean:
	@rm -rf target/*
	@cargo clean

run: cargo run

build: @cargo build --release

install:
	install -Dm755 target/release/$(PKGNAME) $(PKGDIR)$(PREFIX)/$(PKGNAME)

