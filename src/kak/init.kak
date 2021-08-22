declare-option str skyspell_lang
declare-option str skyspell_project
declare-option range-specs spell_errors
declare-option int skyspell_error_count
declare-option str skyspell_word_to_add

define-command -params 1 skyspell-enable %{
  evaluate-commands %sh{
    echo "set global skyspell_lang $1"
    echo "set global skyspell_project $(pwd)"
  }
  add-highlighter global/spell ranges spell_errors
  hook -group skyspell global BufWritePost .* skyspell-check
  hook -group skyspell global BufCreate \*spelling\* skyspell-hooks
  # If we've just enable spell checking *and* the current buffer is modified,
  # we want to spell check the current buffer right away.
  # On the other hand, maybe kakoune is still editing the *scratch* buffer at this point
  try %{
    write
  }
}

define-command skyspell-hooks %{
  map buffer normal '<ret>'  ':<space>skyspell-action jump<ret>'
  map buffer normal 'a'      ':<space>skyspell-action add-global<ret>'
  map buffer normal 'e'      ':<space>skyspell-action add-extension<ret>'
  map buffer normal 'p'      ':<space>skyspell-action add-project<ret>'
  map buffer normal 'f'      ':<space>skyspell-action add-file<ret>'
  map buffer normal 'n'      ':<space>skyspell-action skip-name<ret>'
  map buffer normal 's'      ':<space>skyspell-action skip-file<ret>'
}

define-command skyspell-disable %{
  remove-highlighter global/spell
  remove-hooks global skyspell
}

define-command skyspell-check -docstring "check the open buffers for spelling errors" %{
  evaluate-commands %sh{
    : $kak_timestamp
    : $kak_opt_skyspell_project
    skyspell --lang $kak_opt_skyspell_lang kak check $kak_buflist
  }
}

define-command -hidden -params 1.. skyspell-action %{
  execute-keys gi GL
  evaluate-commands %sh{
    : $kak_selection
    : $kak_opt_skyspell_project
    skyspell --lang $kak_opt_skyspell_lang kak $*
  }
}

define-command skyspell-help -docstring "show help message" %{
   info -title "Skyspell Help" "<ret>: Jump to spelling error
a : Add the word to the global ignore list
e : Add the word to the ignore list for this extension
p : Add the word to the ignore list for the current project
f : Add the word to the ignore list for this file
n : Always skip this file name
s : Always skip this file
"
}

define-command skyspell-list -docstring "list spelling errors" %{
   buffer *spelling*
   skyspell-help
}



define-command skyspell-next -docstring "go to the next spelling error" %{
   evaluate-commands %sh{
     : $kak_opt_skyspell_project
     : $kak_cursor_line
     : $kak_cursor_column
     skyspell --lang $kak_opt_skyspell_lang kak next-error "${kak_opt_spell_errors}"
   }
}

define-command skyspell-previous -docstring "go to the previous spelling error" %{
   evaluate-commands %sh{
     : $kak_opt_skyspell_project
     : $kak_cursor_line
     : $kak_cursor_column
     skyspell --lang $kak_opt_skyspell_lang kak previous-error "${kak_opt_spell_errors}"
   }
}


define-command skyspell-replace -docstring "replace the selection with a suggestion " %{
  evaluate-commands %sh{
    if [ -z "${kak_opt_skyspell_lang}" ]; then
      printf %s\\n 'echo -markup {Error}The `skyspell_lang` option is not set'
      exit 1
    fi

    if [ -z "${kak_selection}" ]; then
      printf %s\\n 'echo -markup {Error}The selection is empty'
      exit 1
    fi
  }

  evaluate-commands %sh{
    : $kak_opt_skyspell_project
    : $kak_selection
    skyspell --lang $kak_opt_skyspell_lang kak suggest
  }

}
