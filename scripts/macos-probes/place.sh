k() { osascript -e "tell application \"System Events\" to $1"; }
osascript -e 'tell application "Finder" to activate'; sleep 1
k 'tell process "Finder" to count windows'
for i in 1 2 3; do osascript -e 'tell application "Finder" to make new Finder window'; sleep 0.7; done
echo finder-windows; k 'tell process "Finder" to get {position, size} of every window'
k 'keystroke "g" using {command down, shift down}'; sleep 1
echo goto-sheet-vs-window; k 'tell process "Finder" to get {position, size} of sheet 1 of window 1'; k 'tell process "Finder" to get {position, size} of window 1'
k 'key code 53'; sleep 0.5
for i in 1 2 3; do k 'keystroke "w" using command down'; sleep 0.5; done
osascript -e 'tell application "TextEdit" to activate'; sleep 1
k 'keystroke "o" using command down'; sleep 1.5
echo textedit-open-panel; k 'tell process "TextEdit" to get {name, position, size} of every window'
k 'key code 53'; sleep 0.5
k 'tell process "TextEdit" to count windows'
