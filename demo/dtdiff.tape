# VHS tape for recording dtdiff demo GIF.
# Usage: vhs demo/dtdiff.tape
#
# Prerequisites:
#   - dtcat and dtdiff binaries built and in PATH
#   - Demo files in demo/

Output demo/dtdiff.gif

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

# ─── 1. SHOW THE INPUT FILES ───
Type "dtcat demo/old.xlsx"
Enter
Sleep 3s

Type "dtcat demo/new.xlsx"
Enter
Sleep 3s

# ─── 2. KEY-BASED DIFF ───
Type "dtdiff demo/old.xlsx demo/new.xlsx --key ID"
Enter
Sleep 4s

# ─── 3. JSON OUTPUT ───
Type "dtdiff demo/old.xlsx demo/new.xlsx --key ID --json"
Enter
Sleep 4s
