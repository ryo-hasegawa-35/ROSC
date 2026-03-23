# Benchmark Fixture Inventory And Reproducibility Plan

## Purpose

This document freezes the benchmark input classes and reproducibility rules that
should exist before any performance claim is made.

## Fixture Root

The repository fixture root is:

- `fixtures/benchmarks/`

Primary inventory files:

- `fixtures/benchmarks/workload-catalog.json`
- `fixtures/benchmarks/context-template.json`

## Workload Classes

The initial benchmark catalog should include:

- control-plane message bursts
- dense sensor streams
- mixed installation traffic
- malformed pressure traffic
- recovery and late-joiner traffic
- adapter bridge traffic

## Reproducibility Rules

Every future benchmark run must record:

- git revision
- build profile
- operating system and hardware notes
- workload ID
- packet-size assumptions
- destination count
- timing source and duration

## Interpretation Rule

Benchmarks should answer one named question at a time, such as:

- throughput ceiling
- p99 queue latency
- overload containment behavior
- recovery overhead

They should not collapse all concerns into one headline number.

## Initial Inventory

The first workload inventory is stored in:

- `fixtures/benchmarks/workload-catalog.json`

The required metadata template is stored in:

- `fixtures/benchmarks/context-template.json`
