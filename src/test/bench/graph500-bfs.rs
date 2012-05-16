/**

An implementation of the Graph500 Bread First Search problem in Rust.

*/

use std;
import std::time;
import std::map;
import std::map::hashmap;
import std::deque;
import std::deque::t;
//import std::par;
import io::writer_util;
import comm::*;
import int::abs;

type node_id = i64;
type graph = [map::set<node_id>];
type bfs_result = [node_id];

iface queue<T: send> {
    fn add_back(T);
    fn pop_front() -> T;
    fn size() -> uint;
}

#[doc="Creates a queue based on ports and channels.

This is admittedly not ideal, but it will help us work around the deque
bugs for the time being."]
fn create_queue<T: send>() -> queue<T> {
    type repr<T: send> = {
        p : port<T>,
        c : chan<T>,
        mut s : uint,
    };

    let p = port();
    let c = chan(p);

    impl<T: send> of queue<T> for repr<T> {
        fn add_back(x : T) {
            let x = x;
            send(self.c, x);
            self.s += 1u;
        }

        fn pop_front() -> T {
            self.s -= 1u;
            recv(self.p)
        }

        fn size() -> uint { self.s }
    }

    let Q : repr<T> = { p : p, c : c, mut s : 0u };
    Q as queue::<T>
}

fn make_edges(scale: uint, edgefactor: uint) -> [(node_id, node_id)] {
    let r = rand::rng();

    fn choose_edge(i: node_id, j: node_id, scale: uint, r: rand::rng)
        -> (node_id, node_id) {

        let A = 0.57;
        let B = 0.19;
        let C = 0.19;
 
        if scale == 0u {
            (i, j)
        }
        else {
            let i = i * 2;
            let j = j * 2;
            let scale = scale - 1u;
            
            let x = r.next_float();

            if x < A {
                choose_edge(i, j, scale, r)
            }
            else {
                let x = x - A;
                if x < B {
                    choose_edge(i + 1, j, scale, r)
                }
                else {
                    let x = x - B;
                    if x < C {
                        choose_edge(i, j + 1, scale, r)
                    }
                    else {
                        choose_edge(i + 1, j + 1, scale, r)
                    }
                }
            }
        }
    }

    vec::from_fn((1u << scale) * edgefactor) {|_i|
        choose_edge(0, 0, scale, r)
    }
}

fn make_graph(N: uint, edges: [(node_id, node_id)]) -> graph {
    let graph = vec::from_fn(N) {|_i| map::int_hash() };

    vec::each(edges) {|e| 
        let (i, j) = e;
        map::set_add(graph[i], j);
        map::set_add(graph[j], i);
        true
    }

    graph
}

#[doc="Returns a vector of all the parents in the BFS tree rooted at key.

Nodes that are unreachable have a parent of -1."]
fn bfs(graph: graph, key: node_id) -> bfs_result {
    let marks : [mut node_id] 
        = vec::to_mut(vec::from_elem(vec::len(graph), -1));

    let Q = create_queue();

    Q.add_back(key);
    marks[key] = key;

    while Q.size() > 0u {
        let t = Q.pop_front();

        graph[t].each_key() {|k| 
            if marks[k] == -1 {
                marks[k] = t;
                Q.add_back(k);
            }
            true
        };
    }

    vec::from_mut(marks)
}

#[doc="Performs at least some of the validation in the Graph500 spec."]
fn validate(edges: [(node_id, node_id)], 
            root: node_id, tree: bfs_result) -> bool {
    // There are 5 things to test. Below is code for each of them.

    // 1. The BFS tree is a tree and does not contain cycles.
    //
    // We do this by iterating over the tree, and tracing each of the
    // parent chains back to the root. While we do this, we also
    // compute the levels for each node.

    log(info, "Verifying tree structure...");

    let mut status = true;
    let level = tree.map() {|parent| 
        let mut parent = parent;
        let mut path = [];

        if parent == -1 {
            // This node was not in the tree.
            -1
        }
        else {
            while parent != root {
                if vec::contains(path, parent) {
                    status = false;
                }

                path += [parent];
                parent = tree[parent];
            }

            // The length of the path back to the root is the current
            // level.
            path.len() as int
        }
    };
    
    if !status { ret status }

    // 2. Each tree edge connects vertices whose BFS levels differ by
    //    exactly one.

    log(info, "Verifying tree edges...");

    let status = tree.alli() {|k, parent|
        if parent != root && parent != -1 {
            level[parent] == level[k] - 1
        }
        else {
            true
        }
    };

    if !status { ret status }

    // 3. Every edge in the input list has vertices with levels that
    //    differ by at most one or that both are not in the BFS tree.

    log(info, "Verifying graph edges...");

    let status = edges.all() {|e| 
        let (u, v) = e;

        abs(level[u] - level[v]) <= 1
    };

    if !status { ret status }    

    // 4. The BFS tree spans an entire connected component's vertices.

    // This is harder. We'll skip it for now...

    // 5. A node and its parent are joined by an edge of the original
    //    graph.

    log(info, "Verifying tree and graph edges...");

    let status = par::alli(tree) {|u, v|
        if v == -1 || u as int == root {
            true
        }
        else {
            edges.contains((u as int, v)) || edges.contains((v, u as int))
        }
    };

    if !status { ret status }    

    // If we get through here, all the tests passed!
    true
}

