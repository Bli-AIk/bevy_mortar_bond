#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "No repositories provided; nothing to do."
  exit 0
fi

for repo in "$@"; do
  if [ ! -d "${repo}" ]; then
    echo "Skipping ${repo}; directory does not exist."
    continue
  fi

  git_dir="${repo}/.git"
  if [ ! -d "${git_dir}" ] && [ ! -f "${git_dir}" ]; then
    echo "Skipping ${repo}; not a git repository."
    continue
  fi

  echo "Processing ${repo}..."
  git -C "${repo}" fetch origin
  if git -C "${repo}" show-ref --quiet --verify "refs/remotes/origin/develop"; then
    git -C "${repo}" checkout develop
    git -C "${repo}" reset --hard origin/develop
    echo "  -> checked out develop"
  else
    echo "  -> develop branch missing; keeping current branch"
  fi
done
