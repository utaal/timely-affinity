use initialize::Configuration;

pub fn core_ids() -> Vec<usize> {
    unsafe {
        let mut cpuset = ::std::mem::zeroed::<::libc::cpu_set_t>();
        ::libc::sched_getaffinity(0, ::std::mem::size_of::<::libc::cpu_set_t>(), &mut cpuset);
        let mut core_ids = Vec::new();
        for i in 0..::libc::CPU_SETSIZE as usize {
            if ::libc::CPU_ISSET(i, &cpuset) {
                core_ids.push(i);
            }
        }
        core_ids
    }
}

pub fn worker_core_ids(config: &mut Configuration, mut core_ids: Vec<usize>) -> std::sync::Arc<Vec<usize>> {

    ::std::sync::Arc::new(match config {
        Configuration::Cluster { ref threads, ref process, ref addresses, ref mut spawn_fn, .. } => {
            let comm_core_ids = ::std::sync::Arc::new(core_ids.split_off(*threads));
            let comm_threads = (addresses.len() - 1) * 2;

            // if there are enough available cores, assign each comm thread to its own core
            if comm_core_ids.len() >= comm_threads {
                use std::sync::{Arc, Mutex};
                use std::ops::DerefMut;

                let mut comm_core_ids = comm_core_ids.iter();
                let mut send_core_ids = comm_core_ids.by_ref().take(comm_threads / 2).cloned().map(|x| Some(x)).collect::<Vec<Option<usize>>>();
                let mut recv_core_ids = comm_core_ids.by_ref().take(comm_threads / 2).cloned().map(|x| Some(x)).collect::<Vec<Option<usize>>>();
                send_core_ids.insert(*process, None);
                recv_core_ids.insert(*process, None);
                let comm_core_ids = Arc::new(Mutex::new((send_core_ids, recv_core_ids)));

                *spawn_fn = Box::new(move |idx, send, remote, loop_fn| {
                    let comm_core_id = {
                        let mut lock = comm_core_ids.lock().unwrap();
                        let &mut(ref send_ids, ref recv_ids) = lock.deref_mut();
                        if send {
                            send_ids[remote.unwrap()]
                        } else {
                            recv_ids[remote.unwrap()]
                        }.take().unwrap()
                    };
                    unsafe {
                        let mut comm_cpuset = ::std::mem::zeroed::<::libc::cpu_set_t>();
                        eprintln!("idx = {}, send = {:?}, remote = {:?}, core id = {:?}", idx, send, remote, comm_core_id);
                        ::libc::CPU_ZERO(&mut comm_cpuset);
                        ::libc::CPU_SET(comm_core_id, &mut comm_cpuset);
                        ::libc::sched_setaffinity(0, ::std::mem::size_of::<::libc::cpu_set_t>(), &comm_cpuset);
                    }

                    loop_fn.start()
                });
                // otherwise, pool all the remaining available core and set all comm thread's affinity
                // to the pool
            } else {
                *spawn_fn = Box::new(move |idx, send, remote, loop_fn| {
                    unsafe {
                        let mut comm_cpuset = ::std::mem::zeroed::<::libc::cpu_set_t>();
                        eprintln!("idx = {}, send = {:?}, remote = {:?}, comm core ids = {:?}", idx, send, remote, comm_core_ids);
                        ::libc::CPU_ZERO(&mut comm_cpuset);
                        for id in comm_core_ids.iter() {
                            ::libc::CPU_SET(*id, &mut comm_cpuset);
                        }
                        ::libc::sched_setaffinity(0, ::std::mem::size_of::<::libc::cpu_set_t>(), &comm_cpuset);
                    }

                    loop_fn.start()
                });
            }

            eprintln!("core ids: {:?}", core_ids);

            core_ids
        },
        _ => core_ids,
    })
}