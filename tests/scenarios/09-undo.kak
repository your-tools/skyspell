edit some-text.txt
execute-keys i %{skyspell} <ret> <esc>

skyspell-enable en_US

skyspell-list

execute-keys -with-maps a

assert-equal "skyspell error count" %opt{skyspell_error_count} 0

assert-global-config-contains global skyspell

execute-keys -with-maps u

assert-equal "skyspell error count" %opt{skyspell_error_count} 1

quit
