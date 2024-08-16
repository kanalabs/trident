use crate::Rpc;
use std::time::SystemTime;


// Generic entry point fn to select the next rpc and return its position
pub fn pick(list: &mut [Rpc]) -> (Rpc, Option<usize>) {
    // If len is 1, return the only element
    if list.len() == 1 {
        return (list[0].clone(), Some(0));
    } else if list.is_empty() {
        return (Rpc::default(), None);
    }

    algo(list)
}

// Sorting algo
pub fn argsort(data: &[Rpc]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<usize>>();

    // Use sort_by_cached_key with a closure that compares latency
    // Uses pdqsort and does not allocate so should be fast
    indices.sort_unstable_by_key(|&index| data[index].status.latency as u64);

    indices
}

// Selection algorithms
//
// Selected via features. selection-weighed-round-robin is a default feature.
// In order to have custom algos, you must add and enable the feature,
// as well as modify the cfg of the default algo to accomodate your new feature.
//
#[cfg(all(
    feature = "selection-weighed-round-robin",
    not(feature = "selection-random"),
    not(feature = "old-weighted-round-robin"),
))]
fn algo(list: &mut [Rpc]) -> (Rpc, Option<usize>) {
    // Sort by latency
    let indices = argsort(list);

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get current time")
        .as_micros();

    // Picks the second fastest one rpc that meets our requirements
    // Also take into account min_delta_time

    // Set fastest rpc as default
    let mut choice = indices[0];
    let mut choice_consecutive = 0;
    for i in indices.iter().rev() {
        if list[*i].max_consecutive > list[*i].consecutive
            && (time - list[*i].last_used > list[*i].min_time_delta)
        {
            choice = *i;
            choice_consecutive = list[*i].consecutive;
        }

        // remove consecutive
        list[*i].consecutive = 0;
    }

    // If no RPC has been selected, fall back to the fastest RPC
    list[choice].consecutive = choice_consecutive + 1;
    list[choice].last_used = time;
    (list[choice].clone(), Some(choice))
}
