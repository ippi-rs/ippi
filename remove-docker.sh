#!/bin/sh
set -e

echo "Removing Docker and switching to Podman/Containerfile..."

echo "Adding changed files..."
git add .

echo "Committing changes..."
git commit -m "Remove Docker, switch to Podman/Containerfile, fix ARM cross-compilation

- Remove Docker job from GitHub Actions CI
- Rename Dockerfile to Containerfile (Podman native)
- Rename docker-compose.yml to compose.yml
- Maintain symlinks for backward compatibility
- Update podman-compose.sh to use -f compose.yml
- Switch ARM cross-compilation to use 'cross' for reliable builds
- Install cross tool for all cross-compilation targets
- Keep Node.js 24 compatibility (FORCE_JAVASCRIPT_ACTIONS_TO_NODE24)"

echo "Pushing to GitHub..."
git push origin main

echo "✅ Docker removed, Podman/Containerfile setup complete"