fn main() {
    let scale = 14u;

    let start = time::precise_time_s();
    let edges = make_edges(scale, 16u);
    let stop = time::precise_time_s();

    io::stdout().write_line(#fmt("Generated %? edges in %? seconds.",
                                 vec::len(edges), stop - start));

    let start = time::precise_time_s();
    let graph = make_graph(1u << scale, edges);
    let stop = time::precise_time_s();

    let mut total_edges = 0u;
    vec::each(graph) {|edges| total_edges += edges.size(); true };

    io::stdout().write_line(#fmt("Generated graph with %? edges in %? seconds.",
                                 total_edges / 2u,
                                 stop - start));

    let root = 0;
    
    let start = time::precise_time_s();
    let bfs_tree = bfs(graph, root);
    let stop = time::precise_time_s();

    io::stdout().write_line(#fmt("BFS completed in %? seconds.",
                                 stop - start));

    let start = time::precise_time_s();
    assert(validate(edges, root, bfs_tree));
    let stop = time::precise_time_s();

    io::stdout().write_line(#fmt("Validation completed in %? seconds.",
                                 stop - start));
}


// par stuff /////////////////////////////////////////////////////////

mod par {
import comm::port;
import comm::chan;
import comm::send;
import comm::recv;
import task::spawn;

iface future<T: send> {
    fn get() -> T;
}

type future_<T: send> = {
    mut slot : option<T>,
    port : port<T>,
};

impl<T: send> of future<T> for future_<T> {
    fn get() -> T {
        get(self)
    }
}

fn get<T: send>(f: future_<T>) -> T {
    alt(f.slot) {
      some(x) { x }
      none {
        let x = recv(f.port);
        f.slot = some(x);
        x
      }
    }
}


#[doc="Executes a bit of code asynchronously.

Returns a handle that can be used to retrieve the result at your
leisure."]
fn future<T: send>(thunk : fn~() -> T) -> future<T> {
    let p = port();
    let c = chan(p);

    spawn() {||
        send(c, thunk());
    }

    {mut slot: none::<T>, port : p} as future::<T>
}

#[doc="The maximum number of tasks this module will spawn for a single
 operationg."]
const max_tasks : uint = 32u;

#[doc="The minimum number of elements each task will process."]
const min_granularity : uint = 1024u;

#[doc="An internal helper to map a function over a large vector and
 return the intermediate results.

This is used to build most of the other parallel vector functions,
like map or alli."]
fn map_slices<A: send, B: send>(xs: [A], f: fn~(uint, [A]) -> B) -> [B] {
    let len = xs.len();
    if len < min_granularity {
        // This is a small vector, fall back on the normal map.
        [f(0u, xs)]
    }
    else {
        let num_tasks = uint::min(max_tasks, len / min_granularity);

        let items_per_task = len / num_tasks;

        let mut futures = [];
        let mut base = 0u;
        while base < len {
            let slice = vec::slice(xs, base,
                                   uint::min(len, base + items_per_task));
            futures += [future() {|copy base|
                f(base, slice)
            }];
            base += items_per_task;
        }

        futures.map() {|ys|
            ys.get()
        }
    }
}

#[doc="A parallel version of map."]
fn map<A: send, B: send>(xs: [A], f: fn~(A) -> B) -> [B] {
    vec::concat(map_slices(xs) {|_base, slice|
        map(slice, f)
    })
}

#[doc="Returns true if the function holds for all elements in the vector."]
fn alli<A: send>(xs: [A], f: fn~(uint, A) -> bool) -> bool {
    vec::all(map_slices(xs) {|base, slice|
        slice.alli() {|i, x|
            f(i + base, x)
        }
    }) {|x| x }
}
}