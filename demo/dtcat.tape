# VHS tape for recording dtcat demo GIF.
# Usage: vhs demo/dtcat.tape
#
# Prerequisites:
#   - dtcat binary built and in PATH
#   - Demo CSV files in demo/

Output demo/dtcat.gif

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

# ─── 1. VIEW A CSV FILE ───
Type "dtcat demo/sales.csv"
Enter
Sleep 3s

# ─── 2. SCHEMA ONLY ───
Type "dtcat demo/sales.csv --schema"
Enter
Sleep 2s

# ─── 3. SUMMARY STATISTICS ───
Type "dtcat demo/sales.csv --describe"
Enter
Sleep 3s

# ─── 4. HEAD AND TAIL ───
Type "dtcat demo/sales.csv --head 3 --tail 2"
Enter
Sleep 2s

# ─── 5. CSV OUTPUT ───
Type "dtcat demo/sales.csv --csv --head 5"
Enter
Sleep 3s
