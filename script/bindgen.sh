#!/usr/bin/env bash

#------------------------
# BOOTSTRAP
#------------------------

SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
  DIR="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE" # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
DIR="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)"

. "$DIR/env.sh"

IDF_COMPONENTS=$IDF_PATH/components
LIBS=$(find $IDF_COMPONENTS -name "include" | grep -v esp32s2 | xargs -I{} echo -n "-I '{}' ")

TOOLS=$(realpath $(dirname $(which xtensa-esp32-elf-gcc))/..)
TOOL_INCLUDE=$TOOLS/xtensa-esp32-elf/include

# export BINDGEN_OPTS="--no-layout-tests --use-core --size_t-is-usize --no-prepend-enum-name --ctypes-prefix cty --default-enum-style rust"
export BINDGEN_OPTS="--use-core --size_t-is-usize --raw-line '#![allow(non_camel_case_types, non_upper_case_globals)]'"
export BINDGEN_CLANG_OPTS="'-D__GLIBC_USE(x)=0' -DSSIZE_MAX -I '$DIR/include' -I '$TOOL_INCLUDE' -I '$IDF_COMPONENTS/freertos/include' $LIBS -I '$IDF_COMPONENTS/lwip/include/apps/sntp/' -I '$IDF_COMPONENTS/lwip/include/apps'"

#MAKEFILES=$(find "$(dirname "$DIR")/sys" -name "Makefile")
# echo "$MAKEFILES"

find "$(dirname "$DIR")/sys" -name "Makefile" -execdir bash -c 'make' {} \;
