edit some-text.txt
execute-keys i %{I'm testing skyspell here} <ret> <esc>
execute-keys i %{and an othher one there} <ret> <esc>

skyspell-enable en

skyspell-list

execute-keys -with-maps a

assert-global-config-contains global "skyspell"

quit

