edit some-text.txt
execute-keys i %{There is a misstake here} <ret> <esc>
execute-keys i %{and an othher one there} <ret> <esc>

skyspell-enable en_US

assert-equal "skyspell_error_count" %opt{skyspell_error_count} 2

skyspell-list

execute-keys -with-maps <ret>

assert-selection "misstake"

skyspell-replace

# TODO: we can't force kakoune to select the first entry in the menu, so let's just call "quit" here
# You should instead have two hidden commands
#  - skyspell-suggest
#  - skyspell-replace-impl
#  that you call when `skyspell-replace` generates the `menu` command
quit
