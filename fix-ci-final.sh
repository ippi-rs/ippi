#!/bin/sh
set -e

echo "Adding all changes..."
git add .

echo "Committing fixes for CI..."
git commit -m "Fix CI test and frontend failures

- Update frontend dependencies: Svelte 5, Vite 6, @sveltejs/vite-plugin-svelte 5
- Fix Svelte 5 warnings: replace on:click with onclick, remove unused body CSS
- Fix Rust module resolution: remove external mod declarations for inline modules
  - kvm: remove external vm module, keep inline
  - p2p: remove external network module, keep inline  
  - webrtc: remove external rtc module, keep inline
- Add missing hardware.rs module
- Update rust-version from 1.94 to 1.91 for compatibility
- Fix cargo fmt issues across codebase
- Ensure frontend builds without warnings or errors
- Remove root package.json and package-lock.json (frontend only)
- Maintain Podman/Containerfile setup"

echo "Pushing to GitHub..."
git push origin main

echo "✅ All fixes committed and pushed"