CRATE_VERSION := `cargo pkgid | cut -d "#" -f2`

build-image:
	docker build . -t localhost:5432/ecu_dashboard:{{CRATE_VERSION}} --network=host

publish-image:
	docker push localhost:5432/ecu_dashboard:{{CRATE_VERSION}}

publish-crate:
	cargo publish
	just build-image
	just publish-image
	git tag v{{CRATE_VERSION}}

check-version-bump revision='origin/main':
	@if git diff --quiet {{revision}} HEAD -- ./**/Cargo.toml ./**/src/; then \
		echo "[NO_BUMP_REQUIRED] No changes in Cargo.toml or src/ files since v{{CRATE_VERSION}}, no need to bump version"; \
		exit 0; \
	else \
		just _check-version-incremented {{revision}} `git show {{revision}}:Cargo.toml | grep '^version =' | head -n 1 | cut -d '"' -f2`; \
	fi

_check-version-incremented revision OLD_VERSION:
	@if echo -e "{{CRATE_VERSION}}\n{{OLD_VERSION}}" | sort -V | head -n 1 | grep -q "{{CRATE_VERSION}}"; then \
		echo "[BUMP_REQUIRED] Current version {{CRATE_VERSION}} is not greater than the version at {{revision}} ({{OLD_VERSION}})!"; \
		exit 1; \
	else \
		echo "[NO_BUMP_REQUIRED] Version has been incremented correctly."; \
	fi

clippy:
	cargo clippy --all-features -- -D warnings

test:
	cargo nextest r --all-features --no-tests warn

test-release:
	cargo nextest r --all-features --release --no-tests warn