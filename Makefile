# FS25-AutoDrive-Editor – Build & Deploy
#
# Verwendung:
#   make                  – Release-Builds (Linux + Windows)
#   make debug            – Debug-Builds (Linux + Windows)
#   make linux-release    – Nur Linux Release
#   make windows-debug    – Nur Windows Debug
#   make all              – Alles (Release + Debug, Linux + Windows)
#   make clean            – Binaries-Ordner aufräumen

BIN_NAME    := FS25-AutoDrive-Editor
TARGET_DIR  := /tmp/fs25_ad_editor_target
OUT_DIR     := binaries
WIN_TARGET  := x86_64-pc-windows-msvc

# Ausgabe-Dateinamen
LINUX_REL   := $(OUT_DIR)/$(BIN_NAME)_x64_linux
LINUX_DBG   := $(OUT_DIR)/$(BIN_NAME)_x64_linux_dbg
WIN_REL     := $(OUT_DIR)/$(BIN_NAME)_x64_windows.exe
WIN_DBG     := $(OUT_DIR)/$(BIN_NAME)_x64_windows_dbg.exe
WIN_DBG_PDB := $(OUT_DIR)/$(BIN_NAME)_x64_windows_dbg.pdb

.PHONY: all release debug linux windows \
        linux-release linux-debug windows-release windows-debug \
	check-layers ci-check clean

# Default: nur Release
release: linux-release windows-release

# Alles bauen
all: release debug

debug: linux-debug windows-debug

linux: linux-release linux-debug

windows: windows-release windows-debug

linux-release:
	cargo build --release
	@mkdir -p $(OUT_DIR)
	cp $(TARGET_DIR)/release/$(BIN_NAME) $(LINUX_REL)
	strip $(LINUX_REL)
	@ls -lh $(LINUX_REL)
	@echo "✓ $(LINUX_REL)"

linux-debug:
	cargo build
	@mkdir -p $(OUT_DIR)
	cp $(TARGET_DIR)/debug/$(BIN_NAME) $(LINUX_DBG)
	@ls -lh $(LINUX_DBG)
	@echo "✓ $(LINUX_DBG)"

windows-release:
	cargo xwin build --release --target $(WIN_TARGET)
	@mkdir -p $(OUT_DIR)
	cp $(TARGET_DIR)/$(WIN_TARGET)/release/$(BIN_NAME).exe $(WIN_REL)
	@ls -lh $(WIN_REL)
	@echo "✓ $(WIN_REL)"

windows-debug:
	cargo xwin build --target $(WIN_TARGET)
	@mkdir -p $(OUT_DIR)
	cp $(TARGET_DIR)/$(WIN_TARGET)/debug/$(BIN_NAME).exe $(WIN_DBG)
	@if [ -f "$(TARGET_DIR)/$(WIN_TARGET)/debug/$(BIN_NAME).pdb" ]; then \
		cp $(TARGET_DIR)/$(WIN_TARGET)/debug/$(BIN_NAME).pdb $(WIN_DBG_PDB); \
		ls -lh $(WIN_DBG_PDB); \
		echo "✓ $(WIN_DBG_PDB)"; \
	fi
	@ls -lh $(WIN_DBG)
	@echo "✓ $(WIN_DBG)"

clean:
	rm -f $(OUT_DIR)/$(BIN_NAME)_x64_*
	@echo "✓ $(OUT_DIR) aufgeräumt"

check-layers:
	@./scripts/check_layer_boundaries.sh

ci-check: check-layers
