edit some-text.txt
execute-keys i %{I'm testing skyspell here} <ret> <esc>

skyspell-enable en

skyspell-list

execute-keys -with-maps f

assert-local-config-contains paths "some-text.txt" "skyspell"

quit
