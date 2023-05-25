# X

Run this command:
```
python threaded_therapy.py -f tcp://SERVER:5559 -b tcp://SERVER:5560 -p NICK
```

Where NICK is a nickname you choose, and SERVER is the server address.

# Wayland/libinput

Use this command to test your libinput setup:
```
libinput debug-tablet
```

Usage is the same as above, except we append the flag `-i`.
```
python threaded_therapy.py -f tcp://SERVER:5559 -b tcp://SERVER:5560 -p NICK -i
```
