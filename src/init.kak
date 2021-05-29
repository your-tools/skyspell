declare-option str kak_spell_lang
declare-option range-specs spell_errors
declare-option -hidden str kak_spell_current_error
declare-option str kak_spell_word_to_add

define-command -params 1 kak-spell-enable %{
  evaluate-commands %sh{
    echo "set global kak_spell_lang $1"
  }
  add-highlighter global/spell ranges spell_errors
  hook -group kak-spell global BufWritePost .* kak-spell-check
  hook -group kak-spell global BufCreate \*spelling\* kak-spell-hooks

}

define-command kak-spell-hooks %{
  map buffer normal '<ret>'  ':<space>kak-spell-action jump<ret>'
  map buffer normal 'a'      ':<space>kak-spell-action add-global<ret>'
  map buffer normal 'e'      ':<space>kak-spell-action add-extension<ret>'
  map buffer normal 'f'      ':<space>kak-spell-action add-file<ret>'
  map buffer normal 'n'      ':<space>kak-spell-action skip-name<ret>'
  map buffer normal 's'      ':<space>kak-spell-action skip-file<ret>'
}

define-command kak-spell-disable %{
  remove-highlighter buffer/spell
  remove-hooks global kak-spell
}

define-command kak-spell-check -docstring "check the current buffer for spelling errors" %{
  evaluate-commands %sh{
    : $kak_timestamp
    : $kak_opt_kak_spell_lang
    kak-spell check $kak_buflist
  }
}

define-command -hidden -params 1.. kak-spell-action %{
  execute-keys gi GL
  evaluate-commands %sh{
    : $kak_selection
    : $kak_opt_kak_spell_lang
    kak-spell $*
  }
}

define-command kak-spell-help -docstring "show help message" %{
   info -title "*spelling* Help" "h,j,k,l: Move
<ret>: Jump to spelling error
a : Add the word to the global ignore list
e : Add the word to the ignore list for this extension
f : Add the word to the ignore list for this path
n : Always skip this file name
s : Always skip this file
"
}

define-command kak-spell-list -docstring "list spelling errors" %{
   buffer *spelling*
   kak-spell-help
}



define-command kak-spell-next -docstring "go to the next spelling error" %{
   evaluate-commands %sh{
     : $kak_cursor_line
     : $kak_cursor_column
     kak-spell-action next-error "${kak_opt_spell_errors}"
   }
}

define-command kak-spell-previous -docstring "go to the previous spelling error" %{
   evaluate-commands %sh{
     : $kak_cursor_line
     : $kak_cursor_column
     kak-spell-action previous-error "${kak_opt_spell_errors}"
   }
}


define-command kak-spell-replace -docstring "replace the selection with a suggestion " %{
  evaluate-commands %sh{
    if [ -z "${kak_opt_kak_spell_lang}" ]; then
      printf %s\\n 'echo -markup {Error}The `kak_spell_lang` option is not set'
      exit 1
    fi
  }

  evaluate-commands %sh{
    : $kak_opt_kak_spell_lang
    : $kak_selection
    kak-spell suggest
  }

}
