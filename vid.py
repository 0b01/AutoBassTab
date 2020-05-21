import eyed3
from sys import argv
import io
import numpy as np
from PIL import Image, ImageDraw
import cv2


song = eyed3.load(argv[1])
art = song.tag.images.get("Album Art").image_data
art = Image.open(io.BytesIO(art))

videodims = (640, 380)
fourcc = cv2.VideoWriter_fourcc(*"mp4v")
video = cv2.VideoWriter(argv[3],fourcc, 100, videodims)
img = Image.open(argv[2])
(width, height) = img.size

art = art.resize((320, 320))

for i in range(0, width):
    imtemp = img.crop((-320+i, 0, 320+i, height))
    if i < 640:
        imtemp.paste(art, (320-art.size[0]-i, (height - 320)//2))
    video.write(cv2.cvtColor(np.array(imtemp), cv2.COLOR_RGB2BGR))
video.release()