edit 'some file.txt'
execute-keys i %{There is a misstake here} <ret> <esc>

skyspell-enable en

assert-equal "skyspell_error_count" %opt{skyspell_error_count} 1

skyspell-list

execute-keys -with-maps <ret>

assert-selection "misstake"

quit
