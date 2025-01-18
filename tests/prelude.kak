# Tests helpers for each scenario

face global Error white,red # Make errors more readable on a light background

# skyspell-kak depends on menu, but we run kak with -n, so we have
# to source it explicitly
source "%val[runtime]/autoload/tools/menu.kak"

evaluate-commands %sh{
  skyspell-kak init
}

define-command -params 3 assert-equal %{
  evaluate-commands %sh{
    name="$1"
    actual="$2"
    expected="$3"
    if [ "${actual}" != "${expected}" ];
      then echo "fail incorrect $name: actual: $actual, expected: $expected"
    fi
  }
}

define-command -params 1 assert-selection %{
  evaluate-commands %sh{
    echo assert-equal "selection" "$kak_selection" "$1"
  }
}

define-command assert-config-contains \
  -params 3.. \
  -docstring "Usage: assert-config-key [global|local] [key...] expected-value
  Check that inside the global or local config file, the expected value is present
  key_path should be a list of nested keys,  separated by ' . ', like
  assert-config-contains local 'paths . some.txt' value'
  "  \
%{
  evaluate-commands %sh{
    python "$SKYSPELL_TESTS_PATH/assert_config_contains.py" $*
    if [ $? -ne 0 ];
      then echo fail "assert_config_contains script failed"
    fi
  }
}

define-command assert-global-config-contains -params 2.. -docstring "partial for assert-config-contains global" %{
  evaluate-commands %sh{
    echo assert-config-contains "global" $*
  }
}

define-command assert-local-config-contains -params 2.. -docstring "partial for assert-config-contains local" %{
  evaluate-commands %sh{
    echo assert-config-contains "local" $*
  }
}

