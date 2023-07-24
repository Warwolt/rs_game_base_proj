import signal
import subprocess
import sys

def signal_handler(sig, frame):
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)

subprocess.run(["cargo", "build"])

processes = [
    subprocess.Popen(["cargo", "watch", "-w", "game", "-x", "build -p game"]),
    subprocess.Popen(["cargo", "run"]),
]

while True:
    for p in processes:
        code = p.poll()
        if code != None:
            for p in processes:
                p.terminate()
            sys.exit(code)
