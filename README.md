# Ingot

*the world's worst Photoshop clone*

Ingot is a GTK-based image filtering program — but that's not really its primary
purpose.  The main reason I made Ingot was because I wanted a program that makes
it easy to write image filters and run them interactively.  I've written
command-line apps in the past designed solely to read image A, apply a filter to
it, and write it to image B, and they're always very hard to work with because
half of the parameters are hard-coded, they often only use one thread, and you
can't see what they're doing — which means you won't know if you made a mistake
until they're done.  Ingot fixes all of that — it features a tile-based renderer
backed by a thread pool to maximize CPU utilization, interactive parameters so
you can tweak your filter's settings on the fly, and a live preview that lets
you view the result as it's being rendered.

## Usage

Ingot should be self-contained within the `ingot` binary *[citation needed]*.
The interface itself is fairly simple — select the Open button on the left side
of the header to open an image, choose a filter from the dropdown on the top
right, tweak settings listed in the panel to the right of the image, and then
save your creation with the Save button on the right side of the header.

## Writing a filter

// TODO: finish this part once the RenderProc and Filter traits are complete

## Why is it called Ingot?

That's an excellent question.  ~~Ingot is a name that I came up with after
considerable deliberation because I feel it deeply, truly represents the intent
and motivation of this corpus of code~~ I came up with it in about five minutes
in my stats class.