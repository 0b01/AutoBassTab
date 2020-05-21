#! /usr/bin/python3

import sys
import numpy as np
import matplotlib.pyplot as plt
from scipy import stats
from scipy.io import wavfile
from numpy.lib.stride_tricks import as_strided
from PIL import Image, ImageDraw, ImageFont


dir = sys.argv[1]
notes = np.loadtxt(dir+sys.argv[2], skiprows=1, delimiter=',')
arr = np.loadtxt(dir+sys.argv[3], skiprows=1, delimiter=',')

im = Image.open(dir+"octave.activation.png")
width, height = im.size

# draw boxes
draw = ImageDraw.Draw(im)

for n in notes:
    freq, start, dur = np.array(n)
    f = np.log(freq) * (-91) + 682
    draw.rectangle([(int(start), f - 5), (int(start + dur), f + 5)])
del draw

#------------------

# draw tablature

h = 100
padding = 10
f_im = Image.new(size=(width, h), mode="RGB", color=(255,255,255,0))
draw = ImageDraw.Draw(f_im)
strings = [i + h / 8 for i in range(0, h, h//4)][::-1]
for s in strings:
    draw.line(((0, s), (width, s)), fill=(0,0,0,0))



font = ImageFont.truetype("arial.ttf", 14)
for start, (string, fret) in zip(notes[:, 1], arr):
    draw.text((start, strings[int(string)]-8), str(int(fret)), fill=(0,0,0,0), font=font)

del draw
# f_im.save(dir+"arrangement.png")

im.paste(f_im)
im.crop((0, 0, 1000, h))
im.save(dir+"final.png")