require-module menu

declare-option str skyspell_lang
declare-option str skyspell_project
declare-option range-specs skyspell_errors
declare-option int skyspell_error_count
declare-option str skyspell_word_to_add

set-face global SpellingError ,,red+c

define-command -params 1 skyspell-enable %{
  evaluate-commands %sh{
    echo "set global skyspell_lang $1"
    echo "set global skyspell_project $(pwd)"
  }
  add-highlighter global/spell ranges skyspell_errors
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
  map buffer normal 'g'      ':<space>skyspell-action add-global<ret>'
  map buffer normal 'l'      ':<space>skyspell-action add-lang<ret>'
  map buffer normal 'e'      ':<space>skyspell-action add-extension<ret>'
  map buffer normal 'p'      ':<space>skyspell-action add-project<ret>'
  map buffer normal 'f'      ':<space>skyspell-action add-file<ret>'
  map buffer normal 'u'      ':<space>skyspell-undo<ret>'
}

define-command skyspell-disable %{
  set global skyspell_error_count 0
  set global skyspell_errors 0
  remove-highlighter global/spell
  remove-hooks global skyspell
}

define-command -hidden skyspell-kak-on-failure %{
 fail "skyspell-kak failed - see *debug* for details"
}

define-command skyspell-check -docstring "check the open buffers for spelling errors" %{
  evaluate-commands %sh{
    : $kak_timestamp
    : $kak_opt_skyspell_project
    skyspell-kak --lang $kak_opt_skyspell_lang check $kak_quoted_buflist
    if [ $? -ne 0 ]; then
      echo skyspell-kak-on-failure
    fi
  }
}

define-command skyspell-undo -docstring "undo last operation" %{
  evaluate-commands %sh{
    : $kak_opt_skyspell_lang
    : $kak_opt_skyspell_project
    skyspell-kak --lang $kak_opt_skyspell_lang undo
  }
  write-all
  skyspell-check
  skyspell-list
}

define-command -hidden -params 1.. skyspell-action %{
  execute-keys gi Gl
  evaluate-commands %sh{
    : $kak_selection
    : $kak_opt_skyspell_project
    skyspell-kak --lang $kak_opt_skyspell_lang $*
    if [ $? -ne 0 ]; then
      echo skyspell-kak-on-failure
    fi
  }
}

define-command skyspell-help -docstring "show help message" %{
   info -title "Skyspell Help" "<ret>: Jump to spelling error
g : Add the word to the global ignore list
l : Add the word to the ignore list for the current lang
e : Add the word to the ignore list for this extension
p : Add the word to the ignore list for the current project
f : Add the word to the ignore list for this file
u : Undo last operation
"
}

define-command skyspell-list -docstring "list spelling errors" %{
   buffer *spelling*
   execute-keys gg
   skyspell-help
}

define-command skyspell-next -docstring "go to the next spelling error" %{
   evaluate-commands %sh{
     : $kak_opt_skyspell_project
     : $kak_cursor_line
     : $kak_cursor_column
     skyspell-kak --lang $kak_opt_skyspell_lang next-error "${kak_opt_skyspell_errors}"
     if [ $? -ne 0 ]; then
       echo skyspell-kak-on-failure
     fi
   }
}

define-command skyspell-previous -docstring "go to the previous spelling error" %{
   evaluate-commands %sh{
     : $kak_opt_skyspell_project
     : $kak_cursor_line
     : $kak_cursor_column
     skyspell-kak --lang $kak_opt_skyspell_lang previous-error "${kak_opt_skyspell_errors}"
     if [ $? -ne 0 ]; then
       echo skyspell-kak-on-failure
     fi
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
    skyspell-kak --lang $kak_opt_skyspell_lang suggest
     if [ $? -ne 0 ]; then
       echo skyspell-kak-on-failure
     fi
  }
}
