"""Run one tool command under a Python audit hook that records every file open, network connection and subprocess.
usage: audit_run.py LOG.json [--leaky DECOY_FILE] -- <mtgtools args...>"""
import json, os, sys
log = sys.argv[1]; args = sys.argv[sys.argv.index("--") + 1:]
leaky = sys.argv[sys.argv.index("--leaky") + 1] if "--leaky" in sys.argv else None
events = []
def hook(ev, a):
    if ev == "open" and isinstance(a[0], (str, bytes)) and os.path.realpath(a[0] if isinstance(a[0], str) else a[0].decode()) != os.path.realpath(log): events.append(["open", os.path.realpath(a[0] if isinstance(a[0], str) else a[0].decode()), str(a[1])])
    elif ev in ("socket.connect", "socket.getaddrinfo", "subprocess.Popen", "os.system", "os.exec", "os.posix_spawn"): events.append([ev, repr(a)[:200]])
sys.addaudithook(hook)
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
try:
    if leaky: open(leaky).read()            # negative control: a tool that peeks at the engine's files
    from mtgtools.cli import main
    rc = main(args)
finally:
    json.dump(events, open(log, "w"))
