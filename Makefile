EXAMPLE_DIR:=examples
EXAMPLE_PHYSICS_BALLS:=${EXAMPLE_DIR}/physics-balls
EXAMPLE_SPINNING_DIAMOND:=${EXAMPLE_DIR}/spinning-diamond

.PHONY: fmt
fmt:
	cargo fmt
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo fmt
	cd ${EXAMPLE_SPINNING_DIAMOND} && cargo fmt

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo clippy -- -D warnings
	cd ${EXAMPLE_SPINNING_DIAMOND} && cargo clippy -- -D warnings

.PHONY: check
check: fmt clippy

.PHONY: example-physics-balls
example-physics-balls:
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo run

.PHONY: example-physics-balls-record
example-physics-balls-record:
	cd ${EXAMPLE_PHYSICS_BALLS} && cargo build -r
	cd ${EXAMPLE_PHYSICS_BALLS} && asciinema rec --overwrite -c "cargo run -q -r" test.rec && agg test.rec test.gif && convert -loop 0 test.gif demo.gif && rm test.*

.PHONY: example-spinning-diamond
example-spinning-diamond:
	cd ${EXAMPLE_SPINNING_DIAMOND} && cargo run

.PHONY: example-spinning-diamond-record
example-spinning-diamond-record:
	cd ${EXAMPLE_SPINNING_DIAMOND} && cargo build -r
	cd ${EXAMPLE_SPINNING_DIAMOND} && asciinema rec --overwrite -c "cargo run -q -r" test.rec && agg test.rec test.gif && convert -loop 0 test.gif demo.gif && rm test.*
