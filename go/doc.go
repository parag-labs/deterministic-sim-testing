// Package deterministicsimtesting is a Go port of deterministic-sim-testing:
// deterministic simulation testing for distributed code.
//
// Run message-passing systems inside one deterministic thread, replay any run
// from a single 64-bit seed, and shrink a failing fault schedule down to a
// minimal reproducer. The whole engine - the splitmix64 PRNG, the discrete-event
// simulator, the fault model, and ddmin shrinking - behaves identically to the
// Python, C#, and Java ports.
package deterministicsimtesting

// Version mirrors the reference implementation's __version__.
const Version = "0.1.0"
