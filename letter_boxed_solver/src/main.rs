use anyhow::{anyhow, Result};
use crossbeam_deque::{Steal, Stealer, Worker};
use itertools::Itertools;
use trie_rs::{Trie, TrieBuilder};
use rayon::prelude::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::repeat_with;
use std::thread;

/* Test problems
victim markdown

nkc
dtv
rmo
aiw

askew whirlybird

rsh
wkb
del
yia

ambidexterous

adr
meo
bxu
its
*/

/*
larger puzzles
nkc
dtv
rmo
aiw
usb

*/

const NUM_EDGES: u8 = 4;
const LETTERS_PER_EDGE: u8 = 3;

type Task = (u8, Vec<u8>);
struct TaskResult {
    word: Option<String>,
    tasks: Vec<Task>,
}

fn do_task(task: Task, edges: &Vec<u8>, trie: &Trie<u8>) -> TaskResult {
    let current_edge = task.0;
    let mut chars = task.1;
    let current: &str = std::str::from_utf8(&chars).unwrap();
    let word = if current.len() >= 3 && trie.exact_match(&current) {
        Some(current.to_string())
    } else {
        None
    };
    let mut tasks = Vec::new();
    for edge in 0..NUM_EDGES {
        if edge == current_edge {
            continue;
        }
        for idx in 0..LETTERS_PER_EDGE {
            let next = edges[(edge * LETTERS_PER_EDGE + idx) as usize];
            // push on the next char we want to test
            chars.push(next);
            let word: &str = std::str::from_utf8(&chars).unwrap();
            // if this prefix is valid, add a subtask to continue exploring it
            // todo: if this is small, skip making the subtask and just manually check if the words this prefxies are creatable
            if !trie.predictive_search(word).is_empty() {
                // make a clone for the next task
                let task = (edge, chars.clone());
                tasks.push(task);
            }
            // remove the pushed char
            chars.pop();
        }
    }
    TaskResult { word, tasks }
}

fn is_valid(chain: &Vec<&String>) -> bool {
    if chain.len() == 1 {
        return true;
    }

    let mut chain_iter = chain.iter();
    let mut c = chain_iter.next().unwrap().chars().last().unwrap();
    while let Some(next) = chain_iter.next() {
        let next_first = next.chars().next().unwrap();
        let next_last = next.chars().last().unwrap();
        if c != next_first {
            return false;
        } else {
            c = next_last;
        }
    }
    return true;
}

fn is_solution(chain: &Vec<&String>) -> bool {
    chain
        .iter()
        .map(|s| s.chars())
        .flatten()
        .collect::<HashSet<char>>()
        .len()
        == (NUM_EDGES * LETTERS_PER_EDGE).into()
}

// crossbeam-deque example
fn find_task<T>(local: &Worker<T>, stealers: &Vec<Stealer<T>>) -> Option<T> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        repeat_with(|| {
            // Try stealing a task from one of the other threads.
            stealers.iter().map(|s| s.steal()).collect()
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s: &Steal<T>| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}

fn solve(edges: Vec<u8>, trie: Trie<u8>) -> Result<()> {
    // Search for all words
    // Totally did not need to thread this, but work stealing is fun
    let num_threads = 4;
    let mut workers = Vec::new();
    for _ in 0..num_threads {
        let w: Worker<Task> = Worker::new_fifo();
        workers.push(w);
    }
    let stealers: Vec<Stealer<Task>> = workers.iter().map(|w| w.stealer()).collect();

    // Build up the queues
    let mut words = Vec::new();
    workers[0].push((99, Vec::new())); // there is no starting edge

    while workers[0].len() < 10 {
        let task = workers[0].pop().unwrap();
        let result = do_task(task, &edges, &trie);
        if let Some(w) = result.word {
            words.push(w);
        }
        for task in result.tasks {
            workers[0].push(task);
        }
    }

    words.extend(thread::scope(|s| -> Result<Vec<String>> {
        let mut words = Vec::new();
        let mut handles = Vec::new();
        for worker in workers {
            let h = s.spawn(|| -> Vec<String> {
                // println!("starting worker");
                let worker = worker;
                let mut thread_words = Vec::new();
                loop {
                    if stealers.iter().all(|s| s.is_empty()) {
                        break;
                    }
                    let task = find_task(&worker, &stealers).unwrap();
                    let result = do_task(task, &edges, &trie);
                    if let Some(w) = result.word {
                        thread_words.push(w);
                    }
                    for task in result.tasks {
                        worker.push(task);
                    }
                }
                println!("exiting worker (words found = {})", thread_words.len());
                return thread_words;
            });
            handles.push(h)
        }
        for h in handles {
            words.extend(h.join().unwrap());
        }
        Ok(words)
    })?);

    // singlethreaded
    // let mut words = Vec::new();
    // let worker = Worker::new_fifo();
    // worker.push((99, Vec::new())); // there is no starting edge

    // while !worker.is_empty() {
    //     let task = worker.pop().unwrap();
    //     let result = do_task(task, &edges, &trie);
    //     if let Some(w) = result.word {
    //         words.push(w);
    //     }
    //     for task in result.tasks {
    //         worker.push(task);
    //     }
    // }

    println!("Done creating words | len = {}", words.len());

    // sort does nothing in rayon mode, which iterates over all permutations anyway
    // words.sort_by_cached_key(|s| s.chars().collect::<HashSet<char>>().len());
    // words.reverse();

    let mut len = 0;
    let mut solutions: Vec<Vec<&String>> = Vec::new();
    while solutions.is_empty() {
        len += 1;
        println!("Looking for {} word solutions", len);
        // singlethreaded
        // for chain in words.iter().permutations(len) {
        //     if solutions.len() > 50 {
        //         break;
        //     }

        //     if is_solution(&chain) && is_valid(&chain) {
        //         println!("{} word solution {:?}", len, chain);
        //         solutions.push(chain.clone());
        //     }
        // }

        // praise rayon
        solutions.extend(words.iter()
            .permutations(len)
            .par_bridge()
            .filter(|chain| is_solution(&chain) && is_valid(&chain))
            .collect::<Vec<Vec<&String>>>());
    }
    for chain in solutions.iter() {
        println!("{} word solution {:?}", len, chain);
    }
    println!("Num solutions (capped at 50) = {}", solutions.len());
    Ok(())
}

fn main() -> Result<()> {
    // get today's puzzle
    let mut edges: Vec<u8> = Vec::new();
    for _ in 0..NUM_EDGES {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let edge: Vec<u8> = buf
            .strip_suffix("\n")
            .ok_or(anyhow!("No whitespace?"))?
            .as_bytes()
            .to_vec();
        if edge.len() != LETTERS_PER_EDGE.into() {
            println!(
                "Invalid puzzle given, expected {} letters per edge, was given edge {}",
                LETTERS_PER_EDGE, buf
            );
        }
        edges.extend(edge);
    }
    println!("Solving {:?}", edges);

    // create trie
    let mut builder = TrieBuilder::new();
    let file = File::open("./english-words/words_alpha.txt")?;
    // let file = File::open("/usr/share/dict/words")?;
    for line in io::BufReader::new(file).lines() {
        builder.push(line?);
    }
    let trie = builder.build();

    // Solve
    return solve(edges, trie);
}
