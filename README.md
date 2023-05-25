# X

Run this command:
```
python threaded_therapy.py -f tcp://systemd.rehab:5559 -b tcp://systemd.rehab:5560 -p NICK
```

Where NICK is a nickname you choose.

# Wayland/libinput

Use this command to test your libinput setup:
```
libinput debug-tablet
```

Usage is the same as above, except we append the flag `-i`.
```
python threaded_therapy.py -f tcp://systemd.rehab:5559 -b tcp://systemd.rehab:5560 -p NICK -i
```
