# Therapy

Minimal p2p collaborative whiteboard canvas tool.

I tried every crappy bloated whiteboard app for Linux and none of them worked
with my Wacom tablet. They all look ugly and require you to run a centralized
server (which is slow).

I just want to have a rendering canvas in GL (which needs to be fast),
then connect myself to others using ZMQ pipes in Python.

## Philosophy

There is a binary responsible for rendering the world and you communicate
with it using ZMQ python.

Then you can run python scripts to enable or disable certain features.
These can be:

* Rendering Latex
* Smooth interpolated lines
* Syncing canvases over any preferred transport mechanism.

## Usage

Start the canvas:

```
cargo run
```

Leave this running.

Then in another terminal run any of these python scripts:

* `keyb_nav.py` - keyboard navigation using the arrow keys and zooming in and
  out with the mouse wheel.
* `linetool.py` - click once to start a line, click again to draw it.
* `pencil.py` - click and drag to draw lines. It's the pencil tool.

If you want to use a wacom, you need `libinput-dev` installed, then run:

```
cd python-libbinput/
make
cd ../
./pencil_libinput.py
```

