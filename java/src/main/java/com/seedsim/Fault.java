// Fault schedules: the injected adversity a run is subjected to.
//
// A fault is a plain, immutable value with a time window. The schedule is part of the
// reproducer (seed + faults fully determine a run), and because faults have value
// equality they can be fed straight into the shrinker as opaque items.

package com.seedsim;

/** A fault a run can be subjected to (partition or crash). */
public sealed interface Fault permits Partition, Crash {}
