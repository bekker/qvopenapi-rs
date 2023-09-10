#!/bin/bash

# Create dummy window manager
Xvfb :99 &
export DISPLAY=:99
export LANG=ko_KR.UTF-8
/usr/bin/xfce4-session &
wine /namu/bin/qvopenapi-http.exe
