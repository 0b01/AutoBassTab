from pprint import pprint
from functools import lru_cache
import math

MIDI_A4 = 69
FREQ_A4 = 440.

def freq_to_midi(freq):
    result = 12 * (math.log2(freq) - math.log2(FREQ_A4)) + MIDI_A4
    return result

def freq_to_note(freq, sharp=True):
    midi_number = freq_to_midi(freq)
    num = midi_number - (MIDI_A4 - 4 * 12 - 9)
    note = (num + .5) % 12 - .5
    rnote = int(round(note))
    error = note - rnote
    octave = int(round((num - note) / 12.))
    if sharp:
        names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
    else:
        names = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"]
    return Note(names[rnote], octave)

class Note():
    pitch_classes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    def __init__(self, pname, oct):
        # pitch class
        if pname.upper() in Note.pitch_classes:
            self.pname = pname.upper()
        else:
            raise ValueError('Invalid pitch name')
        # octave
        self.oct = oct
    def __add__(self, step):
        num_chroma = len(Note.pitch_classes)
        note = Note(self.pname, self.oct)
        p_ind = Note.pitch_classes.index(self.pname)
        new_p_ind = (p_ind + step) % num_chroma
        note.pname = Note.pitch_classes[new_p_ind]
        oct_diff = int(step / 12)
        note.oct = self.oct + oct_diff
        if step >0:
            if new_p_ind >= 0 and new_p_ind < p_ind:
                note.oct += 1
        else:
            if new_p_ind > p_ind and new_p_ind < num_chroma:
                note.oct -= 1
        return note

strings = [ Note("E", 1), Note("A", 1), Note("D", 2), Note("G", 2) ]
neck = {(i,j): note + j for j in range(21) for i,note in enumerate(strings)} # fret -> note
remove_open_strings = False
if remove_open_strings:
    del neck[(0,0)]
    del neck[(1,0)]
    del neck[(2,0)]
    del neck[(3,0)]
pos = {} # note -> frets
for k, v in neck.items():
    pos[v] = pos.get(v, [])
    pos[v].append(k)

@lru_cache(maxsize=None)
def biomechanical_cost(a, b):
    ai, aj = a
    bi, bj = b
    c = abs(aj - bj) + abs(ai-bi) * 0.3
    if aj == 0 or bj == 0:
        c += 10
    return c

assert(biomechanical_cost((0, 10), (0, 4)) == 6)
assert(biomechanical_cost((0, 10), (1, 4)) == 6.3)

@lru_cache(maxsize=None)
def arrange2(n1, n2, left=None, right=None):
    curr_min = 1000000000
    temp = None
    if not left:
        left = pos.get(n1, [(0,0)])
    if not right:
        right = pos.get(n2, [(0,0)])
    for pa in left:
        for pb in right:
            c = biomechanical_cost(pa, pb)
            if c < curr_min:
                curr_min = c
                temp = (pa, pb)
    return curr_min, temp

def arrange(notes):
    @lru_cache(maxsize=None)
    def helper(i, j, left=None, right=None, depth=0):
        curr_min = 100000000000
        arr = None
        # print(depth*"\t"+"- ",i,j, left, right)
        if i >= j:
            return 0, []
        for k in range(i, j):
            if left and k == i:
                c_curr, (l, r) = arrange2(notes[k], notes[k+1], left)
            elif right and k == j:
                c_curr, (l, r) = arrange2(notes[k], notes[k+1], right)
            else:
                c_curr, (l, r) = arrange2(notes[k], notes[k+1])
            # print(depth*"\t", k, (neck[l],neck[r]))
            c1, s1 = helper(i, k, right=(l,), depth=depth+1)
            c2, s2 = helper(k+1, j, left=(r,), depth=depth+1)
            c = c_curr + c1 + c2
            if c < curr_min:
                curr_min = c
                arr = s1 + [(l,r)] + s2
        return curr_min, arr
    return helper(0, len(notes)-1, depth=0)

def fingering_arrangement(notes):
    score = [freq_to_note(note) for note in list(notes)]
    n = 50
    ss = [score[i:i+n] for i in range(0, len(score), n)]
    arrs = []
    for s in ss:
        # print(score)
        # s = score[:50]
        cost, arr = arrange(s)
        arr = [i[0] for i in arr[:len(arr)]] + [arr[-1][1]]
        ns = [neck[i] for i in arr]
        # assert(s == ns)
        # print(cost, len(arr), arr)
        arrs += arr
    return arrs


if __name__ == "__main__":
    import numpy as np
    notes = np.loadtxt("./notes.csv", skiprows=1, delimiter=',')
    notes = notes[:, 0]
    print(fingering_arrangement(notes))
