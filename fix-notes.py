#! /usr/bin/python3
import sys
import numpy as np

notes = list(np.loadtxt(sys.argv[1], skiprows=1, delimiter=','))

def clean(notes):
    fixed = []
    for i in range(len(notes) - 1):
        curr = notes[i]
        if curr[2] == 0:
            continue
        next = notes[i+1]
        j = 0
        while next[1] - curr[1] - curr[2] < 2 and (next[0] - curr[0]) < 5: # silence is too short
            curr[2] = next[1] + next[2] - curr[1]
            next[2] = 0
            j += 1
            next = notes[i + j]
        fixed.append(curr)
    return fixed
fixed = clean(notes)
# print(len(notes))
# print(len(fixed))

fixed = np.array(fixed)
np.savetxt(sys.argv[1], fixed, delimiter=",", header="freq,start,dur")