edit some-text.txt
execute-keys i %{I'm testing skyspell here} <ret> <esc>

skyspell-enable en

skyspell-list

execute-keys -with-maps p

assert-local-config-contains project "skyspell"

quit
