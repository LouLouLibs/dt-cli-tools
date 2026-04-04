# VHS tape for recording dtfilter demo GIF.
# Usage: vhs demo/dtfilter.tape
#
# Prerequisites:
#   - dtcat and dtfilter binaries built and in PATH
#   - Demo CSV files in demo/

Output demo/dtfilter.gif

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

# ─── 1. VIEW THE DATA ───
Type "dtcat demo/sales.csv"
Enter
Sleep 3s

# ─── 2. FILTER ROWS ───
Type "dtfilter demo/sales.csv --filter Region=East"
Enter
Sleep 3s

# ─── 3. NUMERIC COMPARISON ───
Type "dtfilter demo/sales.csv --filter 'Revenue>12000'"
Enter
Sleep 3s

# ─── 4. MULTIPLE FILTERS (AND) ───
Type "dtfilter demo/sales.csv --filter Region=East --filter 'Revenue>12000'"
Enter
Sleep 3s

# ─── 5. SELECT COLUMNS + SORT ───
Type "dtfilter demo/sales.csv --columns Region,Product,Revenue --sort Revenue:desc"
Enter
Sleep 3s

# ─── 6. LIMIT OUTPUT ───
Type "dtfilter demo/sales.csv --sort Revenue:desc --limit 3 --columns Product,Revenue"
Enter
Sleep 2s

# ─── 7. CONTAINS FILTER ───
Type "dtfilter demo/sales.csv --filter 'Product~Widget A' --columns Date,Region,Revenue"
Enter
Sleep 3s

# ─── 8. CSV OUTPUT FOR PIPING ───
Type "dtfilter demo/sales.csv --filter Region=West --csv"
Enter
Sleep 3s
