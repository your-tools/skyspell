edit some-text.txt
execute-keys i %{There is a misstake here} <ret> <esc>
execute-keys i %{and an othher one there} <ret> <esc>

skyspell-enable en_US

assert-equal "skyspell_error_count" %opt{skyspell_error_count} 2

execute-keys 1 g
execute-keys 22 l

skyspell-next

assert-equal "selection" "%val{selection}" "othher"

skyspell-previous

assert-equal "selection" "%val{selection}" "misstake"

quit

