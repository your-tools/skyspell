edit some-text.txt
execute-keys i %{There is a misstake here} <ret> <esc>
execute-keys i %{and an othher one there} <ret> <esc>
execute-keys i %{last} <ret> <esc>

skyspell-enable en

execute-keys .

execute-keys k w
assert-equal "selection" "%val{selection}" "last"

write # So that `quit` does not error with "unmodified buffer"

quit
