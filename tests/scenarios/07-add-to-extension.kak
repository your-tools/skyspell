edit some-text.rs
execute-keys i %{fn do_stuff()} <ret> <esc>

skyspell-enable en

skyspell-list

execute-keys -with-maps e

assert-global-config-contains extensions "rs" "fn"

quit
