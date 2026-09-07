// Trace shrinking via delta debugging (ddmin).
//
// Finding a seed that breaks an invariant is only half the job; a 40-fault schedule
// that triggers a bug is nearly useless to a human. ddmin repeatedly throws away
// subsets of the faults and keeps whichever smaller schedule still fails, converging
// on a minimal reproducer. This is the classic Zeller/Hildebrandt algorithm.

using System;
using System.Collections.Generic;

namespace SeedSim;

public static class Shrink
{
    private static List<List<T>> Split<T>(IReadOnlyList<T> xs, int n)
    {
        var k = xs.Count / n;
        var r = xs.Count % n;
        var outChunks = new List<List<T>>();
        var i = 0;
        for (var idx = 0; idx < n; idx++)
        {
            var size = k + (idx < r ? 1 : 0);
            if (size > 0)
            {
                var chunk = new List<T>();
                for (var j = 0; j < size; j++) chunk.Add(xs[i + j]);
                outChunks.Add(chunk);
            }
            i += size;
        }
        return outChunks;
    }

    /// <summary>
    /// Return a minimal sublist of <paramref name="items"/> for which
    /// <paramref name="stillFails"/> holds. <c>stillFails(items)</c> must be true to
    /// begin with - you only shrink a known failure.
    /// </summary>
    public static List<T> Ddmin<T>(IReadOnlyList<T> items, Func<List<T>, bool> stillFails)
    {
        var cur = new List<T>(items);
        if (!stillFails(cur))
            throw new ArgumentException("ddmin requires the full input to already fail");

        var n = 2;
        while (cur.Count >= 2)
        {
            var chunks = Split(cur, n);
            var reduced = false;
            for (var i = 0; i < chunks.Count; i++)
            {
                var complement = new List<T>();
                for (var j = 0; j < chunks.Count; j++)
                    if (j != i) complement.AddRange(chunks[j]);
                if (complement.Count > 0 && stillFails(complement))
                {
                    cur = complement;
                    n = Math.Max(n - 1, 2);
                    reduced = true;
                    break;
                }
            }
            if (!reduced)
            {
                if (n >= cur.Count) break;
                n = Math.Min(cur.Count, 2 * n);
            }
        }
        return cur;
    }
}
