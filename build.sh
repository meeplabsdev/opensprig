#!/usr/bin/env bash
set -euo pipefail

DEBUG=0

for arg in "$@"; do
  case "$arg" in
    --debug)
      DEBUG=1
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      exit 1
      ;;
  esac
done

if [ "$DEBUG" -eq 1 ]; then
  cp memory.x.firmware memory.x
  cargo build
  echo

  arm-none-eabi-size -Ax build/thumbv6m-none-eabi/debug/firmware

  cp memory.x.parts memory.x
  for item in parts/*; do
    [ -e "$item" ] || continue
    if [[ -d "$item" ]]; then
      name="$(basename "$item")"
      (cd "$item" && RUSTFLAGS="-Clink-arg=-Tlink.x -Clink-arg=-Tdefmt.x" cargo build)
      arm-none-eabi-objcopy --remove-section=.boot2 -O binary "build/thumbv6m-none-eabi/debug/$name" "build/thumbv6m-none-eabi/debug/$name.bin"
    fi
  done

  rm memory.x
else
  cp memory.x.firmware memory.x
  DEFMT_LOG="off" cargo build --release
  echo

  arm-none-eabi-objcopy --remove-section=.defmt --remove-section=.comment build/thumbv6m-none-eabi/release/firmware
  arm-none-eabi-size -Ax build/thumbv6m-none-eabi/release/firmware

  cp memory.x.parts memory.x
  for item in parts/*; do
    [ -e "$item" ] || continue
    if [[ -d "$item" ]]; then
      name="$(basename "$item")"
      (cd "$item" && DEFMT_LOG="off" RUSTFLAGS="-Clink-arg=-Tlink.x -Clink-arg=-Tdefmt.x" cargo build --release)
      arm-none-eabi-objcopy --remove-section=.defmt --remove-section=.comment --remove-section=.boot2 -O binary "build/thumbv6m-none-eabi/release/$name" "build/thumbv6m-none-eabi/release/$name.bin"
    fi
  done
  
  rm memory.x
fi
