#!/bin/bash
set -e

export DISPLAY=:99
export LANG=ko_KR.UTF-8

# Clean up stale lock files from previous runs (e.g. container restart)
rm -f /tmp/.X99-lock

# Start Xvfb
Xvfb :99 -screen 0 1024x768x24 &
XVFB_PID=$!

# Wait for the X server socket to appear (up to 30 seconds)
echo "Waiting for Xvfb to be ready..."
for i in $(seq 1 30); do
    if [ -e /tmp/.X11-unix/X99 ]; then
        echo "Xvfb is ready (took ${i}s)"
        break
    fi
    # Also check Xvfb hasn't crashed
    if ! kill -0 "$XVFB_PID" 2>/dev/null; then
        echo "ERROR: Xvfb process died unexpectedly" >&2
        exit 1
    fi
    sleep 1
done

# Final check: did we actually get the socket?
if [ ! -e /tmp/.X11-unix/X99 ]; then
    echo "ERROR: Xvfb failed to start within 30 seconds" >&2
    exit 1
fi

# Now start the window manager
/usr/bin/xfce4-session &

# Give the session a moment to initialize
sleep 2

# Start the application
exec wine /namu/bin/qvopenapi-http.exe
