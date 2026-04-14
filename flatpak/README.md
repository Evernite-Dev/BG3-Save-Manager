# Flatpak packaging

## Prerequisites

Install the tools needed to generate offline dependency manifests:

```bash
pip install aiohttp toml
# flatpak-node-generator
curl -Lo flatpak-node-generator https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/node/flatpak-node-generator.py
# flatpak-cargo-generator
curl -Lo flatpak-cargo-generator https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
```

Also install flatpak-builder and the GNOME SDK:

```bash
flatpak install flathub org.gnome.Platform//49 org.gnome.Sdk//49
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable
flatpak install flathub org.freedesktop.Sdk.Extension.node20
```

## Regenerating offline sources

Run these from the repo root whenever `package-lock.json` or `Cargo.lock` changes:

```bash
python flatpak-node-generator npm package-lock.json -o flatpak/npm-sources.json
python flatpak-cargo-generator src-tauri/Cargo.lock -o flatpak/cargo-sources.json

# Note: paths in the manifest are relative to the manifest file (flatpak/),
# so the sources are referenced as cargo-sources.json / npm-sources.json inside the manifest.
```

Commit both generated files — they are required for the offline Flatpak build.

## Building locally

```bash
flatpak-builder --force-clean build-dir flatpak/com.evernite.BG3SaveManager.yml
flatpak-builder --run build-dir flatpak/com.evernite.BG3SaveManager.yml bg3-save-manager
```

## Submitting to Flathub

1. Fork https://github.com/flathub/flathub
2. Create a new branch named `new-pr`
3. Add a folder `com.evernite.BG3SaveManager/` containing the manifest, desktop file, and metainfo
4. Update the manifest's source to reference a specific git tag and commit hash
5. Open a pull request — Flathub CI will validate the build
