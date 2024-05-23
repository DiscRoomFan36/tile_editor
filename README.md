# Bevy Tile Editor

Just a Tile Editor, after I get down the basics, (see [todo's](#todos)), want to add extra features that I would want in a tile editor, (adding notes to tiles on the grid, adding notes to the tiles themselves, ect)

The "Ethos" of this project, is that you make your tile edited thing in this program, and then when you save it, it's in a both: easily readable for humans format, and easily parsable for programmers format. aka why I use json.

If you need a more compressed format, make a build pipeline.

# I have decided to not use Bevy as a tile editor

Probably should have known, ECS probably isn't that good for a gui application, but the main reason is: I want to be able to put some pixels on the screen. I think the next thing I'll do is see what window tools rust has, if all else fails, switch to raylib lol.

the todo's with bevy always seem to get bigger.

## Quick Start

Probably not so quick, gotta build bevy in release

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

## How to add more images

simply put a folder full of images in the assets folder. (warning: has a rigid structure right now, so if theres a bad file it will crash.)

images should be 32 by 32 for scale reasons

## Why is this 1.0?

Because I want to train my version-ing skills. Also it technically works, The best kind of works.

## TODO's

### For small extensions:

- Be able to load images of different sizes
- Be able to change the size of the tiles in editor. (Zoom out with camera?)
- Make the editor state part of the saved grid json? (might just be for quick saves)
- Better save paths, (maybe keep the quick-save though)
- Add Load method that isn't quick-save
- Make saves pretty-printed, for readability
- Make a ui (if bevy ui uses css, im not using bevy ui)
- More marks at the edges of the grid to mark where a tile is.
- Make the pallet be separated by folders, and stop it from crashing when loading something bad.
- Make some toggles for the tinting on the sprites in the grid.

### Some time in the future:

- Maybe remove _json_ dependency at some point? (Bevy is already a lot)
- Add notes to tiles on the grid
- Add notes to the tiles themselves
- Add multiple layers (so a ground layer and an item layer)
- Add multiple floors
- Add extendible grid, aka add grids side by side you you could make a whole world
