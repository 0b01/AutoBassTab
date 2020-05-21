#! /usr/bin/python3
from pprint import pprint
import os
import sys
import eyed3
from goldfinch import validFileName as vfn
from gmusicapi import Mobileclient
import getpass
try:
  from urllib.request import urlretrieve, urlopen
except ImportError:
  from urllib import urlretrieve, urlopen


if len(sys.argv) == 1:
  print("usage: python gmusic-dl.py <email> <album id>")
  sys.exit()


def normalizePath(input):
  return vfn(input, space="keep", ascii=False, initCap=False).decode('utf-8').rstrip(".")


login = "rickylqhan@gmail.com"
password = "gvyewpgzfbdwasqm"
targetDir = os.getcwd() + "/music"
albumId = sys.argv[1]

eyed3.log.setLevel("ERROR")

api = Mobileclient(debug_logging=False)
api.login(login, password, Mobileclient.FROM_MAC_ADDRESS)

album = api.get_album_info(albumId)
dirName = normalizePath("%s - %s" % (album["artist"], album["name"]))
dirPath = targetDir + "/" + dirName

print("downloading to directory: " + dirPath)
if not os.path.exists(dirPath):
    os.makedirs(dirPath)

for song in album["tracks"]:
  image_url = song['albumArtRef'][0]["url"]
  url = api.get_stream_url(song_id=song["storeId"], quality="hi")
  fileName = normalizePath("%s. %s - %s.mp3" % (song["trackNumber"], song["artist"], song["title"]))
  filePath = dirPath + "/" + fileName
  print("downloading: " + fileName)
  urlretrieve(url, filePath)

  audio = eyed3.load(filePath)
  if audio.tag is None:
    audio.tag = eyed3.id3.Tag()
    audio.tag.file_info = eyed3.id3.FileInfo(filePath)
  audio.tag.artist = song["artist"]
  audio.tag.album = album["name"]
  audio.tag.album_artist = album["artist"]
  audio.tag.title = song["title"]
  audio.tag.track_num = song["trackNumber"]
  image_data = urlopen(image_url).read()
  audio.tag.images.set(3, image_data, "image/jpeg", u"Album Art")
  audio.tag.save()

print("done.")
