#! /bin/bash
dir=$(basename "${1%.*}")
dir=output/${dir// /.}
in="$dir/bass.wav"
out="$dir/octave.wav"

mkdir "output"
mkdir "$dir"
cp "$1" "$dir/music.mp3"

echo "-------------------------------------------------:"
echo "-----------Separating stems:"
spleeter separate -i "$dir/music.mp3" -p spleeter:4stems-16kHz -o $dir -f "{instrument}.wav"
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Changing octave:"
sox $in $out pitch 1200 bass -30 100 gain 10 # sox --plot gnuplot $in -n treble -60 800 > out.plot; gnuplot out.plot
echo "-----------Done!"
echo ""
echo ""

# bpm=`aubio tempo $drums`
# echo "BPM: $bpm"

echo "-------------------------------------------------:"
echo "----------Tracking melody:"
crepe $dir/octave.wav --model-capacity full -p -v --apply-voicing
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Tracking notes:"
./notes/notes $dir/octave.f0.csv $dir/notes.csv
./fix-notes.py $dir/notes.csv
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Solving fingering:"
cat $dir/notes.csv | ./fingering/target/release/fingering -12 > $dir/arrangement.csv
echo "-----------Done!"
echo ""
echo ""




echo "-------------------------------------------------:"
echo "----------Plotting:"
./plot.py $dir/ notes.csv arrangement.csv
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Making video:"
./vid.py $dir/music.mp3 $dir/final.png $dir/vid.mp4
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Adding audio to video:"
ffmpeg -i $dir/vid.mp4 -i $dir/music.mp3 -c copy -map 0:v:0 -map 1:a:0 $dir/full.mp4 -y
ffmpeg -i $dir/vid.mp4 -i $dir/bass.wav -c:v copy -c:a aac $dir/bassonly.mp4 -y

# sox $dir/music.mp3 $dir/nobass.mp3 bass -40 300
# ffmpeg -i $dir/vid.mp4 -i $dir/nobass.mp3 -c copy -map 0:v:0 -map 1:a:0 $dir/nobass.mp4

echo "-----------Done!"
echo ""
echo ""


# echo "-------------------------------------------------:"
# echo "----------Uploading to YouTube:"
artist=`eyeD3 $dir/music.mp3  | grep ' artist' | cut -b 15-`
# album=`eyeD3 $dir/music.mp3  | grep "album: " | cut -b 8-`
title=`eyeD3 $dir/music.mp3  | grep "title" | cut -b 8-`

mv $dir/full.mp4 "$artist - $title - Bass Tab.mp4"
mv $dir/bassonly.mp4 "$artist - $title - Bass Tab - Isolated Bass.mp4"

# python upload_video.py --title="$title($artist) Bass Tab" --file $dir/full.mp4 &
# python upload_video.py --title="$title($artist) Bass Tab (Isolated Bass)" --file $dir/bassonly.mp4 &
# echo "-----------Done!"
# echo ""
# echo ""
