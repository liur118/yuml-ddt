#!/usr/bin/env python3
import struct
import zlib

def create_png(width, height, color, filename):
    """Create a simple solid-color RGBA PNG"""
    def png_chunk(chunk_type, data):
        chunk_len = len(data)
        chunk = struct.pack('>I', chunk_len) + chunk_type + data
        crc = zlib.crc32(chunk_type + data) & 0xffffffff
        return chunk + struct.pack('>I', crc)
    
    # PNG signature
    signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk - color type 6 for RGBA
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr = png_chunk(b'IHDR', ihdr_data)
    
    # Image data - solid color with alpha (RGBA)
    raw_data = b''
    for y in range(height):
        raw_data += b'\x00'  # filter byte
        for x in range(width):
            raw_data += bytes(color[:4])  # RGBA
    
    compressed = zlib.compress(raw_data)
    idat = png_chunk(b'IDAT', compressed)
    
    # IEND chunk
    iend = png_chunk(b'IEND', b'')
    
    with open(filename, 'wb') as f:
        f.write(signature + ihdr + idat + iend)
    print(f'Created {filename}')

# Create icons with a nice blue color (RGBA)
color = (41, 128, 185, 255)  # Nice blue with full opacity
create_png(32, 32, color, '32x32.png')
create_png(32, 32, color, 'icon.png')
create_png(128, 128, color, '128x128.png')
create_png(256, 256, color, '128x128@2x.png')
print('All icons created!')
