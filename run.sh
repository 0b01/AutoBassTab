#! /bin/bash

dir=output/
in=$dir/bass.wav
out=$dir/octave.wav

cp $1 $dir/music.mp3

echo "-------------------------------------------------:"
echo "-----------Separating stems:"
spleeter separate -i $dir/music.mp3 -p spleeter:4stems-16kHz -o $dir -f "{instrument}.wav"
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
crepe $dir/octave.wav --model-capacity tiny -p -v --apply-voicing
mv octave.f0.csv $dir
mv octave.activation.png $dir
echo "-----------Done!"
echo ""
echo ""


echo "-------------------------------------------------:"
echo "----------Tracking notes:"
./notes/notes $dir/octave.f0.csv $dir/notes.csv
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
echo "----------Adding audio:"
ffmpeg -i $dir/vid.mp4 -i $dir/music.mp3 -c copy -map 0:v:0 -map 1:a:0 $dir/full.mp4
ffmpeg -i $dir/vid.mp4 -i $dir/bass.wav -c:v copy -c:a aac $dir/bassonly.mp4
echo "-----------Done!"
echo ""
echo ""
