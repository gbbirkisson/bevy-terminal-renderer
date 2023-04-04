EXAMPLE_DIR:=examples
EXAMPLE_PHYSICS_BALLS:=${EXAMPLE_DIR}/physics-balls

.PHONY: fmt
fmt:
	cargo fmt
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo fmt

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo clippy -- -D warnings

.PHONY: check
check: fmt clippy

.PHONY: example-physics-balls
example-physics-balls:
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo run
