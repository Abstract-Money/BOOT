#!/bin/bash
export PATH=$PATH:/usr/local/bin

# Requirements:
# 1. Have rust and cargo installed
# 2. Have taplo installed (cargo install taplo-cli --locked)
#
# pre-push hook, used to perform static analysis checks on changed files.
#
# Install the hook with the --install option.

project_toplevel=$(git rev-parse --show-toplevel)
git_directory=$(git rev-parse --git-dir)

install_hook() {
  mkdir -p "$git_directory/hooks"
  ln -sfv "$project_toplevel/hooks/pre-push.sh" "$git_directory/hooks/pre-push"
}

if [ "$1" = "--install" ]; then
  if [ -f "$git_directory/hooks/pre-push" ]; then
    read -r -p "There's an existing pre-push hook. Do you want to overwrite it? [y/N] " response
    case "$response" in
    [yY][eE][sS] | [yY])
      install_hook
      ;;
    *)
      printf "Skipping hook installation :("
      exit $?
      ;;
    esac
  else
    install_hook
  fi
  exit $?
fi

# cargo and toml fmt checks
format_check() {
  printf "Starting file formatting check...\n"

  cd $project_toplevel || exit;
  # format rust code
  cargo fmt;
  # format toml files
  find . -type f -iname "*.toml" -print0 | xargs -0 taplo format

  sleep 1; # Give git time to find changed files.
  not_staged_file=$(git diff --name-only)
    if [ "$not_staged_file" != "" ]; then # it means the file changed and it's not staged, i.e. rustfmt did the job.
      git add .
      git commit -m "formatting"
    fi
}

# clippy checks
lint_check() {
  printf "Starting clippy check...\n"
  cargo clippy --quiet -- -D warnings
  clippy_exit_code=$?
  if [ $clippy_exit_code -ne 0 ]; then
    printf "\nclippy found some issues. Fix them manually or run cargo clippy --fix."
    exit 1
  fi
}

lint_check
format_check
