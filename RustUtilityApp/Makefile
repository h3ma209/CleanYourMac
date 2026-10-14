# Sable. Run `make help` for the list.
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

CARGO ?= cargo
PREFIX ?= /Applications
APP := dist/Sable.app
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' crates/ui/Cargo.toml | head -1)

.DEFAULT_GOAL := all

.PHONY: all help build release run run-release test check package install open clean

all: release

help:
	@printf '%s\n' \
		'make                 release build of the ui' \
		'make build           debug build of the workspace' \
		'make release         optimized ui binary (target/release/ui)' \
		'make run             open the debug window' \
		'make run-release     open the release window' \
		'make test            run workspace tests' \
		'make check           typecheck without codegen' \
		'make package         Sable.app plus dmg and zip in dist/' \
		'make install         package, then copy Sable.app to $(PREFIX)' \
		'make open            launch dist/Sable.app' \
		'make clean           delete target/ and dist/' \
		'' \
		'PREFIX=$(PREFIX)  VERSION=$(VERSION)'

build:
	$(CARGO) build --workspace

release:
	$(CARGO) build -p ui --release

run:
	$(CARGO) run -p ui

run-release: release
	./target/release/ui

test:
	$(CARGO) test --workspace

check:
	$(CARGO) check --workspace

package:
	./scripts/package-macos.sh

install: package
	rm -rf "$(PREFIX)/Sable.app"
	cp -R "$(APP)" "$(PREFIX)/Sable.app"
	@echo "Installed $(PREFIX)/Sable.app"

open:
	@if [[ ! -d "$(APP)" ]]; then $(MAKE) package; fi
	open "$(APP)"

clean:
	$(CARGO) clean
	rm -rf dist
