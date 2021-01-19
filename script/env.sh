# This script should be sourced, not executed.

realpath_int() {
  wdir="$PWD"
  [ "$PWD" = "/" ] && wdir=""
  arg=$1
  case "$arg" in
  /*) scriptdir="${arg}" ;;
  *) scriptdir="$wdir/${arg#./}" ;;
  esac
  scriptdir="${scriptdir%/*}"
  echo "$scriptdir"
}

ENV_BOOTSTRAP_VERSION="1"
env_init_main() {
  if ! [ -z "$ENV_BOOTSTRAPED" ]; then
    return
  fi

  # The file doesn't have executable permissions, so this shouldn't really happen.
  # Doing this in case someone tries to chmod +x it and execute...

  # shellcheck disable=SC2128,SC2169,SC2039 # ignore array expansion warning
  if [ -n "${BASH_SOURCE}" ] && [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    echo "This script should be sourced, not executed:"
    # shellcheck disable=SC2039  # reachable only with bash
    echo ". ${BASH_SOURCE[0]}"
    return 1
  fi

  self_path=""

  # shellcheck disable=SC2128  # ignore array expansion warning
  if [ -n "${BASH_SOURCE}" ];        then
    self_path="${BASH_SOURCE}"
  elif [ -n "${ZSH_VERSION}" ];        then
    self_path="${(%):-%x}"
  else
    echo "Could not detect IDF_PATH. Please set it before sourcing this script:"
    echo "  export IDF_PATH=(add path here)"
    return 1
  fi

  # shellcheck disable=SC2169,SC2169,SC2039  # unreachable with 'dash'
  if [[ "$OSTYPE" == "darwin"* ]]; then
    # convert possibly relative path to absolute
    script_dir="$(realpath_int "${self_path}")"
    # resolve any ../ references to make the path shorter
    script_dir="$(
      cd "${script_dir}" || exit 1
      pwd
    )"
  else
    # convert to full path and get the directory name of that
    script_name="$(readlink -f "${self_path}")"
    script_dir="$(dirname "${script_name}")"
  fi

  export IDF_PATH="$(dirname "${script_dir}")/esp-idf"
  "$IDF_PATH/install.sh"
  . "$IDF_PATH/export.sh"

  export ENV_BOOTSTRAPED="1"
}

env_init_main

unset realpath_int
unset env_init_main
unset ENV_BOOTSTRAP_VERSION
