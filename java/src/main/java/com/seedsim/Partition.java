package com.seedsim;

/** The link between two nodes is down for [start, end). */
public record Partition(String a, String b, int start, int end) implements Fault {}
