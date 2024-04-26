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

## p2p

### Find Your Ipv6 Address

p2p is done with ipv6. If you don't have ipv6 then make a zmq bridge with some
server. Pull reqs accepted. Find me narodnik in #math on Libera.

Find your ipv6 address with `ip a`.

```
...
2: eno1: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP group default qlen 1000
    ...
    inet6 65eb:56ea:aa32:9800:3400:1324:6565:89fe/64 scope global dynamic mngtmpaddr noprefixroute 
    ..
```

This is the line with 'global', which means your globalist ipv6 addr.
Remove the bit at the end which says `/64`, and surround it with `[...]`.
So the string you want now will be:

```
[65eb:56ea:aa32:9800:3400:1324:6565:89fe]
```

Everyone who wants access to your canvas, should now get this address.
Share it with them.

### Ping a Canvas

You can skip this step, since it's more like troubleshooting. Before proceeding,
lets try to ping the address and make sure the node is up.

Open `client.py`, and edit the `peer = ...` line with the address.
Then simply run it using `./client.py`.

### Final Config for Drawing

In `pencil.py` (mouse) or `pencil_libinput.py` (for wacom tablets), in the top of the file, you will see:

```python
PEERS = [
    #"[XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX]",
]
```

Simply replace that with your address and uncomment it like this:

```python
PEERS = [
    "[65eb:56ea:aa32:9800:3400:1324:6565:89fe]",
]
```

You can add multiple of these lines fyi.

This needs to be done by all participants so they push changes to each other's
canvas.

