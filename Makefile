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

.PHONY: example-physics-balls-record
example-physics-balls-record:
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo build -r
	cd ${EXAMPLE_PHYSICS_BALLS} && asciinema rec --overwrite -c "cargo run -q -r" test.rec && agg test.rec test.gif && convert -loop 0 test.gif demo.gif && rm test.*
