package com.seedsim;

/** node is down for [start, end): it sends nothing and receives nothing. */
public record Crash(String node, int start, int end) implements Fault {}
