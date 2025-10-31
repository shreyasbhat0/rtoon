.PHONY: changelog changelog-version help

help:
	@echo "Available targets:"
	@echo "  changelog          - Generate/update CHANGELOG.md for all versions"
	@echo "  changelog-version  - Generate changelog for specific version (usage: make changelog-version VERSION=0.1.2)"
	@echo "  install-cliff     - Install git-cliff via cargo"

changelog:
	@./scripts/generate-changelog.sh

changelog-version:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make changelog-version VERSION=0.1.2"; \
		exit 1; \
	fi
	@./scripts/generate-changelog.sh $(VERSION)

install-cliff:
	@cargo install git-cliff

