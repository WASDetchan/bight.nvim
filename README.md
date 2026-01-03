# bight.nvim - A neovim spreadsheet plugin
## Overview
[bight.nvim](https://github.com/WASDetchan/bight.nvim) is a neovim plugin fronted for [bight](https://github.com/WASDetchan/bight).  
It is insipired by Mircrosoft Excel, which is a great product but it has a few downsides, such as load times, its formula language, and not very customizable keymappings.  
Bight can:
- Store and edit spreadsheet data
- Evaluate formulas (wrtitten in Lua language)
- Export as .csva


Bigth is heavily WIP, and most features are not out yet.  
Bight does not yet support: 
- Loading or exporting to .xlsx or .ods
- Loading .csv
- Undoing operations
- Operating on table slices (ranges) in formulas in excel-like manner
- Configuration of keymaps
- Lua API
## Installation
[cargo](https://github.com/rust-lang/cargo) is required to build bight.nvim. No prebuilt binaries are currently shipped.
With [lazy.nvim](https://lazy.folke.io/):
```lua
{ 'WASDetchan/bight.nvim', opts = {} }
```
## Usage
Open a file with nvim
```bash
nvim file.bight
```
Use hjkl to move. Use I to edit cell in a separate buffer, or edit in-place with i or R. If you want to evaluate a formula start the cell's source with '=' with a lua expression following it ('=' as the first symbol of the cell will be changed to be 'return ' and the lua chunk will be evaluated. Use '\=' if you want the literal '='). Yank cell's source with yy or cell's evaluation result with Y. Paste into the cell with p. Enter visual mode with v. In visual mode use p to paste from clipboard to each of the selected cells, Y to yank values of the selected cells as comma-separated values.  
In formulas other cells may be referenced in excel-like manner. The cell positions start from A0. The column's index is the cell's letter coordinate with letters like digits of base 26 number system (so A is 0, B is 1, ..., Z is 25, BZ is 26).  
Available global lua functions:
- POSX(): x coordinate (column index) of the current cell 
- POSY(): y coordinate (row index) of the current cell
- REL(dx, dy): value of the cell dx to the right and dy down
- More formulas and and slice support are coming soon


