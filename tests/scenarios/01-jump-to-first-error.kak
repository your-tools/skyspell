edit some-text.txt
execute-keys i %{There is a misstake here} <ret> <esc>
execute-keys i %{and an othher one there} <ret> <esc>

skyspell-enable en

assert-equal "skyspell_error_count" %opt{skyspell_error_count} 2

skyspell-list

execute-keys -with-maps <ret>

assert-selection "misstake"

quit
