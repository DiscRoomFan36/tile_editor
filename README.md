# Bevy Tile Editor

Just a Tile Editor, after I get down the basics, (see [todo's](#todos)), want to add extra features that I would want in a tile editor, (adding notes to tiles on the grid, adding notes to the tiles themselves, ect)

The "Ethos" of this project, is that you make your tile edited thing in this program, and then when you save it, it's in a both: easily readable for humans format, and easily parsable for programmers format. aka why I use json.

If you need a more compressed format, make a build pipeline.

## How to use

- Left click on a tile toc change it to the current tile selected
- Q/E to change tile selected
- P to Quick-save the grid
- L to Quick-load the grid

## Quick Start

Probably not so quick, gotta build bevy in release

```console
$ cargo run
```

## TODO's

### For basic tile editor (1.0?):

- Make Pallet (tile selector so you can see what your doing)
- Use icon name instead of index for icon selector (will make saves more readable)
- Be able to change the default, and make default more obvious
- Something to add more tiles to the pallet
- Remove rainbow colors, (but keep a secret toggle?)

### For small extensions:

- Use sparse grid for saved output
- Make the editor state part of the saved grid json? (might just be for quick saves)
- Better save paths, (maybe keep the quick-save though)
- Add Load method that isn't quick-save
- Make saves pretty-printed, for readability
- change grid size, (and keep current grid)
- Make a ui (if bevy ui uses css, im not using bevy ui)

### Some time in the future:

- Maybe remove _json_ dependency at some point? (Bevy is already a lot)
- Add notes to tiles on the grid
- Add notes to the tiles themselves
- Add multiple layers (so a ground layer and an item layer)
- Add multiple floors
- Add extendible grid, aka add grids side by side you you could make a whole world
