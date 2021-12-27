set global makecmd "skyspell-ci run"

define-command -override skyspell-next %{
    make-next-error
    execute-keys E
}

define-command -override skyspell-previous %{
    make-previous-error
    execute-keys E
}

define-command -override skyspell-list %{
  edit -existing *make*
%}
