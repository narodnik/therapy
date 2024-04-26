#!/usr/bin/python
# Ping a peer
from pytherapy import ReqApi, Event, Notifier

peer = "[XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX]"
api = ReqApi(peer)
print(api.hello())

