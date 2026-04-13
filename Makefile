.PHONY: build install clean

# The name of the resulting binary
BINARY_NAME = ghcli

# The directory where the binary will be installed
# Using ~/.local/bin is standard for user-specific binaries without requiring root (sudo)
INSTALL_DIR = $(HOME)/.local/bin

build:
	cargo build --release

install: build
	@echo "Installing $(BINARY_NAME) to $(INSTALL_DIR)..."
	mkdir -p $(INSTALL_DIR)
	cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Successfully installed! Make sure $(INSTALL_DIR) is in your PATH."

clean:
	cargo clean
