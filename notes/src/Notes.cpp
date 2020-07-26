#include "MonoNote.h"
#include <fstream>
#include <iostream>
#include <algorithm>
#include <sstream>

using namespace std;

struct Feature {
    float value;
    float start;
    float dur;
};

class Notes {
public:
    Notes(vector<float> level, vector<float> pitch):
        m_channels(0),
        m_stepSize(256),
        m_blockSize(2048),
        m_fmin(40),
        m_fmax(1600),
        m_oF0Candidates(0),
        m_oF0Probs(0),
        m_oVoicedProb(0),
        m_oCandidateSalience(0),
        m_oSmoothedPitchTrack(0),
        m_oNotes(0),
        m_threshDistr(2.0f),
        m_fixedLag(1.0f),
        m_outputUnvoiced(0.0f),
        m_preciseTime(0.0f),
        m_lowAmp(0.1f),
        m_onsetSensitivity(0.1f),
        m_pruneThresh(0.1f),
        m_inputSampleRate(44100),
        m_pitchProb(0),
        m_timestamp(0),
        m_level(level),
        m_pitchTrack(pitch)
    {};

    void process() {
        std::vector<std::vector<std::pair<double, double> > > smoothedPitch;
        for (size_t iFrame = 0; iFrame < m_pitchTrack.size(); ++iFrame) {
            std::vector<std::pair<double, double> > temp;
            if (m_pitchTrack[iFrame] > 0)
            {
                double tempPitch = 12 *
                    std::log(m_pitchTrack[iFrame]/440)/std::log(2.) + 69;
                temp.push_back(std::pair<double,double>(tempPitch, .9));
            }
            smoothedPitch.push_back(temp);
        }

        // In fixed-lag mode, we use fixed-lag processing for the note
        // transitions here as well as for the pitch transitions in
        // process. The main reason we provide the fixed-lag option is so
        // that we can get pitch results incrementally from process; we
        // don't get that outcome here, but we do benefit from its bounded
        // memory usage, which can be quite a big deal. So if the caller
        // asked for it there, we use it here too. (It is a bit slower,
        // but not much.)

        MonoNote mn(m_fixedLag > 0.5f);
        vector<MonoNote::FrameOutput> mnOut = mn.process(smoothedPitch);

        // ofstream pitches("pitches.csv");

        std::cerr << "mnOut size: " << mnOut.size() << std::endl;
        std::cerr << "m_pitchTrack size: " << m_pitchTrack.size() << std::endl;

        // turning feature into a note feature
        Feature f;

        int onsetFrame = 0;
        bool isVoiced = 0;
        bool oldIsVoiced = 0;
        size_t nFrame = m_pitchTrack.size();

        float minNoteFrames = 4;

        // the body of the loop below should be in a function/method
        // but what does it actually do??
        // * takes the result of the note tracking HMM
        // * collects contiguously pitched pitches
        // * writes a note once it notices the voiced segment has ended
        // complications:
        // * it needs a lookahead of two frames for m_level (wtf was I thinking)
        // * it needs to know the timestamp (which can be guessed from the frame no)
        // *
        std::vector<float> notePitchTrack; // collects pitches for 1 note at a time
        for (size_t iFrame = 0; iFrame < nFrame; ++iFrame)
        {


            isVoiced = mnOut[iFrame].noteState < 3
                && smoothedPitch[iFrame].size() > 0
                && mnOut[iFrame].pitch > 3
                && (iFrame >= nFrame-2
                    || ((m_level[iFrame]/m_level[iFrame+2]) > m_onsetSensitivity));

            // pitches << mnOut[iFrame].pitch << "," << mnOut[iFrame].noteState << "," << isVoiced << endl;

            if (isVoiced && iFrame != nFrame-1)
            {
                if (!oldIsVoiced) // beginning of a note
                {
                    onsetFrame = iFrame;
                }
                float pitch = smoothedPitch[iFrame][0].first;
                notePitchTrack.push_back(pitch); // add to the note's pitch m_pitchTrack
            } else { // not currently voiced
                if (oldIsVoiced) // end of note
                {
                    if (notePitchTrack.size() >= minNoteFrames)
                    {
                        std::sort(notePitchTrack.begin(), notePitchTrack.end());
                        float medianPitch = notePitchTrack[notePitchTrack.size()/2];
                        float medianFreq =
                            std::pow(2,(medianPitch - 69) / 12) * 440;
                        f.value = medianFreq;
                        float start = onsetFrame;
                        float end   = iFrame;
                        f.start = start;
                        f.dur = end - start;
                        _notes.push_back(f);
                    }
                    notePitchTrack.clear();
                }
            }
            oldIsVoiced = isVoiced;
        }
    }
public:
    vector<Feature> _notes;
private:
    size_t m_channels;
    size_t m_stepSize;
    size_t m_blockSize;
    float m_fmin;
    float m_fmax;

    mutable int m_oF0Candidates;
    mutable int m_oF0Probs;
    mutable int m_oVoicedProb;
    mutable int m_oCandidateSalience;
    mutable int m_oSmoothedPitchTrack;
    mutable int m_oNotes;

    float m_threshDistr;
    float m_fixedLag;
    float m_outputUnvoiced;
    float m_preciseTime;
    float m_lowAmp;
    float m_onsetSensitivity;
    float m_pruneThresh;
    float m_inputSampleRate;

    deque<vector<pair<double, double> > > m_pitchProb;
    deque<float> m_timestamp;
    vector<float> m_level;
    vector<float> m_pitchTrack;
};

extern "C" {
int get_notes(float out[], int size, float level[], float pitch[]) {
    vector<float> l(level, level + size);
    vector<float> p(pitch, pitch + size);
    Notes notes(l, p);
    notes.process();

    vector<float>* ret = new vector<float>();
    int i = 0;
    for (auto note : notes._notes) {
        out[i++] = note.value;
        out[i++] = note.start;
        out[i++] = note.dur;
    }
    return notes._notes.size() * 3;
}
}

int main(int argc, char **argv) {
    cout << "Usage: ./notes infile.f0.csv outfile.csv" << endl;
    ifstream data(argv[1]);
    string line;
    vector<float> time;
    vector<float> pitch;
    vector<float> level;

    getline(data,line); // skip title
    while(getline(data,line))
    {
        stringstream lineStream(line);
        string cell;

        getline(lineStream,cell,','); float t = stof(cell);
        getline(lineStream,cell,','); float f = stof(cell);
        getline(lineStream,cell,','); float c = stof(cell);
        getline(lineStream,cell,','); float l = stof(cell);

        time.push_back(t);
        pitch.push_back(c > 0.4 ? f : 0);
        level.push_back(l);
    }

    std::cout << time.size() << endl;
    std::cout << pitch.size() << endl;
    std::cout << "reading in data" << endl;

    Notes notes(level, pitch);
    notes.process();
    ofstream outfile(argv[2]);
    outfile <<"note,start,dur" << endl;
    for (auto note : notes._notes) {
        outfile << note.value << ","
                << note.start << ","
                << note.dur << endl;
    }

    float output[time.size() * 3];
    int ret_sz = get_notes(output, time.size(), level.data(), pitch.data());
    for (float* i = output; i < output + ret_sz; i++) {
        cout << *i << endl;
    }

}
