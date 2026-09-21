import { describe, expect, it } from "vitest";

import { Crash, FaultSchedule } from "./faults.js";
import { Node, Simulator, type Msg } from "./sim.js";

class Recorder extends Node {
  fired: string[] = [];

  onTimer(name: string, payload: unknown): void {
    this.fired.push(`${name}:${String(payload)}`);
  }
}

class Echo extends Node {
  received: Array<[string, Msg]> = [];

  onMessage(src: string, msg: Msg): void {
    this.received.push([src, msg]);
  }
}

describe("simulator", () => {
  it("fires timers and records them", () => {
    const sim = new Simulator(1);
    const n = new Recorder("n1");
    sim.add(n);
    n.timer(5, "tick", 42);
    sim.run();

    expect(n.fired).toEqual(["tick:42"]);
    expect(sim.trace).toEqual([
      { time: 5, event: "timer", fields: "name=tick,node=n1" },
    ]);
  });

  it("swallows timers on a crashed node", () => {
    const sim = new Simulator(1, new FaultSchedule([], [new Crash("n1", 0, 100)]));
    const n = new Recorder("n1");
    sim.add(n);
    n.timer(5, "tick", null);
    sim.run();

    expect(n.fired).toEqual([]);
    expect(sim.trace).toEqual([]);
  });

  it("fires events in time then insertion order", () => {
    const sim = new Simulator(1);
    const order: number[] = [];
    sim.at(5, () => order.push(1));
    sim.at(1, () => order.push(2));
    sim.at(5, () => order.push(3));
    sim.run();
    expect(order).toEqual([2, 1, 3]);
  });

  it("stops at the until horizon", () => {
    const sim = new Simulator(1);
    let count = 0;
    sim.at(5, () => (count += 1));
    sim.at(50, () => (count += 1));
    sim.run(10);
    expect(count).toBe(1);
  });

  it("throws on a negative delay", () => {
    const sim = new Simulator(1);
    expect(() => sim.at(-1, () => {})).toThrow();
  });

  it("delivers messages through the network", () => {
    const sim = new Simulator(1);
    const a = new Echo("a");
    const b = new Echo("b");
    sim.add(a);
    sim.add(b);
    sim.at(0, () => a.send("b", { kind: "ping", value: 7 }));
    sim.run();

    expect(b.received.length).toBe(1);
    expect(b.received[0][0]).toBe("a");
    expect(b.received[0][1]).toEqual({ kind: "ping", value: 7 });
  });
});
