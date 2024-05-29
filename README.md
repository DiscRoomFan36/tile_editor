# Tile Editor

Just a Tile Editor, after I get down the basics, (see [todo's](#todos)), want to add extra features that I would want in a tile editor, (adding notes to tiles on the grid, adding notes to the tiles themselves, ect)

The "Ethos" of this project, is that you make your tile edited thing in this program, and then when you save it, it's in a both: easily readable for humans format, and easily parsable for programmers format. aka why I use json.

If you need a more compressed format, make a build pipeline.

## Quick Start

```console
$ cargo run
```

## How to use

- Left click on a tile toc change it to the current tile selected
- Q/E to change tile selected
- P to Quick-save the grid
- L to Quick-load the grid
- W/S to resize the grid by rows
- A/D to resize the grid by cols

## TODO's

### For small extensions:

- Be able to change the size of the tiles in editor. (Zoom out with camera?)
- Make the editor state part of the saved grid json? (might just be for quick saves)
- Better save paths, (maybe keep the quick-save though)
- Add Load method that isn't quick-save
- Make saves pretty-printed, for readability
- Make a good ui
- More marks at the edges of the grid to mark where a tile is.
- Make the pallet be separated by folders, and stop it from crashing when loading something bad.
- Make some toggles for the tinting on the sprites in the grid.

### Some time in the future:

- Maybe remove _json_ dependency at some point?
- Add notes to tiles on the grid
- Add notes to the tiles themselves
- Add multiple layers (so a ground layer and an item layer)
- Add multiple floors
- Add extendible grid, aka add grids side by side you you could make a whole world
