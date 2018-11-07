extern crate timely;
extern crate timely_numa;
// extern crate hwloc;
extern crate libc;

use timely::Configuration;
use timely::dataflow::{InputHandle, ProbeHandle};
use timely::dataflow::operators::{Input, Exchange, Inspect, Probe};

// use hwloc::{ObjectType, Topology, CpuBindFlags, CPUBIND_PROCESS};

fn main() {

    let mut config = Configuration::from_args(std::env::args()).unwrap();

    let mut core_ids = unsafe {
        // let pid = ::libc::getpid();
        let mut cpuset = ::std::mem::zeroed::<::libc::cpu_set_t>();
        ::libc::sched_getaffinity(0, ::std::mem::size_of::<::libc::cpu_set_t>(), &mut cpuset);
        let mut core_ids = Vec::new();
        for i in 0..::libc::CPU_SETSIZE as usize {
            if ::libc::CPU_ISSET(i, &cpuset) {
                core_ids.push(i);
            }
        }
        eprintln!("{:?}", core_ids);
        core_ids
    };

    match config {
        Configuration::Cluster { threads, ref mut spawn_fn, .. } => {
            eprintln!("threads = {}", threads);
            let comm_core_ids = core_ids.split_off(threads);
            unsafe {
                let mut cpuset = ::std::mem::zeroed::<::libc::cpu_set_t>();
                eprintln!("comm core ids = {}", threads);
                for id in comm_core_ids.into_iter() {
                    ::libc::CPU_SET(id, &mut cpuset);
                }
                ::libc::sched_setaffinity(0, ::std::mem::size_of::<::libc::cpu_set_t>(), &mut cpuset);
            }
        },
        _ => (),
    }

    // let topo = Topology::new();
    // for i in 0..topo.depth() {
    //     println!("*** Objects at level {}", i);
    //     for (idx, object) in topo.objects_at_depth(i).iter().enumerate() {
    //         println!("{}: {}", idx, object);
    //     }
    // }

    // eprintln!("{}", topo.size_at_depth(topo.depth_for_type(&ObjectType::NUMANode).unwrap()));
    // eprintln!("{:?}", topo.get_cpubind(CPUBIND_PROCESS));


    // initializes and runs a timely dataflow.
    // timely::execute_from_args(std::env::args(), |worker| {

    //     let index = worker.index();
    //     let mut input = InputHandle::new();
    //     let mut probe = ProbeHandle::new();

    //     // create a new input, exchange data, and inspect its output
    //     worker.dataflow(|scope| {
    //         scope.input_from(&mut input)
    //              .exchange(|x| *x)
    //              .inspect(move |x| println!("worker {}:\thello {}", index, x))
    //              .probe_with(&mut probe);
    //     });

    //     // introduce data and watch!
    //     for round in 0..10 {
    //         if index == 0 {
    //             input.send(round);
    //         }
    //         input.advance_to(round + 1);
    //         while probe.less_than(input.time()) {
    //             worker.step();
    //         }
    //     }
    // }).unwrap();
}
