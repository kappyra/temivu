"""Local demo bridge: commands over stdin, RGBA frames and status over stdout."""
import math
import os
from pathlib import Path
import select
import struct
import sys
import time

import gi

gi.require_version("Gst", "1.0")
gi.require_version("GstVideo", "1.0")
from gi.repository import Gst, GstVideo

Gst.init(None)
player = Gst.ElementFactory.make("playbin")
if player is None:
    raise RuntimeError("GStreamer playbin is unavailable")
sink_bin = Gst.parse_bin_from_description(
    "videoconvert ! videoscale ! video/x-raw,format=RGBA,"
    "width=[1,720],height=[1,720],pixel-aspect-ratio=1/1 ! "
    "appsink name=frames max-buffers=1 drop=true wait-on-eos=false", True
)
sink = sink_bin.get_by_name("frames")
player.set_property("video-sink", sink_bin)
player.set_property("uri", Path(sys.argv[1]).resolve(strict=True).as_uri())
player.set_property("volume", 0.75)
bus = player.get_bus()
player.set_state(Gst.State.PAUSED)

output = sys.stdout.buffer
rate = 1.0
playing = False
pending = b""
last_status = 0.0


def clock(query):
    ok, ticks = query(Gst.Format.TIME)
    return max(0, ticks) / Gst.SECOND if ok else 0.0


def packet(width=0, height=0, status=0, pixels=b""):
    output.write(struct.pack(">IIddB", width, height, clock(player.query_position),
                             clock(player.query_duration), status))
    output.write(pixels)
    output.flush()


def command(line):
    global rate, playing
    try:
        new_playing, volume, speed, seek = map(float, line.split())
    except ValueError:
        return
    if not all(map(math.isfinite, (new_playing, volume, speed, seek))):
        return
    player.set_property("volume", min(1.0, max(0.0, volume)))
    speed = min(2.0, max(0.25, speed))
    if seek >= 0.0 or speed != rate:
        position = seek if seek >= 0.0 else clock(player.query_position)
        duration = clock(player.query_duration)
        if duration > 0.0:
            position = min(position, duration)
        if player.seek(speed, Gst.Format.TIME, Gst.SeekFlags.FLUSH | Gst.SeekFlags.ACCURATE,
                       Gst.SeekType.SET, int(position * Gst.SECOND), Gst.SeekType.NONE, -1):
            rate = speed
    requested = bool(new_playing)
    if requested != playing:
        playing = requested
        player.set_state(Gst.State.PLAYING if playing else Gst.State.PAUSED)


try:
    while True:
        if select.select([sys.stdin], [], [], 0.008 if playing else 0.08)[0]:
            chunk = os.read(sys.stdin.fileno(), 4096)
            if not chunk:
                break
            pending += chunk
            while b"\n" in pending:
                line, pending = pending.split(b"\n", 1)
                command(line)
            if len(pending) > 4096:
                break
        message = bus.pop_filtered(Gst.MessageType.ERROR | Gst.MessageType.EOS)
        if message and message.type == Gst.MessageType.ERROR:
            error, _ = message.parse_error()
            print(f"Temivu video: {error.message}", file=sys.stderr)
            packet(status=2)
            break
        if message and message.type == Gst.MessageType.EOS:
            playing = False
            player.set_state(Gst.State.PAUSED)
            packet(status=1)

        sample = sink.emit("try-pull-sample" if playing else "try-pull-preroll", 0)
        if sample:
            info = GstVideo.VideoInfo.new_from_caps(sample.get_caps())
            buffer = sample.get_buffer()
            ok, mapping = buffer.map(Gst.MapFlags.READ)
            if ok:
                try:
                    width, height = info.width, info.height
                    stride, offset = info.stride[0], info.offset[0]
                    row = width * 4
                    if (0 < width <= 720 and 0 < height <= 720 and stride >= row
                            and 0 <= offset <= len(mapping.data) - (height - 1) * stride - row):
                        view = memoryview(mapping.data)
                        if stride == row:
                            pixels = view[offset:offset + height * row]
                        else:
                            pixels = b"".join(view[offset + y * stride:offset + y * stride + row]
                                              for y in range(height))
                        packet(width, height, pixels=pixels)
                finally:
                    buffer.unmap(mapping)
            last_status = time.monotonic()
        elif playing and time.monotonic() - last_status >= 0.25:
            packet()
            last_status = time.monotonic()
except BrokenPipeError:
    pass
finally:
    player.set_state(Gst.State.NULL)
