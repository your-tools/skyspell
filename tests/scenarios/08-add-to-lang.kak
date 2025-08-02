edit some-text.rs
execute-keys i %{skyspell} <ret> <esc>

skyspell-enable en_US

skyspell-list

execute-keys -with-maps l

assert-global-config-contains lang en skyspell

quit
