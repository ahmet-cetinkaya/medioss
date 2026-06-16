.PHONY: format

# Format all supported files in-place
format:
	prettier --write **/*.md
