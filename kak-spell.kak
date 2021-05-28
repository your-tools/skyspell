declare-option str kak_spell_lang
declare-option range-specs spell_errors
declare-option -hidden str kak_spell_current_error
declare-option str kak_spell_word_to_add

define-command -params 1 kak-spell-enable %{
  evaluate-commands %sh{
    echo "set buffer kak_spell_lang $1"
  }
  add-highlighter buffer/spell ranges spell_errors
  hook -group kak-spell buffer BufWritePost .* kak-spell
}

define-command kak-spell-disable %{
  remove-highlighter buffer/spell
  remove-hooks buffer kak-spell
}

define-command kak-spell -docstring "check the current buffer for spelling errors" %{
  evaluate-commands %sh{
    kak-spell \
      --lang "${kak_opt_kak_spell_lang}" \
      check \
      --filetype "${kak_opt_filetype}" \
      "${kak_buffile}" \
      --kak-timestamp ${kak_timestamp} \
      --kakoune
  }
}

define-command kak-spell-list -docstring "list spelling errors" %{
   evaluate-commands %sh{
    kak-spell \
      --lang "${kak_opt_kak_spell_lang}" \
      list \
      --filetype "${kak_opt_filetype}" \
      "${kak_buffile}" \
   }
   info -title "*spelling* Help" "h,j,k,l: Move
<ret>: Jump to spelling error
a    : Add the word to the personal dictionary
"
}

define-command kak-spell-jump -hidden %{
  edit -existing *spelling*
  execute-keys  gi <a-E>
  set-option global kak_spell_current_error %val{selection}
  execute-keys ga
  select %opt{kak_spell_current_error}
}

define-command kak-spell-add-from-spelling-buffer -params 1 -hidden %{
  execute-keys gi <a-w> l Gl
  evaluate-commands %sh{
    word="$kak_selection"
    kak-spell --quiet --lang $1 add $word
  }
  execute-keys ga
  kak-spell
  kak-spell-list
}

define-command kak-spell-next -docstring "go to the next spelling error" %{
   evaluate-commands %sh{
     kak-spell next \
      --ranges "${kak_opt_spell_errors}" \
      --pos "${kak_cursor_line}.${kak_cursor_column}"
   }
}

define-command kak-spell-previous -docstring "go to the previous spelling error" %{
   evaluate-commands %sh{
     kak-spell previous \
      --ranges "${kak_opt_spell_errors}" \
      --pos "${kak_cursor_line}.${kak_cursor_column}"
   }
}

define-command kak-spell-add -params 0..1 -docstring "add the selection to the user dict" %{
  evaluate-commands %sh{
    if [ -z "${kak_opt_kak_spell_lang}" ]; then
      printf %s\\n 'echo -markup {Error}The `kak_spell_lang` option is not set'
      exit 1
    fi
  }
  nop %sh{
    if [ -z "$1" ]; then
      kak-spell --lang $kak_opt_kak_spell_lang add $kak_selection
    else
      kak-spell --lang $kak_opt_kak_spell_lang add $1
    fi
  }
  write
}

define-command kak-spell-remove -params 0..1 -docstring "remove the selection from the user dict" %{
  evaluate-commands %sh{
    if [ -z "${kak_opt_kak_spell_lang}" ]; then
      printf %s\\n 'echo -markup {Error}The `kak_spell_lang` option is not set'
      exit 1
    fi
  }
  nop %sh{
    if [ -z "$1" ]; then
      kak-spell --lang $kak_opt_kak_spell_lang remove $kak_selection
    else
      kak-spell --lang $kak_opt_kak_spell_lang remove $1
    fi
  }
  write
}

define-command kak-spell-replace -docstring "replace the selection with a suggestion " %{
  evaluate-commands %sh{
    if [ -z "${kak_opt_kak_spell_lang}" ]; then
      printf %s\\n 'echo -markup {Error}The `kak_spell_lang` option is not set'
      exit 1
    fi
  }

  evaluate-commands %sh{ kak-spell --lang $kak_opt_kak_spell_lang replace $kak_selection --kakoune }
}
