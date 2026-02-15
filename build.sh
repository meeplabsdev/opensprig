DEFMT_LOG="off" cargo build --release
echo

arm-none-eabi-objcopy --remove-section=.defmt --remove-section=.comment build/thumbv6m-none-eabi/release/firmware
arm-none-eabi-size -Ax build/thumbv6m-none-eabi/release/firmware

for item in parts/*; do
  [ -e "$item" ] || continue
  if [[ -d "$item" ]]; then
    DEFMT_LOG="off" RUSTFLAGS="-Clink-arg=-Tlink.x -Clink-arg=-Tdefmt.x" cargo build -p "$(basename "$item")" --release
    arm-none-eabi-objcopy --remove-section=.defmt --remove-section=.comment -O binary "build/thumbv6m-none-eabi/release/$(basename "$item")" "build/thumbv6m-none-eabi/release/$(basename "$item").bin"
  fi
done