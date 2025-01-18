edit some-text.txt
execute-keys i %{There is no mistake there} <ret> <esc>

skyspell-enable en

assert-equal "skyspell error count" %opt{skyspell_error_count} 0

quit
