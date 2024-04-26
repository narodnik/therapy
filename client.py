#!/usr/bin/python
# Ping a peer
from pytherapy import ReqApi, Event, Notifier

# localhost
peer = "[::1]"
# Edit this for a remote host
#peer = "[XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX]"

api = ReqApi(peer)
print(api.hello())

