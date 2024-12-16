all: prettier format check lint-fix test
.PHONY: all

format:
	@cargo fmt --all
.PHONY: format-fix

check:
	@cargo check --all
.PHONY: check

format-check:
	@cargo fmt --all -- --check
.PHONY: format

prettier:
	@COMPOSE_PROJECT_NAME=challenge docker compose -f scripts/prettier/compose.prettier.yml run --rm prettier
.PHONY: prettier

prettier-build:
	@COMPOSE_PROJECT_NAME=challenge docker compose -f scripts/prettier/compose.prettier.yml --progress=plain build prettier
.PHONY: prettier-build

lint:
	@cargo clippy --all -- -D warnings
.PHONY: lint

lint-fix:
	@cargo clippy --all --fix --allow-dirty --allow-staged
.PHONY: lint-fix

test:
	@cargo test
.PHONY: test

test-compile:
	@cargo test --no-run --locked
.PHONY: test-compile

duplexscan:
	@docker build --progress=plain -f build.Dockerfile --output type=local,dest=. .
.PHONY: duplexscan

duplexscan-glibc:
	@docker build --progress=plain -f glibc.Dockerfile --output type=local,dest=. .
.PHONY: duplexscan-glibc

%:
	@:

