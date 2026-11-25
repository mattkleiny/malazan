set shell := ["bash", "-cu"]

godot := env("GODOT", "godot4")
project := "godot"
builds := "builds"

# Lint the workspace with Clippy; warnings fail.
clippy:
    CARGO_TARGET_DIR=target/clippy cargo clippy --workspace --all-targets -- -D warnings

# Run the test suite.
test:
    cargo test --workspace

# `--no-default-features` leaves out editor-only code (plugins, gizmos, ECS debugger and profiler) and `register-docs`,
# so neither editor classes nor F1 class docs are built into exported binaries.
# `steam` connects to Steam when it is running and plays on without it. Leave STEAM_APP_ID unset for
# builds shipped outside Steam: baking it in makes the game quit and relaunch through Steam.
# Build the GDExtension (release).
build *args:
    cargo build -p game --release --no-default-features --features steam {{args}}

# Build a universal (arm64 + x86_64) macOS GDExtension at the path the export expects.
build-macos-universal *args: (build "--target aarch64-apple-darwin" args) (build "--target x86_64-apple-darwin" args)
    mkdir -p target/release
    lipo -create -output target/release/libgame.dylib \
      target/aarch64-apple-darwin/release/libgame.dylib \
      target/x86_64-apple-darwin/release/libgame.dylib

# A fresh checkout has no `.godot/` cache, and exporting without one fails on unregistered
# global classes. `.blend` models need `filesystem/import/blender/blender_path` in the editor settings.
# Import project assets (run before exporting a fresh checkout).
import:
    {{godot}} --headless --path {{project}} --import

# Stamp application/config/version from VERSION so the main menu and exporters see it.
# A leading v is stripped so the menu's "v%s" is not doubled. No-op when VERSION is unset.
stamp-version:
    #!/usr/bin/env python3
    import os
    import re
    from pathlib import Path

    raw = os.environ.get("VERSION", "").strip()
    if not raw:
        print("VERSION unset; leaving application/config/version empty (menu shows dev)")
        raise SystemExit(0)

    version = raw[1:] if raw.startswith("v") else raw
    path = Path("godot/project.godot")
    text = path.read_text()
    pattern = r'(?m)^config/version=.*$'
    if re.search(pattern, text):
        text = re.sub(pattern, f'config/version="{version}"', text, count=1)
    else:
        text = re.sub(
            r'(?m)^\[application\]\s*$',
            f'[application]\nconfig/version="{version}"',
            text,
            count=1,
        )
    path.write_text(text)
    print(f"application/config/version={version}")

# Export one preset from `export_presets.cfg` without rebuilding the GDExtension.
export preset: stamp-version
    mkdir -p {{builds}}/$(echo "{{preset}}" | tr '[:upper:]' '[:lower:]')
    {{godot}} --headless --path {{project}} --export-release "{{preset}}"

# Export a full release build for macOS (preset path → builds/macos/Game.zip).
export-macos: build-macos-universal import (export "macOS")

# Export a full release build for Linux (preset path → builds/linux/Game.x86_64).
export-linux: build import (export "Linux")

# Export a full release build for Windows (preset path → builds/windows/Game.exe).
export-windows: build import (export "Windows")
