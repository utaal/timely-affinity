# timely-affinity
Affinity-aware `execute` for timely dataflow, in linux.

Currently requires a forked `timely_communication`, in your `Cargo.toml`:

```
timely_communication = { git = "https://github.com/utaal/timely-dataflow", branch = "thread-spawn-hook" }
timely = { git = "https://github.com/utaal/timely-dataflow", branch = "thread-spawn-hook" }
timely_affinity = { git = "https://github.com/utaal/timely-affinity" }
```

For a process, given the process' affinity:

* the `w` worker threads will be pinned each to one of the first `w` available cores,
* the comm threads will have an affinity mask spanning all the remaining available cores.

You can affect this behaviour by setting the process' affinity, so in a dual-socket numa machine (with 4 physical cores on each node):

```
# bind the first process to the first node (cpus 0-3)
hwloc-bind node:0 -- ./target/debug/examples/hello -n 2 -p 0 -w 2
# bind the second process to the second node (cpus 4-7)
hwloc-bind node:1 -- ./target/debug/examples/hello -n 2 -p 1 -w 2
```

Worker 0 and 1 end up on cpus 0 and 1, respectively. Worker 2 and 3 (second process) end up on cpus 4 and 5, respectively.
The first process' comm threads share cores 2 and 3, and the second process' comm threads share cores 6 and 7.
