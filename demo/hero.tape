# VHS tape for recording hero demo GIF — one quick example of each tool.
# Usage: vhs demo/hero.tape
#
# Prerequisites:
#   - dtcat, dtfilter, dtdiff binaries built and in PATH
#   - Demo files in demo/

Output demo/hero.gif

Set FontSize 14
Set Width 1100
Set Height 550
Set Padding 15
Set Theme "GruvboxDarkHard"

Set TypingSpeed 80ms

Set Shell "bash"

Hide
    Type 'export PS1="> "'
    Enter
    Type "clear"
    Enter
Show

# ─── VIEW ───
Type "dtcat demo/sales.xlsx --head 5"
Enter
Sleep 3s

# ─── FILTER ───
Type "dtfilter demo/sales.xlsx --filter Region=East --sort Revenue:desc --limit 3"
Enter
Sleep 3s

# ─── DIFF ───
Type "dtdiff demo/old.xlsx demo/new.xlsx --key ID"
Enter
Sleep 4s
